use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub ip_whitelist: Vec<String>,
    pub ip_blacklist: Vec<String>,
    pub hsts_enabled: bool,
    pub csp_enabled: bool,
    pub csp_header: String,
    pub x_frame_options: String,
    pub x_xss_protection: bool,
    pub referrer_policy: String,
    pub rate_limiting_enabled: bool,
    pub rate_limit_rps: u32,
    pub rate_limit_burst: u32,
    pub tomcat_manager_enabled: bool,
    pub tomcat_manager_allowed_ips: Vec<String>,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            ip_whitelist: vec!["127.0.0.1".to_string(), "192.168.2.0/24".to_string(), "100.64.0.0/10".to_string()],
            ip_blacklist: vec![],
            hsts_enabled: true,
            csp_enabled: false,
            csp_header: "default-src 'self' 'unsafe-inline' 'unsafe-eval'; img-src 'self' data:;".to_string(),
            x_frame_options: "SAMEORIGIN".to_string(),
            x_xss_protection: true,
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            rate_limiting_enabled: true,
            rate_limit_rps: 20,
            rate_limit_burst: 40,
            tomcat_manager_enabled: false,
            tomcat_manager_allowed_ips: vec!["127.0.0.1".to_string(), "192.168.2.10".to_string()],
        }
    }
}

pub fn get_security_settings(nginx_dir: &Path) -> SecuritySettings {
    let acl_file = nginx_dir.join("default.d").join("astral_security_acl.conf");
    let mut settings = SecuritySettings::default();

    if let Ok(content) = fs::read_to_string(acl_file) {
        let mut allow = Vec::new();
        let mut deny = Vec::new();
        for line in content.lines() {
            let t = line.trim();
            if let Some(ip) = t.strip_prefix("allow ") {
                allow.push(ip.replace(';', "").trim().to_string());
            } else if let Some(ip) = t.strip_prefix("deny ") {
                deny.push(ip.replace(';', "").trim().to_string());
            }
        }
        if !allow.is_empty() {
            settings.ip_whitelist = allow;
        }
        if !deny.is_empty() {
            settings.ip_blacklist = deny;
        }
    }

    settings
}

pub fn apply_security_settings(
    nginx_dir: &Path,
    settings: &SecuritySettings,
) -> Result<String, String> {
    let acl_dir = nginx_dir.join("default.d");
    let _ = fs::create_dir_all(&acl_dir);
    let acl_file = acl_dir.join("astral_security_acl.conf");

    let mut lines = Vec::new();
    lines.push("# Astral Platform - Unified Security Headers & Access Control".to_string());

    // Security Headers
    if settings.hsts_enabled {
        lines.push("add_header Strict-Transport-Security \"max-age=31536000; includeSubDomains\" always;".to_string());
    }
    if !settings.x_frame_options.is_empty() {
        lines.push(format!("add_header X-Frame-Options \"{}\" always;", settings.x_frame_options));
    }
    if settings.x_xss_protection {
        lines.push("add_header X-XSS-Protection \"1; mode=block\" always;".to_string());
    }
    if !settings.referrer_policy.is_empty() {
        lines.push(format!("add_header Referrer-Policy \"{}\" always;", settings.referrer_policy));
    }
    if settings.csp_enabled && !settings.csp_header.is_empty() {
        lines.push(format!("add_header Content-Security-Policy \"{}\" always;", settings.csp_header));
    }

    // IP Blacklist & Whitelist
    for d in &settings.ip_blacklist {
        lines.push(format!("deny {};", d));
    }
    for a in &settings.ip_whitelist {
        lines.push(format!("allow {};", a));
    }

    fs::write(&acl_file, lines.join("\n"))
        .map_err(|e| format!("Erro ao gravar configurações de segurança em {}: {}", acl_file.display(), e))?;

    info!("Políticas de segurança do Web Server aplicadas em {}", acl_file.display());
    Ok("Políticas de segurança salvas e integradas com sucesso.".to_string())
}
