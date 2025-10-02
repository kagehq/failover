use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::Response,
    routing::any,
    Router,
};
use clap::Parser;
use http::header;
use reqwest::Client;
use serde_json;
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time;
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::{error, info, warn};
use url::Url;

#[derive(Parser, Clone)]
#[command(name = "failover-proxy")]
#[command(about = "A reverse proxy with automatic failover")]
struct Args {
    #[arg(long, help = "Primary upstream URL")]
    primary: String,

    #[arg(long, help = "Backup upstream URL")]
    backup: String,

    #[arg(long, default_value = "0.0.0.0:8080", help = "Listen address")]
    listen: String,

    #[arg(long, default_value = "2s", help = "Health check interval")]
    check_interval: humantime::Duration,

    #[arg(long, default_value = "3", help = "Fail threshold")]
    fail_threshold: u32,

    #[arg(long, default_value = "2", help = "Recover threshold")]
    recover_threshold: u32,

    #[arg(long, default_value = "10MB", help = "Max request body size")]
    max_body: String,

    #[arg(long, help = "Config file path")]
    config: Option<String>,

    #[arg(long, help = "Enable JSON logging")]
    json_logs: bool,
}

#[derive(Clone)]
struct AppState {
    primary: String,
    backup: String,
    client: Client,
    is_primary_healthy: Arc<AtomicBool>,
    fail_count: Arc<std::sync::atomic::AtomicU32>,
    recover_count: Arc<std::sync::atomic::AtomicU32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.json_logs {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter("failover_proxy=info")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("failover_proxy=info")
            .init();
    }

    let listen_addr: SocketAddr = args.listen.parse()?;

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let app_state = AppState {
        primary: args.primary.clone(),
        backup: args.backup.clone(),
        client: client.clone(),
        is_primary_healthy: Arc::new(AtomicBool::new(true)),
        fail_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        recover_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
    };

    // Start health check task
    let health_state = app_state.clone();
    let args_clone = args.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(args_clone.check_interval.into());
        loop {
            interval.tick().await;
            check_health(&health_state, &args_clone).await;
        }
    });

    // Parse max body size
    let max_body_bytes = parse_size(&args.max_body)?;

    let app = Router::new()
        .route("/*path", any(proxy_handler))
        .route("/", any(proxy_handler))
        .route("/__failover/health", axum::routing::get(health_handler))
        .route("/__failover/state", axum::routing::get(state_handler))
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(max_body_bytes))
        )
        .with_state(app_state);

    info!("Starting failover proxy on {}", listen_addr);
    info!("Primary: {}", args.primary);
    info!("Backup: {}", args.backup);

    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn state_handler(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let is_primary_healthy = state.is_primary_healthy.load(Ordering::Relaxed);
    let fail_count = state.fail_count.load(Ordering::Relaxed);
    let recover_count = state.recover_count.load(Ordering::Relaxed);
    
    axum::Json(serde_json::json!({
        "on_backup": !is_primary_healthy,
        "primary": state.primary,
        "backup": state.backup,
        "fail_count": fail_count,
        "recover_count": recover_count,
        "since_unix": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }))
}

async fn check_health(state: &AppState, args: &Args) {
    let primary_url = &state.primary;
    let is_healthy = state.is_primary_healthy.load(Ordering::Relaxed);

    match health_check(primary_url, &state.client).await {
        Ok(_) => {
            if !is_healthy {
                let recover_count = state.recover_count.fetch_add(1, Ordering::Relaxed) + 1;
                if recover_count >= args.recover_threshold {
                    state.is_primary_healthy.store(true, Ordering::Relaxed);
                    state.fail_count.store(0, Ordering::Relaxed);
                    state.recover_count.store(0, Ordering::Relaxed);
                    info!("Primary recovered, switching back");
                }
            } else {
                state.fail_count.store(0, Ordering::Relaxed);
            }
        }
        Err(e) => {
            if is_healthy {
                let fail_count = state.fail_count.fetch_add(1, Ordering::Relaxed) + 1;
                if fail_count >= args.fail_threshold {
                    state.is_primary_healthy.store(false, Ordering::Relaxed);
                    state.recover_count.store(0, Ordering::Relaxed);
                    warn!("Primary failed ({}), switching to backup: {}", fail_count, e);
                }
            }
        }
    }
}

async fn health_check(url: &str, client: &Client) -> anyhow::Result<()> {
    let response = client
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("HTTP {}", response.status()))
    }
}

async fn proxy_handler(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Response<Body>, StatusCode> {
    let is_primary_healthy = state.is_primary_healthy.load(Ordering::Relaxed);
    let target_url = if is_primary_healthy {
        &state.primary
    } else {
        &state.backup
    };

    let target_uri = match build_target_uri(target_url, &uri) {
        Ok(uri) => uri,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let mut request_builder = state.client
        .request(method, &target_uri)
        .body(body.to_vec());

    // Copy headers (excluding hop-by-hop headers)
    for (name, value) in headers.iter() {
        if !is_hop_by_hop_header(name) {
            if let Ok(header_value) = HeaderValue::from_bytes(value.as_bytes()) {
                request_builder = request_builder.header(name, header_value);
            }
        }
    }

    match request_builder.send().await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            let body = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            let mut response_builder = Response::builder().status(status);
            
            // Copy response headers
            for (name, value) in headers.iter() {
                if !is_hop_by_hop_header(name) {
                    if let Ok(header_value) = HeaderValue::from_bytes(value.as_bytes()) {
                        response_builder = response_builder.header(name, header_value);
                    }
                }
            }

            response_builder
                .body(Body::from(body))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(e) => {
            error!("Proxy request failed: {}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

fn build_target_uri(base: &str, original_uri: &Uri) -> anyhow::Result<String> {
    let base_url = Url::parse(base)?;
    let path_and_query = original_uri.path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    
    let target_url = base_url.join(path_and_query)?;
    Ok(target_url.to_string())
}

fn is_hop_by_hop_header(name: &header::HeaderName) -> bool {
    matches!(
        name,
        &header::CONNECTION
            | &header::PROXY_AUTHENTICATE
            | &header::PROXY_AUTHORIZATION
            | &header::TE
            | &header::TRAILER
            | &header::TRANSFER_ENCODING
            | &header::UPGRADE
    )
}

fn parse_size(size_str: &str) -> anyhow::Result<usize> {
    let size_str = size_str.to_uppercase();
    let (number, unit) = if size_str.ends_with("KB") {
        (&size_str[..size_str.len() - 2], "KB")
    } else if size_str.ends_with("MB") {
        (&size_str[..size_str.len() - 2], "MB")
    } else if size_str.ends_with("GB") {
        (&size_str[..size_str.len() - 2], "GB")
    } else {
        (size_str.as_str(), "")
    };

    let number: usize = number.parse()?;
    let multiplier = match unit {
        "KB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        _ => 1,
    };

    Ok(number * multiplier)
}
