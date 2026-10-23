use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: String,
    pub auth_secret: String,
    pub sudo_pass: Option<String>,
    pub web_dist: PathBuf,
    pub backup_dir: PathBuf,
    pub nginx_conf_dir: PathBuf,
    pub tomcat_conf_dir: PathBuf,
    pub tomcat_webapps_dir: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let bind_addr = env::var("ASTRAL_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8096".to_string());
        let auth_secret = env::var("ASTRAL_AUTH_SECRET")
            .unwrap_or_else(|_| "astral-core-auth-secret-change-in-prod-2026".to_string());
        let sudo_pass = env::var("ASTRAL_SUDO_PASS").ok();
        let web_dist = env::var("ASTRAL_WEB_DIST")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/home/euripedes/astral/astral-webserver/web/dist"));
        let backup_dir = env::var("ASTRAL_BACKUP_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/backups/astral-webserver"));
        let nginx_conf_dir = env::var("ASTRAL_NGINX_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/etc/nginx"));
        let tomcat_conf_dir = env::var("ASTRAL_TOMCAT_CONF_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/etc/tomcat"));
        let tomcat_webapps_dir = env::var("ASTRAL_TOMCAT_WEBAPPS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/lib/tomcat/webapps"));

        Self {
            bind_addr,
            auth_secret,
            sudo_pass,
            web_dist,
            backup_dir,
            nginx_conf_dir,
            tomcat_conf_dir,
            tomcat_webapps_dir,
        }
    }
}
