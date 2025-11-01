use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_plotly::plotly::{
    Plot, Configuration, Scatter,
    common::{Marker, Mode, Title, Line},
    layout::{Axis, Layout},
};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::HtmlSelectElement;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    id: i64,
    source: String,
    chunk_index: i64,
    text_preview: String,
    score: f64,
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse {
    points: Vec<SearchResult>,
}

#[derive(Clone)]
enum DemoData {
    RomanceUMAP2D,
    RomancePCA2D,
}

impl DemoData {
    fn name(&self) -> &str {
        match self {
            Self::RomanceUMAP2D => "Romance (2D UMAP)",
            Self::RomancePCA2D => "Romance (2D PCA)",
        }
    }
    fn file(&self) -> &str {
        match self {
            Self::RomanceUMAP2D => "assets/demo-data/romance-2d-umap.json",
            Self::RomancePCA2D => "assets/demo-data/romance-2d-pca.json",
        }
    }
    fn description(&self) -> &str {
        match self {
            Self::RomanceUMAP2D => "2D UMAP projection",
            Self::RomancePCA2D => "2D PCA (for comparison)",
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let data = use_state(|| None::<ApiResponse>);
    let selected_demo = use_state(|| 0usize);
    let loading = use_state(|| false);
    let demos = vec![DemoData::RomanceUMAP2D, DemoData::RomancePCA2D];

    {
        let data = data.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            let data = data.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match Request::get("assets/demo-data/romance-2d-umap.json").send().await {
                    Ok(response) => {
                        match response.json::<ApiResponse>().await {
                            Ok(api_response) => data.set(Some(api_response)),
                            Err(e) => gloo_console::error!("JSON error:", format!("{:?}", e)),
                        }
                    }
                    Err(e) => gloo_console::error!("Fetch error:", format!("{:?}", e)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_demo_change = {
        let data = data.clone();
        let selected_demo = selected_demo.clone();
        let loading = loading.clone();
        let demos = demos.clone();
        Callback::from(move |e: Event| {
            let target: HtmlSelectElement = e.target_unchecked_into();
            let idx = target.value().parse::<usize>().unwrap_or(0);
            selected_demo.set(idx);
            let data = data.clone();
            let loading = loading.clone();
            let file = demos[idx].file().to_string();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match Request::get(&file).send().await {
                    Ok(response) => {
                        match response.json::<ApiResponse>().await {
                            Ok(api_response) => data.set(Some(api_response)),
                            Err(e) => gloo_console::error!("JSON error:", format!("{:?}", e)),
                        }
                    }
                    Err(e) => gloo_console::error!("Fetch error:", format!("{:?}", e)),
                }
                loading.set(false);
            });
        })
    };

    let plot = if let Some(api_response) = (*data).as_ref() {
        let x: Vec<f64> = api_response.points.iter().map(|p| p.x).collect();
        let y: Vec<f64> = api_response.points.iter().map(|p| p.y).collect();
        let text: Vec<String> = api_response.points.iter()
            .map(|p| format!("Score: {:.3}<br>{}", p.score, p.text_preview.chars().take(100).collect::<String>()))
            .collect();

        let trace = Scatter::default().x(x).y(y).mode(Mode::Markers).text_array(text)
            .marker(Marker::default().size(10).color("rgb(93, 164, 214)")
                .line(Line::new().width(0.5).color("rgb(255, 255, 255)")));

        let demo_title = demos[*selected_demo].name();
        let layout = Layout::default()
            .title(Title::new(&format!("{} - Text Embeddings", demo_title)))
            .x_axis(Axis::default().title(Title::new("Dimension 1")).zero_line(true))
            .y_axis(Axis::default().title(Title::new("Dimension 2")).zero_line(true))
            .height(700).hover_mode(yew_plotly::plotly::layout::HoverMode::Closest);

        let mut plot = Plot::new();
        plot.add_trace(trace);
        plot.set_layout(layout);
        plot.set_configuration(Configuration::default().responsive(true));
        html! { <yew_plotly::Plotly plot={plot} /> }
    } else {
        html! { <div>{ if *loading { "Loading..." } else { "No data" } }</div> }
    };

    let info_box = if let Some(api_response) = (*data).as_ref() {
        let demo = &demos[*selected_demo];
        html! {
            <div class="info">
                <strong>{"Demo: "}</strong> { demo.name() }<br/>
                <strong>{"Method: "}</strong> { demo.description() }<br/>
                <strong>{"Results: "}</strong> { api_response.points.len() } { " chunks visualized" }
            </div>
        }
    } else { html! {} };

    html! {
        <div class="container">
            <h1>{ "UMAP Text Visualizer - Demo" }</h1>
            <div class="subtitle">{ "Dimensionality reduction (UMAP/PCA) of text embeddings" }</div>
            <div class="controls">
                <label>
                    <strong>{ "Select Demo: " }</strong>
                    <select onchange={on_demo_change}>
                        { for demos.iter().enumerate().map(|(i, demo)| {
                            html! {
                                <option value={i.to_string()} selected={i == *selected_demo}>
                                    { demo.name() } {" - "} { demo.description() }
                                </option>
                            }
                        }) }
                    </select>
                </label>
            </div>
            { info_box }
            <div class="plot-container">{ plot }</div>
            <div style="margin-top: 30px; padding: 15px; background: #fff3cd; border-left: 4px solid #ffc107;">
                <strong>{ "Note: " }</strong>
                { "Static demo with pre-computed results. Clone repo for full interactive app." }
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
