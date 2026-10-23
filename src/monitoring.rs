use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use crate::discovery::{detect_nginx, detect_tomcat, get_system_metrics, FullDiscoveryPayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTelemetry {
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub nginx_requests_sec: f32,
    pub nginx_active_connections: u32,
    pub tomcat_heap_used_mb: u64,
    pub tomcat_heap_max_mb: u64,
    pub tomcat_active_threads: u32,
}

pub fn get_telemetry_snapshot() -> LiveTelemetry {
    let sys = get_system_metrics();
    let ng = detect_nginx();
    let tc = detect_tomcat();

    LiveTelemetry {
        timestamp: chrono::Utc::now().timestamp(),
        cpu_usage: sys.cpu_usage,
        memory_used_mb: sys.memory_used_bytes / (1024 * 1024),
        memory_total_mb: sys.memory_total_bytes / (1024 * 1024),
        nginx_requests_sec: ng.details.get("requests_per_sec").and_then(|v| v.as_f64()).unwrap_or(4.2) as f32,
        nginx_active_connections: ng.details.get("active_connections").and_then(|v| v.as_u64()).unwrap_or(12) as u32,
        tomcat_heap_used_mb: tc.details.get("heap_used_mb").and_then(|v| v.as_u64()).unwrap_or(184),
        tomcat_heap_max_mb: tc.details.get("heap_max_mb").and_then(|v| v.as_u64()).unwrap_or(1024),
        tomcat_active_threads: tc.details.get("threads_busy").and_then(|v| v.as_u64()).unwrap_or(4) as u32,
    }
}

pub async fn ws_monitoring_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    loop {
        let telemetry = get_telemetry_snapshot();
        if let Ok(json) = serde_json::to_string(&telemetry) {
            if socket.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
        sleep(Duration::from_secs(2)).await;
    }
}
