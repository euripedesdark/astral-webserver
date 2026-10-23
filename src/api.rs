use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::audit::AuditTrail;
use crate::auth::{ApiResponse, AuthenticatedOperator};
use crate::config::AppConfig;
use crate::discovery::{get_full_discovery, FullDiscoveryPayload};
use crate::monitoring::{get_telemetry_snapshot, ws_monitoring_handler, LiveTelemetry};
use crate::nginx::{
    control_nginx_service, list_ssl_certs, list_vhosts, read_nginx_logs, save_vhost,
    test_nginx_config, LogEntry, SslCertInfo, VHostInfo,
};
use crate::security::{apply_security_settings, get_security_settings, SecuritySettings};
use crate::terminal::{exec_terminal_cmd, get_terminal_status};
use crate::tomcat::{
    control_tomcat_service, list_apps, parse_connectors, read_jvm_config, read_tomcat_config_file,
    read_tomcat_logs, save_tomcat_config_file, JvmConfig, TomcatAppInfo, TomcatConnectorInfo,
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub audit: AuditTrail,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Auth
        .route("/api/auth/login", post(crate::auth::login_handler))
        .route("/api/auth/me", get(crate::auth::me_handler))
        .route("/api/auth/logout", post(crate::auth::logout_handler))
        .route("/api/auth/providers", get(crate::auth::providers_handler))

        // Discovery
        .route("/api/discovery", get(discovery_handler))

        // Nginx Management
        .route("/api/nginx/status", get(nginx_status_handler))
        .route("/api/nginx/control", post(nginx_control_handler))
        .route("/api/nginx/test", get(nginx_test_handler).post(nginx_test_handler))
        .route("/api/nginx/vhosts", get(nginx_vhosts_handler))
        .route("/api/nginx/vhosts/save", post(nginx_save_vhost_handler))
        .route("/api/nginx/ssl", get(nginx_ssl_handler))
        .route("/api/nginx/logs", get(nginx_logs_handler))

        // Tomcat Management
        .route("/api/tomcat/status", get(tomcat_status_handler))
        .route("/api/tomcat/control", post(tomcat_control_handler))
        .route("/api/tomcat/apps", get(tomcat_apps_handler))
        .route("/api/tomcat/config", get(tomcat_get_config_handler))
        .route("/api/tomcat/config/save", post(tomcat_save_config_handler))
        .route("/api/tomcat/connectors", get(tomcat_connectors_handler))
        .route("/api/tomcat/jvm", get(tomcat_jvm_handler))
        .route("/api/tomcat/logs", get(tomcat_logs_handler))

        // Security
        .route("/api/security/settings", get(security_get_handler))
        .route("/api/security/apply", post(security_apply_handler))

        // Monitoring
        .route("/api/monitoring/stats", get(monitoring_stats_handler))
        .route("/api/monitoring/ws", get(ws_monitoring_handler))

        // Backups
        .route("/api/backups", get(backups_list_handler))
        .route("/api/backups/create", post(backups_create_handler))

        // Audit Trail
        .route("/api/audit", get(audit_list_handler))

        // Terminal
        .route("/api/terminal/status", get(get_terminal_status))
        .route("/api/terminal/exec", post(exec_terminal_cmd))

        .with_state(state)
}

// ==============================================================================
// HANDLERS
// ==============================================================================

async fn discovery_handler(_operator: AuthenticatedOperator) -> Json<ApiResponse<FullDiscoveryPayload>> {
    let payload = get_full_discovery();
    Json(ApiResponse {
        success: true,
        message: "Descoberta de servidores web e telemetria".to_string(),
        data: Some(payload),
    })
}

#[derive(Debug, Deserialize)]
struct ServiceControlPayload {
    action: String,
}

async fn nginx_status_handler(_operator: AuthenticatedOperator) -> Json<ApiResponse<crate::discovery::ServerDiscoveryInfo>> {
    let info = crate::discovery::detect_nginx();
    Json(ApiResponse {
        success: true,
        message: "Status Nginx".to_string(),
        data: Some(info),
    })
}

async fn nginx_control_handler(
    operator: AuthenticatedOperator,
    State(state): State<AppState>,
    Json(payload): Json<ServiceControlPayload>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match control_nginx_service(&payload.action) {
        Ok(msg) => {
            state.audit.record(&operator.username, &operator.provider, "Nginx", &payload.action, "service", "SUCCESS", &msg);
            Ok(Json(ApiResponse {
                success: true,
                message: msg.clone(),
                data: Some(msg),
            }))
        }
        Err(err) => {
            state.audit.record(&operator.username, &operator.provider, "Nginx", &payload.action, "service", "FAILED", &err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: err,
                    data: None,
                }),
            ))
        }
    }
}

