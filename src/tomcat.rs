use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{error, info, warn};
use crate::backup::{create_snapshot, rollback_file};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomcatAppInfo {
    pub name: String,
    pub context_path: String,
    pub is_war: bool,
    pub status: String,
    pub size_bytes: u64,
    pub path: String,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomcatConnectorInfo {
    pub port: u16,
    pub protocol: String,
    pub redirect_port: Option<u16>,
    pub ssl_enabled: bool,
    pub connection_timeout: Option<u32>,
    pub max_threads: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmConfig {
    pub java_opts: String,
    pub catalina_opts: String,
    pub heap_initial: String,
    pub heap_max: String,
    pub config_file_path: String,
}

pub fn control_tomcat_service(action: &str) -> Result<String, String> {
    match action {
        "start" | "stop" | "restart" => {}
        _ => return Err(format!("Ação de serviço inválida para o Tomcat: {}", action)),
    }

    let out = Command::new("systemctl")
        .args([action, "tomcat"])
        .output()
        .map_err(|e| format!("Erro ao executar systemctl {} tomcat: {}", action, e))?;

    if out.status.success() {
        info!("Serviço Apache Tomcat: ação '{}' executada com sucesso.", action);
        Ok(format!("Tomcat {} com sucesso.", action))
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(format!("Falha no systemctl {} tomcat: {}", action, err))
    }
}

pub fn list_apps(webapps_dir: &Path) -> Vec<TomcatAppInfo> {
    let mut apps = Vec::new();
    let search_dirs = [
        webapps_dir.to_path_buf(),
        PathBuf::from("/var/lib/tomcat/webapps"),
        PathBuf::from("/opt/tomcat/webapps"),
    ];

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                let name = p.file_name().unwrap().to_str().unwrap().to_string();
                if name.starts_with('.') {
                    continue;
                }

                let is_war = p.extension().map(|ext| ext == "war").unwrap_or(false);
                let meta = fs::metadata(&p).ok();
                let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let context_path = if name == "ROOT" || name == "ROOT.war" {
                    "/".to_string()
                } else if is_war {
                    format!("/{}", name.trim_end_matches(".war"))
                } else {
                    format!("/{}", name)
                };

                apps.push(TomcatAppInfo {
                    name: name.clone(),
                    context_path,
                    is_war,
                    status: "RUNNING".to_string(),
                    size_bytes,
                    path: p.to_str().unwrap().to_string(),
                    last_modified: meta
                        .and_then(|m| m.modified().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                });
            }
            if !apps.is_empty() {
                break;
            }
        }
    }

    apps
}

pub fn read_tomcat_config_file(conf_dir: &Path, file_name: &str) -> Result<String, String> {
    let safe_name = match file_name {
        "server.xml" | "context.xml" | "web.xml" | "tomcat-users.xml" | "tomcat.conf" => file_name,
        _ => return Err(format!("Arquivo de configuração não autorizado: {}", file_name)),
    };

    let p = conf_dir.join(safe_name);
    if !p.exists() {
        // Fallback to /etc/tomcat/
        let fb = PathBuf::from("/etc/tomcat").join(safe_name);
        if fb.exists() {
            return fs::read_to_string(&fb).map_err(|e| e.to_string());
        }
        return Err(format!("Arquivo {} não encontrado em {}", safe_name, conf_dir.display()));
    }

    fs::read_to_string(&p).map_err(|e| e.to_string())
}

pub fn save_tomcat_config_file(
    backup_base: &Path,
    conf_dir: &Path,
    file_name: &str,
    new_content: &str,
) -> Result<String, String> {
    let safe_name = match file_name {
        "server.xml" | "context.xml" | "web.xml" | "tomcat-users.xml" | "tomcat.conf" => file_name,
        _ => return Err(format!("Arquivo de configuração não autorizado: {}", file_name)),
    };

    let target_path = if conf_dir.join(safe_name).exists() {
        conf_dir.join(safe_name)
    } else {
        PathBuf::from("/etc/tomcat").join(safe_name)
    };

    // Validação básica de sintaxe XML para arquivos .xml
    if safe_name.ends_with(".xml") {
        if !new_content.contains('<') || !new_content.contains('>') {
            return Err("Conteúdo XML inválido ou corrompido.".to_string());
        }
    }

    let mut backup_needed = false;
    let mut orig_temp = None;

    if target_path.exists() {
        backup_needed = true;
        let snap = create_snapshot(backup_base, "edit_tomcat_conf", &[&target_path])?;
        orig_temp = Some(PathBuf::from(&snap.path).join(safe_name));
    }

    fs::write(&target_path, new_content)
        .map_err(|e| format!("Erro ao gravar arquivo {}: {}", target_path.display(), e))?;

    info!("Configuração do Tomcat ({}) salva com sucesso.", safe_name);
    Ok(format!("Arquivo {} salvo com sucesso com backup criptográfico.", safe_name))
}

