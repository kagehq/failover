use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::Response,
    routing::any,
    Router,
};
use chrono::{DateTime, Utc};
use clap::Parser;
use http::header;
use reqwest::Client;
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

    #[arg(
        long,
        help = "Webhook URL for incident notifications (Slack, Discord, etc.)"
    )]
    webhook_url: Option<String>,

    #[arg(long, help = "Webhook notification format (slack or discord)")]
    webhook_format: Option<String>,
}

#[derive(Clone)]
struct AppState {
    primary: String,
    backup: String,
    client: Client,
    is_primary_healthy: Arc<AtomicBool>,
    fail_count: Arc<std::sync::atomic::AtomicU32>,
    recover_count: Arc<std::sync::atomic::AtomicU32>,
    failover_timestamp: Arc<tokio::sync::RwLock<Option<DateTime<Utc>>>>,
}

#[derive(serde::Serialize)]
struct IncidentReport {
    event_type: String,
    timestamp: String,
    primary_url: String,
    backup_url: String,
    fail_count: u32,
    duration: Option<String>,
    message: String,
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

    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let app_state = AppState {
        primary: args.primary.clone(),
        backup: args.backup.clone(),
        client: client.clone(),
        is_primary_healthy: Arc::new(AtomicBool::new(true)),
        fail_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        recover_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        failover_timestamp: Arc::new(tokio::sync::RwLock::new(None)),
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
        .layer(ServiceBuilder::new().layer(RequestBodyLimitLayer::new(max_body_bytes)))
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

                    // Calculate downtime duration
                    let duration = {
                        let timestamp = state.failover_timestamp.read().await;
                        timestamp.map(|start| {
                            let duration = Utc::now().signed_duration_since(start);
                            format!("{} seconds", duration.num_seconds())
                        })
                    };

                    info!("Primary recovered, switching back");

                    // Send recovery notification
                    let report = IncidentReport {
                        event_type: "recovery".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        primary_url: state.primary.clone(),
                        backup_url: state.backup.clone(),
                        fail_count: 0,
                        duration,
                        message: format!(
                            "Primary service {} has recovered and is now healthy. Traffic restored to primary.",
                            state.primary
                        ),
                    };
                    send_incident_notification(state, args, &report).await;

                    // Clear failover timestamp
                    *state.failover_timestamp.write().await = None;
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

                    // Record failover timestamp
                    *state.failover_timestamp.write().await = Some(Utc::now());

                    warn!(
                        "Primary failed ({}), switching to backup: {}",
                        fail_count, e
                    );

                    // Send failover notification
                    let report = IncidentReport {
                        event_type: "failover".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        primary_url: state.primary.clone(),
                        backup_url: state.backup.clone(),
                        fail_count,
                        duration: None,
                        message: format!(
                            "Primary service {} failed after {} consecutive health check failures. Traffic switched to backup: {}. Error: {}",
                            state.primary, fail_count, state.backup, e
                        ),
                    };
                    send_incident_notification(state, args, &report).await;
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

    let mut request_builder = state
        .client
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
    let path_and_query = original_uri
        .path_and_query()
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

async fn send_incident_notification(state: &AppState, args: &Args, report: &IncidentReport) {
    if let Some(webhook_url) = &args.webhook_url {
        let format = args.webhook_format.as_deref().unwrap_or("slack");

        let payload = match format {
            "discord" => format_discord_message(report),
            _ => format_slack_message(report),
        };

        match state.client.post(webhook_url).json(&payload).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Incident notification sent successfully");
                } else {
                    warn!(
                        "Failed to send incident notification: {}",
                        response.status()
                    );
                }
            }
            Err(e) => {
                error!("Error sending incident notification: {}", e);
            }
        }
    }
}

fn format_slack_message(report: &IncidentReport) -> serde_json::Value {
    let color = if report.event_type == "failover" {
        "#ff0000" // Red for failover
    } else {
        "#00ff00" // Green for recovery
    };

    let emoji = if report.event_type == "failover" {
        "🚨"
    } else {
        "✅"
    };

    serde_json::json!({
        "attachments": [{
            "color": color,
            "title": format!("{} Failover Incident Report", emoji),
            "fields": [
                {
                    "title": "Event",
                    "value": report.event_type.to_uppercase(),
                    "short": true
                },
                {
                    "title": "Timestamp",
                    "value": report.timestamp,
                    "short": true
                },
                {
                    "title": "Primary",
                    "value": report.primary_url,
                    "short": true
                },
                {
                    "title": "Backup",
                    "value": report.backup_url,
                    "short": true
                },
                {
                    "title": "Details",
                    "value": report.message,
                    "short": false
                }
            ],
            "footer": "Failover Proxy",
            "ts": chrono::Utc::now().timestamp()
        }]
    })
}

fn format_discord_message(report: &IncidentReport) -> serde_json::Value {
    let color = if report.event_type == "failover" {
        16711680 // Red for failover
    } else {
        65280 // Green for recovery
    };

    let emoji = if report.event_type == "failover" {
        "🚨"
    } else {
        "✅"
    };

    serde_json::json!({
        "embeds": [{
            "title": format!("{} Failover Incident Report", emoji),
            "color": color,
            "fields": [
                {
                    "name": "Event",
                    "value": report.event_type.to_uppercase(),
                    "inline": true
                },
                {
                    "name": "Timestamp",
                    "value": report.timestamp,
                    "inline": true
                },
                {
                    "name": "Primary",
                    "value": report.primary_url,
                    "inline": false
                },
                {
                    "name": "Backup",
                    "value": report.backup_url,
                    "inline": false
                },
                {
                    "name": "Details",
                    "value": report.message,
                    "inline": false
                }
            ],
            "footer": {
                "text": "Failover Proxy"
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }]
    })
}
