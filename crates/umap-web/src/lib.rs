use gloo_file::{File, callbacks::FileReader};
use gloo_net::http::Request;
use log::{error, info};
use serde::Deserialize;
use yew::prelude::*;
use yew_plotly::Plotly;
use yew_plotly::plotly::{Layout, Plot, Scatter, Scatter3D, common::Mode};

#[derive(Deserialize, Debug, Clone)]
struct Point2D {
    #[serde(rename = "id")]
    _id: i64,
    source: String,
    chunk_index: i64,
    score: f32,
    text_preview: String,
    x: f32,
    y: f32,
}

#[derive(Deserialize, Debug, Clone)]
struct Point3D {
    #[serde(rename = "id")]
    _id: i64,
    source: String,
    chunk_index: i64,
    score: f32,
    text_preview: String,
    x: f32,
    y: f32,
    z: f32,
}
#[derive(Deserialize, Debug, Clone)]
struct Search2DResp {
    points: Vec<Point2D>,
}
#[derive(Deserialize, Debug, Clone)]
struct Search3DResp {
    points: Vec<Point3D>,
}

#[function_component(App)]
pub fn app() -> Html {
    let query = use_state(String::new);
    let dims = use_state(|| 2usize);
    let k = use_state(|| 30usize);
    let plot = use_state(Plot::new);
    let status = use_state(String::new);
    let error_msg = use_state(|| Option::<String>::None);
    let method = use_state(|| String::from("umap"));
    let tokens_per_chunk = use_state(|| 1000usize);
    let overlap = use_state(|| 300usize);
    let n_neighbors = use_state(|| 15usize);
    let min_dist = use_state(|| 0.1f32);
    let n_epochs = use_state(|| 200usize);
    let lr = use_state(|| 1.0f32);
    let neg_rate = use_state(|| 5usize);
    let repulsion = use_state(|| 1.0f32);
    let spread = use_state(|| 1.0f32);
    let readers = use_mut_ref(Vec::<FileReader>::new);

    let run_search = {
        let query = query.clone();
        let dims = dims.clone();
        let k = k.clone();
        let plot = plot.clone();
        let n_neighbors_state = n_neighbors.clone();
        let min_dist_state = min_dist.clone();
        let n_epochs_state = n_epochs.clone();
        let lr_state = lr.clone();
        let neg_rate_state = neg_rate.clone();
        let repulsion_state = repulsion.clone();
        let spread_state = spread.clone();
        let method_state = method.clone();
        let status_state = status.clone();
        let err_state = error_msg.clone();
        Callback::from(move |_| {
            let query = (*query).clone();
            let dims_val = *dims;
            let k_val = *k;
            let plot = plot.clone();
            let nn = *n_neighbors_state;
            let md = *min_dist_state;
            let ne = *n_epochs_state;
            let lrv = *lr_state;
            let ngr = *neg_rate_state;
            let rep = *repulsion_state;
            let spr = *spread_state;
            let method = (*method_state).clone();
            let status_state = status_state.clone();
            let err_state = err_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!(
                    "/api/search?query={}&k={}&dims={}&method={}&n_neighbors={}&min_dist={}&n_epochs={}&learning_rate={}&negative_sample_rate={}&repulsion_strength={}&spread={}",
                    urlencoding::encode(&query),
                    k_val,
                    dims_val,
                    method,
                    nn,
                    md,
                    ne,
                    lrv,
                    ngr,
                    rep,
                    spr
                );
                info!("search GET {}", &url);
                match Request::get(&url).send().await {
                    Ok(resp) => {
                        if dims_val == 2 {
                            match resp.json::<Search2DResp>().await {
                                Ok(data) => {
                                    let xs: Vec<f64> =
                                        data.points.iter().map(|p| p.x as f64).collect();
                                    let ys: Vec<f64> =
                                        data.points.iter().map(|p| p.y as f64).collect();
                                    let scores: Vec<f64> =
                                        data.points.iter().map(|p| p.score as f64).collect();
                                    let texts: Vec<String> = data
                                        .points
                                        .iter()
                                        .map(|p| {
                                            format!(
                                                "{}:{} — {} (similarity: {:.3})",
                                                p.source, p.chunk_index, p.text_preview, p.score
                                            )
                                        })
                                        .collect();
                                    // Normalize scores to [0,1] range within this result set for better color contrast
                                    let min_score =
                                        scores.iter().cloned().fold(f64::INFINITY, f64::min);
                                    let max_score =
                                        scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                                    let score_range = (max_score - min_score).max(0.001);
                                    let colors: Vec<String> = scores
                                        .iter()
                                        .map(|&s| {
                                            let norm =
                                                ((s - min_score) / score_range).clamp(0.0, 1.0);
                                            // Blue (low) -> Cyan -> Green -> Yellow -> Red (high)
                                            let r = (norm * 255.0) as u8;
                                            let g = (if norm < 0.5 {
                                                norm * 2.0
                                            } else {
                                                2.0 - norm * 2.0
                                            } * 255.0)
                                                as u8;
                                            let b = ((1.0 - norm) * 255.0) as u8;
                                            format!("rgb({},{},{})", r, g, b)
                                        })
                                        .collect();
                                    let trace = Scatter::new(xs, ys)
                                        .mode(Mode::Markers)
                                        .text_array(texts)
                                        .marker(
                                            yew_plotly::plotly::common::Marker::new()
                                                .color_array(colors)
                                                .size(10),
                                        );
                                    let mut plt = Plot::new();
                                    let title_text = format!(
                                        "Scores: {:.3}-{:.3} (Red=High, Blue=Low)",
                                        min_score, max_score
                                    );
                                    let layout = Layout::new().title(title_text.as_str().into());
                                    plt.add_trace(trace);
                                    plt.set_layout(layout);
                                    plot.set(plt);
                                    status_state
                                        .set(format!("Fetched {} points", data.points.len()));
                                    err_state.set(None);
                                }
                                Err(e) => {
                                    error!("JSON error: {}", e);
                                    err_state.set(Some(format!("JSON error: {}", e)));
                                }
                            }
                        } else {
                            match resp.json::<Search3DResp>().await {
                                Ok(data) => {
                                    let xs: Vec<f64> =
                                        data.points.iter().map(|p| p.x as f64).collect();
                                    let ys: Vec<f64> =
                                        data.points.iter().map(|p| p.y as f64).collect();
                                    let zs: Vec<f64> =
                                        data.points.iter().map(|p| p.z as f64).collect();
                                    let scores: Vec<f64> =
                                        data.points.iter().map(|p| p.score as f64).collect();
                                    let texts: Vec<String> = data
                                        .points
                                        .iter()
                                        .map(|p| {
                                            format!(
                                                "{}:{} — {} (similarity: {:.3})",
                                                p.source, p.chunk_index, p.text_preview, p.score
                                            )
                                        })
                                        .collect();
                                    // Normalize scores to [0,1] range within this result set for better color contrast
                                    let min_score =
                                        scores.iter().cloned().fold(f64::INFINITY, f64::min);
                                    let max_score =
                                        scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                                    let score_range = (max_score - min_score).max(0.001);
                                    let colors: Vec<String> = scores
                                        .iter()
                                        .map(|&s| {
                                            let norm =
                                                ((s - min_score) / score_range).clamp(0.0, 1.0);
                                            // Blue (low) -> Cyan -> Green -> Yellow -> Red (high)
                                            let r = (norm * 255.0) as u8;
                                            let g = (if norm < 0.5 {
                                                norm * 2.0
                                            } else {
                                                2.0 - norm * 2.0
                                            } * 255.0)
                                                as u8;
                                            let b = ((1.0 - norm) * 255.0) as u8;
                                            format!("rgb({},{},{})", r, g, b)
                                        })
                                        .collect();
                                    let trace = Scatter3D::new(xs, ys, zs)
                                        .mode(Mode::Markers)
                                        .text_array(texts)
                                        .marker(
                                            yew_plotly::plotly::common::Marker::new()
                                                .color_array(colors)
                                                .size(6),
                                        );
                                    let mut plt = Plot::new();
                                    let title_text = format!(
                                        "Scores: {:.3}-{:.3} (Red=High, Blue=Low)",
                                        min_score, max_score
                                    );
                                    let layout = Layout::new().title(title_text.as_str().into());
                                    plt.add_trace(trace);
                                    plt.set_layout(layout);
                                    plot.set(plt);
                                    status_state
                                        .set(format!("Fetched {} points", data.points.len()));
                                    err_state.set(None);
                                }
                                Err(e) => {
                                    error!("JSON error: {}", e);
                                    err_state.set(Some(format!("JSON error: {}", e)));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("HTTP error: {}", e);
                        err_state.set(Some(format!("HTTP error: {}", e)));
                    }
                }
            });
        })
    };

    let on_input = {
        let query = query.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            query.set(input.value());
        })
    };
    let on_dims = {
        let dims = dims.clone();
        Callback::from(move |_| dims.set(if *dims == 2 { 3 } else { 2 }))
    };
    let on_k = {
        let k = k.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = input.value().parse::<usize>() {
                k.set(v);
            }
        })
    };

    // File upload controls
    let file_input_ref = use_node_ref();
    let on_upload = {
        let file_input_ref = file_input_ref.clone();
        let tokens_per_chunk = tokens_per_chunk.clone();
        let overlap = overlap.clone();
        let status = status.clone();
        let err_state = error_msg.clone();
        let readers = readers.clone();
        Callback::from(move |_| {
            let input = file_input_ref.cast::<web_sys::HtmlInputElement>().unwrap();
            if let Some(files) = input.files() {
                for i in 0..files.length() {
                    if let Some(file) = files.get(i) {
                        let f: File = File::from(file);
                        let filename = f.name();
                        let tpc = *tokens_per_chunk;
                        let ov = *overlap;
                        let status2 = status.clone();
                        let err2 = err_state.clone();
                        let reader = gloo_file::callbacks::read_as_text(&f, move |res| match res {
                            Ok(text) => {
                                let payload = serde_json::json!({
                                    "filename": filename,
                                    "content": text,
                                    "tokens_per_chunk": tpc,
                                    "overlap": ov,
                                    "dim": 512,
                                });
                                let status3 = status2.clone();
                                let err3 = err2.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    info!("ingest POST /api/ingest_text");
                                    match Request::post("/api/ingest_text")
                                        .json(&payload)
                                        .unwrap()
                                        .send()
                                        .await
                                    {
                                        Ok(resp) => {
                                            let txt = resp.text().await.unwrap_or_default();
                                            status3.set(format!("Ingested {}", txt));
                                            err3.set(None);
                                        }
                                        Err(e) => {
                                            error!("Ingest error: {}", e);
                                            err3.set(Some(format!("Ingest error: {}", e)));
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                error!("File read error: {}", e);
                                err2.set(Some(format!("File read error: {}", e)));
                            }
                        });
                        readers.borrow_mut().push(reader);
                    }
                }
            }
        })
    };

    html! {
        <>
        <style>
            {".hoverlayer .hovertext, .hoverlayer .hovertext rect { background-color: #fffacd !important; fill: #fffacd !important; }"}
            {".hoverlayer .hovertext path { fill: #fffacd !important; }"}
            {".hoverlayer .hovertext text, .hoverlayer .hovertext .name, g.hovertext text { fill: #000000 !important; }"}
        </style>
        <div style="font-family: system-ui, sans-serif; padding: 1rem;">
            <h2>{"UMAP Visualizer"}</h2>
            <div style="display:flex; gap: 0.5rem; align-items: center;">
                <input type="text" placeholder="Search query..." oninput={on_input} style="width: 380px;" />
                <button onclick={run_search.clone()}>{"Search"}</button>
                <button onclick={on_dims.clone()}>{format!("Toggle {}D", if *dims == 2 {"3"} else {"2"})}</button>
                <label>{"k:"}</label>
                <input type="number" value={k.to_string()} oninput={on_k} min="5" max="200" />
                <label>{"method:"}</label>
                <select onchange={{ let method = method.clone(); Callback::from(move |e: Event| {
                    let sel: web_sys::HtmlSelectElement = e.target_unchecked_into();
                    method.set(sel.value());
                }) }}>
                    <option value="umap" selected={(*method)=="umap"}>{"UMAP"}</option>
                    <option value="pca" selected={(*method)=="pca"}>{"PCA"}</option>
                </select>
                <label style="margin-left:0.5rem;">{"n_neighbors:"}</label>
                <input type="number" min="2" max="200" value={n_neighbors.to_string()} oninput={{ let n_neighbors = n_neighbors.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); if let Ok(v)=input.value().parse(){ n_neighbors.set(v);} }) }} />
                <label>{"min_dist:"}</label>
                <input type="number" step="0.01" value={min_dist.to_string()} oninput={{ let min_dist = min_dist.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); if let Ok(v)=input.value().parse(){ min_dist.set(v);} }) }} />
                <label>{"epochs:"}</label>
                <input type="number" min="10" max="5000" value={n_epochs.to_string()} oninput={{ let n_epochs = n_epochs.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); if let Ok(v)=input.value().parse(){ n_epochs.set(v);} }) }} />
                <label>{"lr:"}</label>
                <input type="number" step="0.1" value={lr.to_string()} oninput={{ let lr = lr.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); if let Ok(v)=input.value().parse(){ lr.set(v);} }) }} />
                <label>{"neg_rate:"}</label>
                <input type="number" min="1" max="50" value={neg_rate.to_string()} oninput={{ let neg_rate = neg_rate.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); if let Ok(v)=input.value().parse(){ neg_rate.set(v);} }) }} />
                <label>{"repulsion:"}</label>
                <input type="number" step="0.1" value={repulsion.to_string()} oninput={{ let repulsion = repulsion.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); if let Ok(v)=input.value().parse(){ repulsion.set(v);} }) }} />
                <span style="margin-left:1rem;color:#666;">{ (*status).clone() }</span>
            </div>
            {
                if let Some(msg) = &*error_msg {
                    html!{<div style="margin-top:0.5rem; padding: 0.5rem; background:#fee; color:#a00; border:1px solid #f88; border-radius:4px;">{ format!("Error: {}", msg) }</div>}
                } else { html!{} }
            }
            <div style="display:flex; gap: 0.5rem; align-items:center; margin-top:0.5rem;">
                <input type="file" multiple=true ref={file_input_ref.clone()} />
                <label>{"tokens/chunk:"}</label>
                <input type="number" min="100" max="5000" value={tokens_per_chunk.to_string()} oninput={{ let tokens_per_chunk = tokens_per_chunk.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); if let Ok(v)=input.value().parse(){ tokens_per_chunk.set(v);} }) }} />
                <label>{"overlap:"}</label>
                <input type="number" min="0" max="4000" value={overlap.to_string()} oninput={{ let overlap = overlap.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); if let Ok(v)=input.value().parse(){ overlap.set(v);} }) }} />
                <button onclick={on_upload}>{"Ingest Files"}</button>
            </div>
            <div style="height: 640px; margin-top: 1rem;">
                <Plotly plot={(*plot).clone()} />
            </div>
        </div>
        </>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
