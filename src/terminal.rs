use axum::{
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use crate::auth::{ApiResponse, AuthenticatedOperator};

#[derive(Debug, Deserialize)]
pub struct TerminalExecRequest {
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct TerminalExecResponse {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct TerminalStatusResponse {
    pub allowed: bool,
    pub username: String,
    pub provider: String,
    pub message: String,
}

pub async fn get_terminal_status(
    operator: AuthenticatedOperator,
) -> Result<Json<TerminalStatusResponse>, (StatusCode, Json<TerminalStatusResponse>)> {
    if !operator.has_wheel && !operator.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(TerminalStatusResponse {
                allowed: false,
                username: operator.username,
                provider: operator.provider,
                message: "Acesso negado: o terminal do sistema operacional é restrito a administradores Samba AD ou Linux Sudo. Superusuários PostgreSQL não possuem permissão de shell.".to_string(),
            }),
        ));
    }

    Ok(Json(TerminalStatusResponse {
        allowed: true,
        username: operator.username,
        provider: operator.provider,
        message: "Terminal operacional autorizado.".to_string(),
    }))
}

pub async fn exec_terminal_cmd(
    operator: AuthenticatedOperator,
    Json(payload): Json<TerminalExecRequest>,
) -> Result<Json<ApiResponse<TerminalExecResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    if !operator.has_wheel && !operator.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Permissão insuficiente para invocar shell no servidor.".to_string(),
                data: None,
            }),
        ));
    }

    let cmd = payload.command.trim();
    if cmd.is_empty() {
        return Ok(Json(ApiResponse {
            success: true,
            message: "Comando vazio ignorado".to_string(),
            data: Some(TerminalExecResponse {
                command: String::new(),
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            }),
        }));
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let exit_code = out.status.code().unwrap_or(-1);

            Ok(Json(ApiResponse {
                success: true,
                message: "Comando executado".to_string(),
                data: Some(TerminalExecResponse {
                    command: cmd.to_string(),
                    stdout,
                    stderr,
                    exit_code,
                    success: out.status.success(),
                }),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Falha ao invocar processo: {}", e),
                data: None,
            }),
        )),
    }
}
