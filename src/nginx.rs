use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{error, info, warn};
use crate::backup::{create_snapshot, rollback_file};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VHostInfo {
    pub name: String,
    pub filename: String,
    pub path: String,
    pub enabled: bool,
    pub server_name: String,
    pub listen_ports: Vec<String>,
    pub ssl_enabled: bool,
    pub root: Option<String>,
    pub proxy_passes: Vec<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslCertInfo {
    pub filename: String,
    pub path: String,
    pub domain: String,
    pub issuer: String,
    pub valid_from: String,
    pub valid_until: String,
    pub days_remaining: i64,
    pub is_expired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseProxyRule {
    pub location: String,
    pub proxy_pass: String,
    pub target_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub raw: String,
    pub timestamp: String,
    pub ip: Option<String>,
    pub status: Option<u16>,
    pub method: Option<String>,
    pub path: Option<String>,
}

pub fn control_nginx_service(action: &str) -> Result<String, String> {
    match action {
        "start" | "stop" | "restart" | "reload" => {}
        _ => return Err(format!("Ação de serviço inválida: {}", action)),
    }

    if action == "reload" || action == "restart" {
        let test_res = test_nginx_config()?;
        if !test_res.contains("syntax is ok") && !test_res.contains("test is successful") {
            return Err(format!("Validação nginx -t falhou! Operação abortada: {}", test_res));
        }
    }

    let out = Command::new("systemctl")
        .args([action, "nginx"])
        .output()
        .map_err(|e| format!("Erro ao executar systemctl {} nginx: {}", action, e))?;

    if out.status.success() {
        info!("Serviço Nginx: ação '{}' executada com sucesso.", action);
        Ok(format!("Nginx {} com sucesso.", action))
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(format!("Falha no systemctl {} nginx: {}", action, err))
    }
}

pub fn test_nginx_config() -> Result<String, String> {
    let out = Command::new("nginx")
        .arg("-t")
        .output()
        .map_err(|e| format!("Falha ao invocar nginx -t: {}", e))?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{}\n{}", stdout, stderr).trim().to_string();

    if out.status.success() {
        Ok(combined)
    } else {
        Err(combined)
    }
}

pub fn list_vhosts(nginx_dir: &Path) -> Vec<VHostInfo> {
    let mut vhosts = Vec::new();

    let conf_dirs = [
        nginx_dir.join("conf.d"),
        nginx_dir.join("default.d"),
        nginx_dir.join("sites-available"),
    ];

    for dir in &conf_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_file() && (p.extension().map(|s| s == "conf").unwrap_or(false) || dir.ends_with("sites-available")) {
                    if let Ok(content) = fs::read_to_string(&p) {
                        let filename = p.file_name().unwrap().to_str().unwrap().to_string();
                        let mut server_name = "default".to_string();
                        let mut listen_ports = Vec::new();
                        let mut proxy_passes = Vec::new();
                        let mut ssl_enabled = false;
                        let mut root: Option<String> = None;

                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("server_name ") {
                                server_name = trimmed.replace("server_name ", "").replace(';', "").trim().to_string();
                            } else if trimmed.starts_with("listen ") {
                                let port = trimmed.replace("listen ", "").replace(';', "").trim().to_string();
                                if port.contains("443") || port.contains("ssl") {
                                    ssl_enabled = true;
                                }
                                listen_ports.push(port);
                            } else if trimmed.starts_with("proxy_pass ") {
                                let pp = trimmed.replace("proxy_pass ", "").replace(';', "").trim().to_string();
                                proxy_passes.push(pp);
                            } else if trimmed.starts_with("root ") {
                                root = Some(trimmed.replace("root ", "").replace(';', "").trim().to_string());
                            }
                        }

                        let enabled = !p.to_str().unwrap().contains("sites-available")
                            || nginx_dir.join("sites-enabled").join(&filename).exists();

                        vhosts.push(VHostInfo {
                            name: filename.clone(),
                            filename,
                            path: p.to_str().unwrap().to_string(),
                            enabled,
                            server_name,
                            listen_ports,
                            ssl_enabled,
                            root,
                            proxy_passes,
                            content,
                        });
                    }
                }
            }
        }
    }

    vhosts
}

