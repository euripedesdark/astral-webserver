use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub id: String,
    pub timestamp: String,
    pub description: String,
    pub path: String,
    pub files: Vec<String>,
    pub sha256sums: Vec<String>,
}

pub fn compute_file_sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Falha ao ler arquivo: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

pub fn create_snapshot(
    backup_base: &Path,
    action_label: &str,
    paths_to_backup: &[&Path],
) -> Result<BackupEntry, String> {
    let now = Utc::now();
    let id = format!("{}_{}", now.format("%Y%m%d_%H%M%S"), action_label);
    let target_dir = backup_base.join(&id);

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Falha ao criar diretório de backup: {}", e))?;

    let mut saved_files = Vec::new();
    let mut sha256_lines = Vec::new();

    for p in paths_to_backup {
        if !p.exists() {
            continue;
        }

        let rel_name = p.file_name().unwrap().to_str().unwrap();
        let dest = target_dir.join(rel_name);

        if p.is_file() {
            fs::copy(p, &dest).map_err(|e| format!("Erro ao copiar arquivo {}: {}", p.display(), e))?;
            let hash = compute_file_sha256(&dest)?;
            sha256_lines.push(format!("{}  {}", hash, rel_name));
            saved_files.push(rel_name.to_string());
        } else if p.is_dir() {
            // Cria tarball gz do diretório
            let tar_dest = target_dir.join(format!("{}.tar.gz", rel_name));
            let status = Command::new("tar")
                .args(["-czf", tar_dest.to_str().unwrap(), "-C", p.parent().unwrap().to_str().unwrap(), rel_name])
                .output()
                .map_err(|e| format!("Erro ao criar tar do diretório {}: {}", p.display(), e))?;

            if status.status.success() {
                let hash = compute_file_sha256(&tar_dest)?;
                let tar_name = format!("{}.tar.gz", rel_name);
                sha256_lines.push(format!("{}  {}", hash, tar_name));
                saved_files.push(tar_name);
            }
        }
    }

    let sums_file = target_dir.join("SHA256SUMS");
    fs::write(&sums_file, sha256_lines.join("\n"))
        .map_err(|e| format!("Erro ao escrever SHA256SUMS: {}", e))?;

    let entry = BackupEntry {
        id,
        timestamp: now.to_rfc3339(),
        description: action_label.to_string(),
        path: target_dir.to_str().unwrap().to_string(),
        files: saved_files,
        sha256sums: sha256_lines,
    };

    info!("Snapshot de segurança criado com sucesso em: {}", entry.path);
    Ok(entry)
}

pub fn list_backups(backup_base: &Path) -> Vec<BackupEntry> {
    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(backup_base) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let sums_path = p.join("SHA256SUMS");
                let mut files = Vec::new();
                let mut sha256sums = Vec::new();

                if let Ok(content) = fs::read_to_string(sums_path) {
                    for line in content.lines() {
                        sha256sums.push(line.to_string());
                        if let Some((_, name)) = line.split_once("  ") {
                            files.push(name.to_string());
                        }
                    }
                }

                results.push(BackupEntry {
                    id: p.file_name().unwrap().to_str().unwrap().to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                    description: p.file_name().unwrap().to_str().unwrap().to_string(),
                    path: p.to_str().unwrap().to_string(),
                    files,
                    sha256sums,
                });
            }
        }
    }
    results.sort_by(|a, b| b.id.cmp(&a.id));
    results
}

pub fn rollback_file(backup_file: &Path, original_file: &Path) -> Result<(), String> {
    if !backup_file.exists() {
        return Err(format!("Arquivo de backup não encontrado: {}", backup_file.display()));
    }
    fs::copy(backup_file, original_file)
        .map_err(|e| format!("Erro durante o rollback para {}: {}", original_file.display(), e))?;
    info!("Rollback executado com sucesso: {} restaurado a partir de {}", original_file.display(), backup_file.display());
    Ok(())
}
