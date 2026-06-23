use axum::{
    Form,
    Router,
    extract::State,
    response::Html,
    routing::{get, get_service, post},
};
use filter::filter::filter_chain::FilterChain;
use filter::filter::config::{
    encounter::EncounterFilter,
    price::PriceFilter,
    seat::SeatPositionFilter,
};
use parser::core::encounter::MatchNature;
use scanner::core::app_state::AppState as ScannerAppState;
use scanner::core::scan::{ScanConfig, ScanMode};
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tower_http::services::ServeDir;

const INDEX_HTML: &str = include_str!("pages/index.html");
const CONFIG_UPDATED_HTML: &str = include_str!("pages/config_updated.html");
const ROOT_CARGO_TOML: &str = include_str!("../../../../Cargo.toml");

#[derive(Clone)]
struct AppState {
    config_tx: Arc<watch::Sender<ScanConfig>>,
    state_tx: Option<Arc<watch::Sender<ScannerAppState>>>,
}

#[derive(Deserialize)]
struct ScanConfigForm {
    interval: u64,
    mode: String,
    nature: String,
    #[serde(default)]
    price_min: Option<String>,
    #[serde(default)]
    price_max: Option<String>,
    #[serde(default)]
    seat_category: Option<String>,
    #[serde(default)]
    seat_bloc: Option<String>,
    #[serde(default)]
    seat_row: Option<String>,
    #[serde(default)]
    side_by_side: Option<String>,
    #[serde(default)]
    match_title: Option<String>,
    #[serde(default)]
    is_preview: Option<String>,
    #[serde(default)]
    proxy_enabled: Option<String>,
}

#[derive(Deserialize)]
struct ScanStateForm {
    state: String,
}

fn extract_root_app_version() -> String {
    let mut in_package_section = false;

    for line in ROOT_CARGO_TOML.lines() {
        let trimmed = line.trim();

        if trimmed == "[package]" {
            in_package_section = true;
            continue;
        }

        if in_package_section && trimmed.starts_with('[') {
            break;
        }

        if in_package_section && trimmed.starts_with("version") {
            if let Some((_, value)) = trimmed.split_once('=') {
                let version = value.trim().trim_matches('"');
                if !version.is_empty() {
                    return version.to_string();
                }
            }
        }
    }

    "unknown".to_string()
}


/// Renders the admin page with the current scanner configuration pre-filled in the form.
///
/// # Arguments
/// * `state` - Shared application state containing the current `ScanConfig` available through
///   the `watch::Sender`.
///
/// # Returns
/// Returns an `Html<String>` response containing the rendered `INDEX_HTML` template populated
/// with current configuration values.
async fn index(State(state): State<AppState>) -> Html<String> {
    let config = state.config_tx.borrow();
    let app_version = extract_root_app_version();
    let interval = config.interval;
    let chk_scan_toggle = if state
        .state_tx
        .as_ref()
        .map(|tx| *tx.borrow() == ScannerAppState::Stopped)
        .unwrap_or(false)
    {
        "checked"
    } else {
        ""
    };

    let sel_passive    = if config.mode == ScanMode::PassiveScan    { "selected" } else { "" };
    let sel_aggressive = if config.mode == ScanMode::AggressiveScan { "selected" } else { "" };
    let sel_rugby      = if config.nature == MatchNature::Rugby      { "selected" } else { "" };
    let sel_basketball = if config.nature == MatchNature::Basketball { "selected" } else { "" };
    let sel_other      = if config.nature == MatchNature::Other      { "selected" } else { "" };

    let chain           = config.filter_chain.as_deref();
    let price_min       = chain.and_then(|c| c.price_min()).map(|v| v.to_string()).unwrap_or_default();
    let price_max       = chain.and_then(|c| c.price_max()).map(|v| v.to_string()).unwrap_or_default();
    let seat_category   = chain.and_then(|c| c.seat_category()).unwrap_or("").to_string();
    let seat_bloc       = chain.and_then(|c| c.seat_bloc()).unwrap_or("").to_string();
    let seat_row        = chain.and_then(|c| c.seat_row()).unwrap_or("").to_string();
    let side_by_side    = chain.and_then(|c| c.side_by_side()).map(|v| v.to_string()).unwrap_or_default();
    let match_title     = chain.and_then(|c| c.encounter_title()).unwrap_or("").to_string();
    let chk_preview     = if config.is_preview { "checked" } else { "" };
    let chk_proxy       = if config.proxy_enabled { "checked" } else { "" };

    let html = INDEX_HTML
        .replace("{interval}", &interval.to_string())
        .replace("{sel_passive}", sel_passive)
        .replace("{sel_aggressive}", sel_aggressive)
        .replace("{sel_rugby}", sel_rugby)
        .replace("{sel_basketball}", sel_basketball)
        .replace("{sel_other}", sel_other)
        .replace("{price_min}", &price_min)
        .replace("{price_max}", &price_max)
        .replace("{seat_category}", &seat_category)
        .replace("{seat_bloc}", &seat_bloc)
        .replace("{seat_row}", &seat_row)
        .replace("{side_by_side}", &side_by_side)
        .replace("{match_title}", &match_title)
        .replace("{chk_preview}", chk_preview)
        .replace("{chk_proxy}", chk_proxy)
        .replace("{chk_scan_toggle}", chk_scan_toggle)
        .replace("{app_version}", &app_version);

    Html(html)
}

