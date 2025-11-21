use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_plotly::plotly::{
    Plot, Configuration, Scatter, Scatter3D,
    common::{Marker, Mode, Title, Line, Label, Font},
    layout::{Axis, Layout},
};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::HtmlSelectElement;

// Build-time constants
const BUILD_GIT_SHA: &str = env!("BUILD_GIT_SHA");
const BUILD_HOST: &str = env!("BUILD_HOST");
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
const COPYRIGHT: &str = "Copyright (c) 2025 Michael A. Wright";
const GITHUB_URL: &str = "https://github.com/sw-viz/umap";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    id: i64,
    source: String,
    chunk_index: i64,
    text_preview: String,
    score: f64,
    x: f64,
    y: f64,
    #[serde(default)]
    z: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse {
    points: Vec<SearchResult>,
}

#[derive(Clone)]
enum DemoData {
    RomanceUMAP2D,
    RomancePCA2D,
    RomanceUMAP3D,
    RomancePCA3D,
}

impl DemoData {
    fn name(&self) -> &str {
        match self {
            Self::RomanceUMAP2D => "Romance (2D UMAP)",
            Self::RomancePCA2D => "Romance (2D PCA)",
            Self::RomanceUMAP3D => "Romance (3D UMAP)",
            Self::RomancePCA3D => "Romance (3D PCA)",
        }
    }
    fn file(&self) -> &str {
        match self {
            Self::RomanceUMAP2D => "assets/demo-data/romance-2d-umap.json",
            Self::RomancePCA2D => "assets/demo-data/romance-2d-pca.json",
            Self::RomanceUMAP3D => "assets/demo-data/romance-3d-umap.json",
            Self::RomancePCA3D => "assets/demo-data/romance-3d-pca.json",
        }
    }
    fn description(&self) -> &str {
        match self {
            Self::RomanceUMAP2D => "2D UMAP projection",
            Self::RomancePCA2D => "2D PCA (linear reduction)",
            Self::RomanceUMAP3D => "3D UMAP projection",
            Self::RomancePCA3D => "3D PCA (linear reduction)",
        }
    }
    fn is_3d(&self) -> bool {
        matches!(self, Self::RomanceUMAP3D | Self::RomancePCA3D)
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let data = use_state(|| None::<ApiResponse>);
    let selected_demo = use_state(|| 0usize);
    let loading = use_state(|| false);
    let demos = vec![
        DemoData::RomanceUMAP2D,
        DemoData::RomancePCA2D,
        DemoData::RomanceUMAP3D,
        DemoData::RomancePCA3D,
    ];

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
            gloo_console::info!("Loading:", file.clone());
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
        let demo = &demos[*selected_demo];
        let x: Vec<f64> = api_response.points.iter().map(|p| p.x).collect();
        let y: Vec<f64> = api_response.points.iter().map(|p| p.y).collect();
        let text: Vec<String> = api_response.points.iter()
            .map(|p| format!("Score: {:.3}<br>{}", p.score, p.text_preview.chars().take(100).collect::<String>()))
            .collect();

        let demo_title = demo.name();

        if demo.is_3d() {
            // 3D visualization
            let z: Vec<f64> = api_response.points.iter()
                .map(|p| p.z.unwrap_or(0.0))
                .collect();

            let trace = Scatter3D::default()
                .x(x)
                .y(y)
                .z(z)
                .mode(Mode::Markers)
                .text_array(text)
                .marker(Marker::default()
                    .size(6)
                    .color("rgb(93, 164, 214)")
                    .line(Line::new().width(0.5).color("rgb(255, 255, 255)")));

            let hover_label = Label::new()
                .background_color("#fffacd")  // Light yellow background
                .font(Font::new().color("#000000"));  // Black text

            let layout = Layout::default()
                .title(Title::new(&format!("{} - Text Embeddings", demo_title)))
                .height(700)
                .hover_mode(yew_plotly::plotly::layout::HoverMode::Closest)
                .hover_label(hover_label);

            let mut plot = Plot::new();
            plot.add_trace(trace);
            plot.set_layout(layout);
            plot.set_configuration(Configuration::default().responsive(true));
            html! { <yew_plotly::Plotly plot={plot} /> }
        } else {
            // 2D visualization
            let trace = Scatter::default()
                .x(x)
                .y(y)
                .mode(Mode::Markers)
                .text_array(text)
                .marker(Marker::default()
                    .size(10)
                    .color("rgb(93, 164, 214)")
                    .line(Line::new().width(0.5).color("rgb(255, 255, 255)")));

            let hover_label = Label::new()
                .background_color("#fffacd")  // Light yellow background
                .font(Font::new().color("#000000"));  // Black text

            let layout = Layout::default()
                .title(Title::new(&format!("{} - Text Embeddings", demo_title)))
                .x_axis(Axis::default().title(Title::new("Dimension 1")).zero_line(true))
                .y_axis(Axis::default().title(Title::new("Dimension 2")).zero_line(true))
                .height(700)
                .hover_mode(yew_plotly::plotly::layout::HoverMode::Closest)
                .hover_label(hover_label);

            let mut plot = Plot::new();
            plot.add_trace(trace);
            plot.set_layout(layout);
            plot.set_configuration(Configuration::default().responsive(true));
            html! { <yew_plotly::Plotly plot={plot} /> }
        }
    } else {
        html! { <div>{ if *loading { "Loading..." } else { "No data" } }</div> }
    };

    let info_box = if let Some(api_response) = (*data).as_ref() {
        let demo = &demos[*selected_demo];
        html! {
            <div class="info">
                <strong>{"Demo: "}</strong> { demo.name() }<br/>
                <strong>{"Method: "}</strong> { demo.description() }<br/>
                <strong>{"Dimensions: "}</strong> { if demo.is_3d() { "3D" } else { "2D" } }<br/>
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
            <div style="margin-top: 30px; padding: 15px; background: #e8f5e9; border-left: 4px solid #4caf50;">
                <strong>{ "About UMAP vs PCA:" }</strong><br/>
                { "UMAP (Uniform Manifold Approximation and Projection) preserves local structure and reveals clusters, " }
                { "while PCA (Principal Component Analysis) is a linear method that may not capture non-linear relationships. " }
                { "Compare the visualizations to see how UMAP can better separate semantic groups in the text data." }
            </div>
            <div style="margin-top: 15px; padding: 15px; background: #fff3cd; border-left: 4px solid #ffc107;">
                <strong>{ "Note: " }</strong>
                { "Static demo with pre-computed results. Clone repo for full interactive app with custom queries." }
            </div>
            <div style="margin-top: 30px; padding: 20px; background: #f5f5f5; border-top: 2px solid #ddd; font-size: 14px;">
                <div style="margin-bottom: 10px;">
                    <strong>{ COPYRIGHT }</strong>
                </div>
                <div style="margin-bottom: 10px;">
                    <a href={GITHUB_URL} target="_blank" style="color: #0066cc;">
                        { "GitHub Repository" }
                    </a>
                    { " | " }
                    <a href={format!("{}/blob/main/LICENSE", GITHUB_URL)} target="_blank" style="color: #0066cc;">
                        { "MIT License" }
                    </a>
                </div>
                <div style="font-size: 12px; color: #666;">
                    { "Build: " }
                    { BUILD_GIT_SHA }
                    { " | Host: " }
                    { BUILD_HOST }
                    { " | " }
                    { BUILD_TIMESTAMP }
                </div>
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
