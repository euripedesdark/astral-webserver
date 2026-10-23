mod api;
mod audit;
mod auth;
mod backup;
mod config;
mod discovery;
mod monitoring;
mod nginx;
mod security;
mod terminal;
mod tomcat;

use api::{create_router, AppState};
use audit::AuditTrail;
use config::AppConfig;
use std::net::SocketAddr;
use std::path::Path;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("astral_webserver=info".parse().unwrap())
                .add_directive("tower_http=info".parse().unwrap()),
        )
        .init();

    info!("Inicializando Astral Web Server (Unified Nginx & Apache Tomcat Admin)...");

    let config = AppConfig::from_env();
    info!("Target Listen Address: {}", config.bind_addr);
    info!("Web Dist Directory: {}", config.web_dist.display());
    info!("Nginx Config Directory: {}", config.nginx_conf_dir.display());
    info!("Tomcat Config Directory: {}", config.tomcat_conf_dir.display());

    let audit = AuditTrail::default();
    audit.record("SYSTEM", "Daemon", "Daemon", "startup", "0.0.0.0:8096", "SUCCESS", "Astral Web Server inicializado com sucesso");

    // Descoberta inicial na inicialização
    let discovery = discovery::get_full_discovery();
    info!(
        "Descoberta de Serviços: Nginx=[installed={}, status={}], Tomcat=[installed={}, status={}]",
        discovery.nginx.installed, discovery.nginx.status,
        discovery.tomcat.installed, discovery.tomcat.status
    );

    let state = AppState {
        config: config.clone(),
        audit: audit.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_router = create_router(state);

    let index_file = config.web_dist.join("index.html");
    let serve_dir = ServeDir::new(&config.web_dist)
        .not_found_service(ServeFile::new(&index_file));

    let app = api_router
        .fallback_service(serve_dir)
        .layer(cors);

    let addr: SocketAddr = config.bind_addr.parse().expect("Endereço ASTRAL_BIND_ADDR inválido");
    info!("Astral Web Server está operacional e escutando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.expect("Falha ao abrir socket TCP");
    axum::serve(listener, app).await.expect("Erro na execução do servidor HTTP");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_issuance_and_verification() {
        let op = auth::AuthenticatedOperator {
            username: "admin_test".to_string(),
            provider: "AD".to_string(),
            is_admin: true,
            groups: vec!["Domain Admins".to_string()],
            has_wheel: true,
        };
        let secret = "test_secret_key_12345678901234567890";
        let token = auth::issue_jwt(&op, secret).expect("Deveria emitir token");
        let claims = auth::verify_jwt(&token, secret).expect("Deveria validar token");

        assert_eq!(claims.sub, "admin_test");
        assert_eq!(claims.provider, "AD");
        assert!(claims.is_admin);
        assert!(claims.has_wheel);
    }

    #[test]
    fn test_audit_recording() {
        let audit = AuditTrail::new(10);
        audit.record("tester", "Linux", "Nginx", "reload", "service", "SUCCESS", "Reload test");
        let list = audit.list(5);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].operator, "tester");
        assert_eq!(list[0].action, "reload");
    }

    #[test]
    fn test_discovery_system_metrics() {
        let sys = discovery::get_system_metrics();
        assert!(!sys.hostname.is_empty());
        assert!(sys.memory_total_bytes > 0);
    }
}