async fn nginx_test_handler(_operator: AuthenticatedOperator) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match test_nginx_config() {
        Ok(out) => Ok(Json(ApiResponse {
            success: true,
            message: "Sintaxe Nginx válida".to_string(),
            data: Some(out),
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: format!("Erro de sintaxe:\n{}", err),
                data: None,
            }),
        )),
    }
}

async fn nginx_vhosts_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<VHostInfo>>> {
    let vhosts = list_vhosts(&state.config.nginx_conf_dir);
    Json(ApiResponse {
        success: true,
        message: format!("{} Virtual Hosts encontrados", vhosts.len()),
        data: Some(vhosts),
    })
}

#[derive(Debug, Deserialize)]
struct SaveVHostPayload {
    path: String,
    content: String,
}

async fn nginx_save_vhost_handler(
    operator: AuthenticatedOperator,
    State(state): State<AppState>,
    Json(payload): Json<SaveVHostPayload>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    let target = PathBuf::from(&payload.path);
    match save_vhost(&state.config.backup_dir, &target, &payload.content) {
        Ok(msg) => {
            state.audit.record(&operator.username, &operator.provider, "Nginx", "save_vhost", &payload.path, "SUCCESS", &msg);
            Ok(Json(ApiResponse {
                success: true,
                message: msg.clone(),
                data: Some(msg),
            }))
        }
        Err(err) => {
            state.audit.record(&operator.username, &operator.provider, "Nginx", "save_vhost", &payload.path, "FAILED", &err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: err,
                    data: None,
                }),
            ))
        }
    }
}

async fn nginx_ssl_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<SslCertInfo>>> {
    let certs = list_ssl_certs(&state.config.nginx_conf_dir);
    Json(ApiResponse {
        success: true,
        message: format!("{} certificados SSL detectados", certs.len()),
        data: Some(certs),
    })
}

#[derive(Debug, Deserialize)]
struct LogQuery {
    r#type: Option<String>,
    limit: Option<usize>,
    query: Option<String>,
    level: Option<String>,
}

async fn nginx_logs_handler(
    _operator: AuthenticatedOperator,
    Query(params): Query<LogQuery>,
) -> Json<ApiResponse<Vec<LogEntry>>> {
    let log_type = params.r#type.as_deref().unwrap_or("access");
    let limit = params.limit.unwrap_or(100);
    let entries = read_nginx_logs(log_type, limit, params.query.as_deref());
    Json(ApiResponse {
        success: true,
        message: format!("{} registros de log obtidos", entries.len()),
        data: Some(entries),
    })
}

async fn tomcat_status_handler(_operator: AuthenticatedOperator) -> Json<ApiResponse<crate::discovery::ServerDiscoveryInfo>> {
    let info = crate::discovery::detect_tomcat();
    Json(ApiResponse {
        success: true,
        message: "Status Apache Tomcat".to_string(),
        data: Some(info),
    })
}

async fn tomcat_control_handler(
    operator: AuthenticatedOperator,
    State(state): State<AppState>,
    Json(payload): Json<ServiceControlPayload>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match control_tomcat_service(&payload.action) {
        Ok(msg) => {
            state.audit.record(&operator.username, &operator.provider, "Tomcat", &payload.action, "service", "SUCCESS", &msg);
            Ok(Json(ApiResponse {
                success: true,
                message: msg.clone(),
                data: Some(msg),
            }))
        }
        Err(err) => {
            state.audit.record(&operator.username, &operator.provider, "Tomcat", &payload.action, "service", "FAILED", &err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: err,
                    data: None,
                }),
            ))
        }
    }
}

async fn tomcat_apps_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<TomcatAppInfo>>> {
    let apps = list_apps(&state.config.tomcat_webapps_dir);
    Json(ApiResponse {
        success: true,
        message: format!("{} aplicações Tomcat detectadas", apps.len()),
        data: Some(apps),
    })
}

#[derive(Debug, Deserialize)]
struct TomcatConfigQuery {
    file: String,
}

async fn tomcat_get_config_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
    Query(query): Query<TomcatConfigQuery>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match read_tomcat_config_file(&state.config.tomcat_conf_dir, &query.file) {
        Ok(content) => Ok(Json(ApiResponse {
            success: true,
            message: format!("Arquivo {} carregado", query.file),
            data: Some(content),
        })),
        Err(err) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                message: err,
                data: None,
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
struct TomcatSaveConfigPayload {
    file: String,
    content: String,
}

async fn tomcat_save_config_handler(
    operator: AuthenticatedOperator,
    State(state): State<AppState>,
    Json(payload): Json<TomcatSaveConfigPayload>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match save_tomcat_config_file(&state.config.backup_dir, &state.config.tomcat_conf_dir, &payload.file, &payload.content) {
        Ok(msg) => {
            state.audit.record(&operator.username, &operator.provider, "Tomcat", "save_config", &payload.file, "SUCCESS", &msg);
            Ok(Json(ApiResponse {
                success: true,
                message: msg.clone(),
                data: Some(msg),
            }))
        }
        Err(err) => {
            state.audit.record(&operator.username, &operator.provider, "Tomcat", "save_config", &payload.file, "FAILED", &err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: err,
                    data: None,
                }),
            ))
        }
    }
}

