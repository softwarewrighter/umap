#!/bin/bash
# Setup demo app source files
# Can be run from anywhere - automatically finds project root

set -e

# Find project root by looking for workspace Cargo.toml
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔍 Project root: $PROJECT_ROOT"

# Verify we found the right directory
if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
    echo "❌ Error: Could not find Cargo.toml at $PROJECT_ROOT"
    exit 1
fi

echo "🚀 Setting up demo app source files..."
echo ""

# Create demo-app at project root
DEMO_APP="$PROJECT_ROOT/demo-app"
echo "📁 Creating $DEMO_APP..."
mkdir -p "$DEMO_APP/src"

# Create Cargo.toml
echo "📝 Creating demo-app/Cargo.toml..."
cat > "$DEMO_APP/Cargo.toml" << 'EOF'
[package]
name = "umap-demo"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-plotly = "0.3"
gloo-net = "0.5"
gloo-console = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = ["HtmlSelectElement", "MouseEvent", "Event"] }
EOF

# Create Trunk.toml - builds TO docs/
echo "📝 Creating demo-app/Trunk.toml..."
cat > "$DEMO_APP/Trunk.toml" << 'EOF'
[build]
target = "index.html"
dist = "../docs"
public_url = "/umap/"

[serve]
port = 8888

[[hooks]]
stage = "post_build"
command = "sh"
command_arguments = ["-c", "echo 'Build complete. Output in docs/'"]
EOF

# Create index.html
echo "📝 Creating demo-app/index.html..."
cat > "$DEMO_APP/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>UMAP Text Visualizer - Demo</title>
    <link data-trunk rel="copy-dir" href="assets" />
    <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        h1 { color: #333; margin-bottom: 10px; }
        .subtitle { color: #666; margin-bottom: 30px; }
        .controls { margin-bottom: 20px; padding: 15px; background: #f9f9f9; border-radius: 4px; }
        select { padding: 8px 16px; margin-right: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px; }
        .plot-container { margin-top: 20px; min-height: 600px; }
        .info { margin-top: 20px; padding: 15px; background: #e3f2fd; border-left: 4px solid #2196f3; border-radius: 4px; }
        .info strong { color: #1976d2; }
    </style>
</head>
<body>
</body>
</html>
EOF

# Create src/lib.rs
echo "📝 Creating demo-app/src/lib.rs..."
cat > "$DEMO_APP/src/lib.rs" << 'EOF'
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
struct Point2D { x: f64, y: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    id: i64,
    source: String,
    chunk_index: i64,
    text_preview: String,
    score: f64,
    #[serde(rename = "point_2d")]
    point_2d: Option<Point2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse {
    points: Vec<SearchResult>,
    query: String,
    k: i64,
    method: String,
    dimensions: i64,
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
        let points: Vec<_> = api_response.points.iter().filter_map(|p| p.point_2d.as_ref()).collect();
        let x: Vec<f64> = points.iter().map(|p| p.x).collect();
        let y: Vec<f64> = points.iter().map(|p| p.y).collect();
        let text: Vec<String> = api_response.points.iter()
            .map(|p| format!("Score: {:.3}<br>{}", p.score, p.text_preview.chars().take(100).collect::<String>()))
            .collect();

        let trace = Scatter::default().x(x).y(y).mode(Mode::Markers).text_array(text)
            .marker(Marker::default().size(10).color("rgb(93, 164, 214)")
                .line(Line::new().width(0.5).color("rgb(255, 255, 255)")));

        let layout = Layout::default()
            .title(Title::new(&format!("{} - {}", api_response.method.to_uppercase(), api_response.query)))
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
        html! {
            <div class="info">
                <strong>{"Query: "}</strong> { &api_response.query }<br/>
                <strong>{"Method: "}</strong> { &api_response.method.to_uppercase() }<br/>
                <strong>{"Results: "}</strong> { api_response.points.len() } { " chunks" }
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
EOF

# Create demo data directory in demo-app assets
echo "📁 Creating demo-app/assets/demo-data..."
mkdir -p "$DEMO_APP/assets/demo-data"

echo ""
echo "✅ Demo app source created!"
echo ""
echo "📂 Files created:"
find "$DEMO_APP" -type f | sort
echo ""
echo "📍 Locations:"
echo "  Source: $DEMO_APP/"
echo "  Output: $PROJECT_ROOT/docs/"
echo ""
echo "📊 Next steps:"
echo "  1. Create demo data: bash $PROJECT_ROOT/scripts/create-demo-data.sh"
echo "  2. Build: cd $DEMO_APP && trunk build --release"
echo "  3. Serve: cd $DEMO_APP && trunk serve --port 8888"