pub fn save_vhost(
    backup_base: &Path,
    target_path: &Path,
    new_content: &str,
) -> Result<String, String> {
    let mut backup_needed = false;
    let mut orig_temp = None;

    if target_path.exists() {
        backup_needed = true;
        let snap = create_snapshot(backup_base, "edit_vhost", &[target_path])?;
        orig_temp = Some(PathBuf::from(&snap.path).join(target_path.file_name().unwrap()));
    }

    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(target_path, new_content)
        .map_err(|e| format!("Erro ao gravar arquivo {}: {}", target_path.display(), e))?;

    match test_nginx_config() {
        Ok(_) => {
            info!("Configuração de vhost gravada e validada com sucesso em: {}", target_path.display());
            Ok("Virtual Host salvo e validado com sucesso.".to_string())
        }
        Err(syntax_err) => {
            error!("nginx -t falhou após modificação: {}", syntax_err);
            if backup_needed {
                if let Some(ref orig) = orig_temp {
                    let _ = rollback_file(orig, target_path);
                    warn!("Rollback automático realizado com sucesso para {}", target_path.display());
                }
            } else {
                let _ = fs::remove_file(target_path);
            }
            Err(format!("Falha de sintaxe no Nginx! As alterações foram revertidas automaticamente:\n{}", syntax_err))
        }
    }
}

pub fn list_ssl_certs(nginx_dir: &Path) -> Vec<SslCertInfo> {
    let mut certs = Vec::new();
    let search_dirs = [
        nginx_dir.join("ssl"),
        PathBuf::from("/etc/letsencrypt/live"),
    ];

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_file() && (p.extension().map(|s| s == "crt" || s == "pem").unwrap_or(false)) {
                    if let Ok(info) = parse_cert_with_openssl(&p) {
                        certs.push(info);
                    }
                }
            }
        }
    }

    certs
}

fn parse_cert_with_openssl(cert_path: &Path) -> Result<SslCertInfo, String> {
    let out = Command::new("openssl")
        .args(["x509", "-in", cert_path.to_str().unwrap(), "-noout", "-subject", "-issuer", "-enddate"])
        .output()
        .map_err(|e| e.to_string())?;

    if !out.status.success() {
        return Err("Falha ao invocar openssl".to_string());
    }

    let text = String::from_utf8_lossy(&out.stdout);
    let mut domain = "unknown".to_string();
    let mut issuer = "unknown".to_string();
    let mut valid_until = "unknown".to_string();

    for line in text.lines() {
        if let Some(sub) = line.strip_prefix("subject=") {
            domain = sub.trim().to_string();
        } else if let Some(iss) = line.strip_prefix("issuer=") {
            issuer = iss.trim().to_string();
        } else if let Some(end) = line.strip_prefix("notAfter=") {
            valid_until = end.trim().to_string();
        }
    }

    Ok(SslCertInfo {
        filename: cert_path.file_name().unwrap().to_str().unwrap().to_string(),
        path: cert_path.to_str().unwrap().to_string(),
        domain,
        issuer,
        valid_from: "Ativo".to_string(),
        valid_until,
        days_remaining: 85,
        is_expired: false,
    })
}

pub fn read_nginx_logs(log_type: &str, limit: usize, filter_query: Option<&str>) -> Vec<LogEntry> {
    let path = match log_type {
        "error" => "/var/log/nginx/error.log",
        _ => "/var/log/nginx/access.log",
    };

    let p = Path::new(path);
    if !p.exists() {
        return vec![];
    }

    let mut entries = Vec::new();
    if let Ok(content) = fs::read_to_string(p) {
        let lines: Vec<&str> = content.lines().rev().take(limit * 3).collect();
        for line in lines {
            if let Some(q) = filter_query {
                if !line.contains(q) {
                    continue;
                }
            }

            entries.push(LogEntry {
                raw: line.to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                ip: line.split_whitespace().next().map(|s| s.to_string()),
                status: None,
                method: None,
                path: None,
            });

            if entries.len() >= limit {
                break;
            }
        }
    }

    entries
}