/// Updates the runtime scanner configuration from the admin form,
/// rebuilds the filter chain, and broadcasts it through the watch channel.
///
/// # Arguments
/// * `state` - Shared application state containing the `watch::Sender<ScanConfig>` used to
///   publish the updated scanner configuration.
/// * `form` - Submitted admin form values (`ScanConfigForm`) used to update scan mode,
///   match nature, preview flag, and filter criteria.
///
/// # Returns
/// Returns an `Html<String>` response containing the confirmation page content
/// (`CONFIG_UPDATED_HTML`) once the new configuration has been sent.
async fn update_config(
    State(state): State<AppState>,
    Form(form): Form<ScanConfigForm>,
) -> Html<String> {
    let mut new_config = state.config_tx.borrow().clone();

    new_config.interval = form.interval;
    new_config.mode = match form.mode.as_str() {
        "aggressive" => ScanMode::AggressiveScan,
        _ => ScanMode::PassiveScan,
    };
    new_config.nature = match form.nature.as_str() {
        "basketball" => MatchNature::Basketball,
        "other" => MatchNature::Other,
        _ => MatchNature::Rugby,
    };

    let price_min       = form.price_min.filter(|s| !s.is_empty()).and_then(|s| s.parse::<f64>().ok());
    let price_max       = form.price_max.filter(|s| !s.is_empty()).and_then(|s| s.parse::<f64>().ok());
    let seat_category   = form.seat_category.filter(|s| !s.is_empty());
    let seat_bloc       = form.seat_bloc.filter(|s| !s.is_empty());
    let seat_row        = form.seat_row.filter(|s| !s.is_empty());
    let side_by_side    = form.side_by_side.filter(|s| !s.is_empty()).and_then(|s| s.parse::<u64>().ok());
    let match_title     = form.match_title.filter(|s| !s.is_empty());

    let position = if seat_category.is_some() || seat_bloc.is_some() || seat_row.is_some() {
        Some(parser::core::seat::SeatComposition {
            category: seat_category.clone().unwrap_or_default(),
            bloc: seat_bloc.clone().unwrap_or_default(),
            row: seat_row.clone().unwrap_or_default(),
            seat_number: 0,
        })
    } else {
        None
    };

    // new_config.match_title = match_title.clone();
    new_config.is_preview = form.is_preview.is_some();
    new_config.proxy_enabled = form.proxy_enabled.is_some();

    // Build the FilterChain from the form values
    let mut chain = FilterChain::new();
    if let Some(title) = match_title {
        chain = chain.add(EncounterFilter::new(Some(title)));
    }
    if price_min.is_some() || price_max.is_some() {
        chain = chain.add(PriceFilter::new(price_min, price_max));
    }
    if position.is_some() || side_by_side.is_some() {
        chain = chain.add(SeatPositionFilter::new(
            position,
            side_by_side.map(|n| n as usize),
        ));
    }
    new_config.filter_chain = Some(Arc::new(chain));

    println!("[DEBUG] Config mise à jour : interval={}s, nature={:?}", new_config.interval, new_config.nature);

    scanner::core::config_file::write_to_file(&new_config, "config_scan.json").unwrap_or_else(|e| eprintln!("❌ Impossible d'écrire config_scan.json : {}", e));
    state.config_tx.send(new_config).ok();

    Html(CONFIG_UPDATED_HTML.to_string())
}

async fn update_state(
    State(state): State<AppState>,
    Form(form): Form<ScanStateForm>,
) -> Html<String> {
    let Some(state_tx) = &state.state_tx else {
        return Html("State channel not configured".to_string());
    };

    let next_state = match form.state.trim().to_ascii_lowercase().as_str() {
        "running" | "run" | "play" | "resume" => ScannerAppState::Running,
        "stopped" | "stop" | "pause" => ScannerAppState::Stopped,
        _ => return Html("Invalid state. Use running or stopped".to_string()),
    };

    if state_tx.send(next_state).is_err() {
        return Html("Failed to propagate state".to_string());
    }

    Html(format!("State updated: {:?}", next_state))
}

/// Starts the admin panel web server, allowing runtime configuration of the scanner through a web interface.
/// The server listens on the port provided by `ADMIN_PANEL_PORT` (default: `3000`) and
/// provides endpoints for viewing the current configuration and updating it through a form submission.
/// 
/// # Arguments
/// * `config_tx` - A `watch::Sender<ScanConfig>` used to broadcast updated scanner configurations to the scanning task when changes are made through the admin panel.
///
/// # Returns
/// This function runs indefinitely, serving the admin panel until the application is terminated.
pub async fn run(config_tx: watch::Sender<ScanConfig>) {
    run_with_state(config_tx, None).await;
}

pub async fn run_with_state(
    config_tx: watch::Sender<ScanConfig>,
    state_tx: Option<watch::Sender<ScannerAppState>>,
) {
    let state = AppState {
        config_tx: Arc::new(config_tx),
        state_tx: state_tx.map(Arc::new),
    };

    let port = std::env::var("ADMIN_PANEL_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(3000);

    let static_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interface/static");

    let app = Router::new()
        .route("/", get(index))
        .route("/config", post(update_config))
        .route("/state", post(update_state))
        .nest_service("/static", get_service(ServeDir::new(static_dir)))
        .with_state(state);

    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&bind_addr).await.unwrap();
    println!("Server as start on http://localhost:{}", port);
    axum::serve(listener, app).await.unwrap();
}