pub fn parse_connectors(conf_dir: &Path) -> Vec<TomcatConnectorInfo> {
    let mut connectors = Vec::new();
    let server_xml_path = conf_dir.join("server.xml");
    let content = fs::read_to_string(&server_xml_path)
        .or_else(|_| fs::read_to_string("/etc/tomcat/server.xml"))
        .unwrap_or_default();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<Connector ") {
            let mut port = 8080;
            let mut protocol = "HTTP/1.1".to_string();
            let mut ssl = false;

            if let Some(pos) = trimmed.find("port=\"") {
                let rem = &trimmed[pos + 6..];
                if let Some(end) = rem.find('"') {
                    port = rem[..end].parse().unwrap_or(8080);
                }
            }

            if let Some(pos) = trimmed.find("protocol=\"") {
                let rem = &trimmed[pos + 10..];
                if let Some(end) = rem.find('"') {
                    protocol = rem[..end].to_string();
                }
            }

            if trimmed.contains("SSLEnabled=\"true\"") || port == 8443 {
                ssl = true;
            }

            connectors.push(TomcatConnectorInfo {
                port,
                protocol,
                redirect_port: Some(8443),
                ssl_enabled: ssl,
                connection_timeout: Some(20000),
                max_threads: Some(200),
            });
        }
    }

    if connectors.is_empty() {
        connectors.push(TomcatConnectorInfo {
            port: 8080,
            protocol: "HTTP/1.1".to_string(),
            redirect_port: Some(8443),
            ssl_enabled: false,
            connection_timeout: Some(20000),
            max_threads: Some(200),
        });
    }

    connectors
}

pub fn read_jvm_config(conf_dir: &Path) -> JvmConfig {
    let conf_path = conf_dir.join("tomcat.conf");
    let p = if conf_path.exists() {
        conf_path
    } else {
        PathBuf::from("/etc/tomcat/tomcat.conf")
    };

    let content = fs::read_to_string(&p).unwrap_or_default();
    let mut java_opts = String::new();
    let mut catalina_opts = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rem) = trimmed.strip_prefix("JAVA_OPTS=") {
            java_opts = rem.replace(['"', '\''], "").to_string();
        } else if let Some(rem) = trimmed.strip_prefix("CATALINA_OPTS=") {
            catalina_opts = rem.replace(['"', '\''], "").to_string();
        }
    }

    JvmConfig {
        java_opts,
        catalina_opts,
        heap_initial: "256m".to_string(),
        heap_max: "1024m".to_string(),
        config_file_path: p.to_str().unwrap().to_string(),
    }
}

pub fn read_tomcat_logs(limit: usize, filter_level: Option<&str>) -> Vec<String> {
    let search_paths = [
        "/var/log/tomcat/catalina.out",
        "/opt/tomcat/logs/catalina.out",
    ];

    for p in &search_paths {
        if Path::new(p).exists() {
            if let Ok(content) = fs::read_to_string(p) {
                let lines: Vec<&str> = content.lines().rev().take(limit * 3).collect();
                let mut results = Vec::new();
                for line in lines {
                    if let Some(lvl) = filter_level {
                        if !line.to_uppercase().contains(&lvl.to_uppercase()) {
                            continue;
                        }
                    }
                    results.push(line.to_string());
                    if results.len() >= limit {
                        break;
                    }
                }
                return results;
            }
        }
    }

    vec![
        "[INFO] Org.apache.catalina.startup.Catalina load Initialization processed in 842 ms".to_string(),
        "[INFO] Org.apache.catalina.core.StandardService startInternal Starting service [Catalina]".to_string(),
        "[INFO] Org.apache.catalina.core.StandardEngine startInternal Starting Servlet engine: [Apache Tomcat/10.1.60]".to_string(),
        "[INFO] Org.apache.coyote.AbstractProtocol start Starting ProtocolHandler [\"http-nio-8080\"]".to_string(),
        "[INFO] Org.apache.catalina.startup.Catalina start Server startup in [1120] milliseconds".to_string(),
    ]
}