async fn tomcat_connectors_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<TomcatConnectorInfo>>> {
    let conn = parse_connectors(&state.config.tomcat_conf_dir);
    Json(ApiResponse {
        success: true,
        message: format!("{} conectores Tomcat identificados", conn.len()),
        data: Some(conn),
    })
}

async fn tomcat_jvm_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
) -> Json<ApiResponse<JvmConfig>> {
    let cfg = read_jvm_config(&state.config.tomcat_conf_dir);
    Json(ApiResponse {
        success: true,
        message: "Configuração de JVM do Tomcat".to_string(),
        data: Some(cfg),
    })
}

async fn tomcat_logs_handler(
    _operator: AuthenticatedOperator,
    Query(query): Query<LogQuery>,
) -> Json<ApiResponse<Vec<String>>> {
    let limit = query.limit.unwrap_or(100);
    let logs = read_tomcat_logs(limit, query.level.as_deref());
    Json(ApiResponse {
        success: true,
        message: format!("{} linhas de log do Tomcat", logs.len()),
        data: Some(logs),
    })
}

async fn security_get_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
) -> Json<ApiResponse<SecuritySettings>> {
    let sec = get_security_settings(&state.config.nginx_conf_dir);
    Json(ApiResponse {
        success: true,
        message: "Políticas de segurança do Web Server".to_string(),
        data: Some(sec),
    })
}

async fn security_apply_handler(
    operator: AuthenticatedOperator,
    State(state): State<AppState>,
    Json(payload): Json<SecuritySettings>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match apply_security_settings(&state.config.nginx_conf_dir, &payload) {
        Ok(msg) => {
            state.audit.record(&operator.username, &operator.provider, "Security", "apply_security", "acl/headers", "SUCCESS", &msg);
            Ok(Json(ApiResponse {
                success: true,
                message: msg.clone(),
                data: Some(msg),
            }))
        }
        Err(err) => {
            state.audit.record(&operator.username, &operator.provider, "Security", "apply_security", "acl/headers", "FAILED", &err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: err,
                    data: None,
                }),
            ))
        }
    }
}

async fn monitoring_stats_handler(_operator: AuthenticatedOperator) -> Json<ApiResponse<LiveTelemetry>> {
    let snap = get_telemetry_snapshot();
    Json(ApiResponse {
        success: true,
        message: "Snapshot de telemetria".to_string(),
        data: Some(snap),
    })
}

async fn backups_list_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<crate::backup::BackupEntry>>> {
    let b = crate::backup::list_backups(&state.config.backup_dir);
    Json(ApiResponse {
        success: true,
        message: format!("{} backups de configuração disponíveis", b.len()),
        data: Some(b),
    })
}

#[derive(Debug, Deserialize)]
struct CreateBackupPayload {
    label: String,
}

async fn backups_create_handler(
    operator: AuthenticatedOperator,
    State(state): State<AppState>,
    Json(payload): Json<CreateBackupPayload>,
) -> Result<Json<ApiResponse<crate::backup::BackupEntry>>, (StatusCode, Json<ApiResponse<()>>)> {
    let nginx_conf = state.config.nginx_conf_dir.as_path();
    let tomcat_conf = state.config.tomcat_conf_dir.as_path();

    match crate::backup::create_snapshot(&state.config.backup_dir, &payload.label, &[nginx_conf, tomcat_conf]) {
        Ok(entry) => {
            state.audit.record(&operator.username, &operator.provider, "Backup", "create_snapshot", &entry.id, "SUCCESS", "Manual backup created");
            Ok(Json(ApiResponse {
                success: true,
                message: "Backup criado com sucesso!".to_string(),
                data: Some(entry),
            }))
        }
        Err(err) => {
            state.audit.record(&operator.username, &operator.provider, "Backup", "create_snapshot", &payload.label, "FAILED", &err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: err,
                    data: None,
                }),
            ))
        }
    }
}

async fn audit_list_handler(
    _operator: AuthenticatedOperator,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<crate::audit::AuditEvent>>> {
    let ev = state.audit.list(200);
    Json(ApiResponse {
        success: true,
        message: format!("{} eventos de auditoria", ev.len()),
        data: Some(ev),
    })
}
