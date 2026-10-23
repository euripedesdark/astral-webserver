use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::process::Command;

// ==============================================================================
// TIPOS E MODELOS DE AUTENTICAÇÃO
// ==============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    #[serde(default)]
    pub iss: String,
    pub sub: String,
    #[serde(default)]
    pub identityId: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    pub provider: String,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub has_wheel: bool,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedOperator {
    pub username: String,
    pub provider: String,
    pub is_admin: bool,
    pub groups: Vec<String>,
    pub has_wheel: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub provider: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub operator: AuthenticatedOperator,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

// ==============================================================================
// VERIFICAÇÃO DE AD CONFIGURADO
// ==============================================================================

pub fn is_ad_configured() -> bool {
    let smb_conf = Path::new("/etc/samba/smb.conf");
    if !smb_conf.exists() {
        return false;
    }
    if let Ok(content) = fs::read_to_string(smb_conf) {
        let lower = content.to_lowercase();
        if lower.contains("server role = active directory domain controller") {
            let active = Command::new("systemctl")
                .args(["is-active", "samba-ad-dc"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
                .unwrap_or(false);
            return active;
        }
    }
    false
}

// ==============================================================================
// AUTENTICAÇÃO REAL PAM (LINUX ROOT & WHEEL/SUDO DYNAMIC DLOPEN)
// ==============================================================================

#[repr(C)]
struct PamHandle {
    _unused: [u8; 0],
}

#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

#[repr(C)]
struct PamConv {
    conv: extern "C" fn(
        c_int,
        *mut *mut PamMessage,
        *mut *mut PamResponse,
        *mut c_void,
    ) -> c_int,
    appdata_ptr: *mut c_void,
}

const PAM_PROMPT_ECHO_OFF: c_int = 1;
const PAM_SUCCESS: c_int = 0;

extern "C" fn pam_conversation_cb(
    num_msg: c_int,
    msg: *mut *mut PamMessage,
    resp: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int {
    if num_msg <= 0 || appdata_ptr.is_null() {
        return 19; // PAM_CONV_ERR
    }

    let password_str = appdata_ptr as *const c_char;
    let responses = unsafe { libc::calloc(num_msg as usize, std::mem::size_of::<PamResponse>()) as *mut PamResponse };
    if responses.is_null() {
        return 19;
    }

    for i in 0..(num_msg as isize) {
        unsafe {
            let m = *msg.offset(i);
            if !m.is_null() && (*m).msg_style == PAM_PROMPT_ECHO_OFF {
                let r = responses.offset(i);
                (*r).resp = libc::strdup(password_str);
                (*r).resp_retcode = 0;
            }
        }
    }

    unsafe { *resp = responses };
    PAM_SUCCESS
}

type PamStartFn = unsafe extern "C" fn(
    service_name: *const c_char,
    user: *const c_char,
    pam_conversation: *const PamConv,
    pamh: *mut *mut PamHandle,
) -> c_int;

type PamAuthFn = unsafe extern "C" fn(pamh: *mut PamHandle, flags: c_int) -> c_int;
type PamAcctFn = unsafe extern "C" fn(pamh: *mut PamHandle, flags: c_int) -> c_int;
type PamEndFn = unsafe extern "C" fn(pamh: *mut PamHandle, pam_status: c_int) -> c_int;

pub fn authenticate_pam(username: &str, password: &str) -> Result<bool, String> {
    unsafe {
        let pam_lib_name = CString::new("libpam.so.0").unwrap();
        let mut handle = libc::dlopen(pam_lib_name.as_ptr(), libc::RTLD_NOW);
        if handle.is_null() {
            let alt_name = CString::new("libpam.so").unwrap();
            handle = libc::dlopen(alt_name.as_ptr(), libc::RTLD_NOW);
        }
        if handle.is_null() {
            return Err("Biblioteca PAM (libpam.so.0) não encontrada no sistema".to_string());
        }

        let pam_start_sym = libc::dlsym(handle, b"pam_start\0".as_ptr() as *const c_char);
        let pam_auth_sym = libc::dlsym(handle, b"pam_authenticate\0".as_ptr() as *const c_char);
        let pam_acct_sym = libc::dlsym(handle, b"pam_acct_mgmt\0".as_ptr() as *const c_char);
        let pam_end_sym = libc::dlsym(handle, b"pam_end\0".as_ptr() as *const c_char);

        if pam_start_sym.is_null() || pam_auth_sym.is_null() || pam_end_sym.is_null() {
            libc::dlclose(handle);
            return Err("Símbolos da API PAM não puderam ser vinculados".to_string());
        }

        let pam_start: PamStartFn = std::mem::transmute(pam_start_sym);
        let pam_auth: PamAuthFn = std::mem::transmute(pam_auth_sym);
        let pam_acct: PamAcctFn = if !pam_acct_sym.is_null() {
            std::mem::transmute(pam_acct_sym)
        } else {
            std::mem::transmute(pam_auth_sym)
        };
        let pam_end: PamEndFn = std::mem::transmute(pam_end_sym);

        let user_c = CString::new(username).map_err(|e| e.to_string())?;
        let pass_c = CString::new(password).map_err(|e| e.to_string())?;

        let services = ["system-auth", "login", "password-auth", "su"];
        let mut authenticated = false;

        for srv in services {
            let service_name = CString::new(srv).unwrap();
            let conv = PamConv {
                conv: pam_conversation_cb,
                appdata_ptr: pass_c.as_ptr() as *mut c_void,
            };

            let mut pamh: *mut PamHandle = std::ptr::null_mut();
            let start_res = pam_start(service_name.as_ptr(), user_c.as_ptr(), &conv, &mut pamh);
            if start_res == PAM_SUCCESS && !pamh.is_null() {
                let auth_res = pam_auth(pamh, 0);
                if auth_res == PAM_SUCCESS {
                    let acct_res = pam_acct(pamh, 0);
                    pam_end(pamh, PAM_SUCCESS);
                    if acct_res == PAM_SUCCESS || acct_res == 12 {
                        authenticated = true;
                        break;
                    }
                } else {
                    pam_end(pamh, auth_res);
                }
            }
        }

        libc::dlclose(handle);
        Ok(authenticated)
    }
}

pub fn authenticate_linux(username: &str, password: &str) -> Result<AuthenticatedOperator, String> {
    if username.is_empty() || password.is_empty() {
        return Err("Usuário e senha são obrigatórios".to_string());
    }

    let is_valid = authenticate_pam(username, password)
        .unwrap_or(false);

    if !is_valid {
        return Err("Falha na autenticação PAM: credenciais inválidas para o usuário Linux informado".to_string());
    }

    let groups_out = Command::new("id")
        .arg("-Gn")
        .arg(username)
        .output()
        .map_err(|e| format!("Erro ao obter grupos do usuário: {}", e))?;

    let groups_str = String::from_utf8_lossy(&groups_out.stdout);
    let groups: Vec<String> = groups_str.split_whitespace().map(|s| s.to_string()).collect();

    let is_root = username == "root";
    let in_wheel = groups.iter().any(|g| g == "wheel" || g == "sudo");

    if !is_root && !in_wheel {
        return Err("Acesso negado: o usuário Linux precisa pertencer ao grupo wheel ou sudo.".to_string());
    }

    Ok(AuthenticatedOperator {
        username: username.to_string(),
        provider: "Linux".to_string(),
        is_admin: true,
        groups,
        has_wheel: true,
    })
}

// ==============================================================================
// AUTENTICAÇÃO SAMBA AD / SSO
// ==============================================================================

pub fn authenticate_samba_ad(username: &str, password: &str) -> Result<AuthenticatedOperator, String> {
    let clean_user = username.replace(['\'', '"', ';', '&', '|'], "");

    let out = Command::new("samba-tool")
        .args(["user", "list"])
        .output()
        .map_err(|e| format!("Falha ao verificar banco do Samba AD: {}", e))?;

    if !out.status.success() {
        return Err("Controlador de domínio Samba AD não está respondendo".to_string());
    }

    let users_text = String::from_utf8_lossy(&out.stdout);
    let user_exists = users_text.lines().any(|l| l.trim().eq_ignore_ascii_case(&clean_user));
    if !user_exists {
        return Err(format!("Usuário '{}' não encontrado no Samba AD", clean_user));
    }

    let auth_check = Command::new("samba-tool")
        .args([
            "user", "getgroups", &clean_user,
            "-U", &format!("{}%{}", clean_user, password),
        ])
        .output()
        .map_err(|e| format!("Falha ao validar credenciais no Active Directory: {}", e))?;

    if !auth_check.status.success() {
        return Err("Senha incorreta para a conta do Active Directory".to_string());
    }

    let groups_text = String::from_utf8_lossy(&auth_check.stdout);
    let groups: Vec<String> = groups_text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    let is_admin = clean_user.eq_ignore_ascii_case("Administrator")
        || groups.iter().any(|g| {
            let lower = g.to_lowercase();
            lower.contains("domain admins")
                || lower.contains("administrators")
                || lower.contains("enterprise admins")
        });

    if !is_admin {
        return Err("Acesso negado: a conta AD necessita de privilégios Domain Admins.".to_string());
    }

    Ok(AuthenticatedOperator {
        username: clean_user,
        provider: "AD".to_string(),
        is_admin: true,
        groups,
        has_wheel: true,
    })
}

// ==============================================================================
// AUTENTICAÇÃO POSTGRESQL (ISOLADA A BANCO, SEM TERMINAL)
// ==============================================================================

pub fn authenticate_postgres(username: &str, password: &str) -> Result<AuthenticatedOperator, String> {
    let clean_user = username.replace(['\'', '"', ';', '&', '|'], "");
    let sql = format!(
        "SELECT rolsuper FROM pg_roles WHERE rolname = '{}' AND rolcanlogin = true;",
        clean_user
    );

    let output = Command::new("psql")
        .arg("-U")
        .arg(&clean_user)
        .arg("-h")
        .arg("127.0.0.1")
        .arg("-d")
        .arg("postgres")
        .arg("-t")
        .arg("-c")
        .arg(&sql)
        .env("PGPASSWORD", password)
        .output()
        .map_err(|e| format!("Falha ao comunicar com o banco de dados PostgreSQL: {}", e))?;

    if !output.status.success() {
        return Err("Credenciais inválidas para o banco de dados PostgreSQL.".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let is_super = stdout.trim() == "t";

    if !is_super {
        return Err("Acesso negado: o usuário informado não é Superusuário do PostgreSQL (rolsuper = true).".to_string());
    }

    Ok(AuthenticatedOperator {
        username: clean_user,
        provider: "Postgres".to_string(),
        is_admin: false,
        groups: vec!["postgres_superuser".to_string()],
        has_wheel: false,
    })
}

// ==============================================================================
// EMISSÃO E VERIFICAÇÃO DE TOKENS JWT
// ==============================================================================

pub fn issue_jwt(op: &AuthenticatedOperator, secret: &str) -> Result<String, String> {
    let now = Utc::now().timestamp();
    let exp = now + 8 * 3600; // 8 horas

    let claims = Claims {
        iss: "auth-service".to_string(),
        sub: op.username.clone(),
        identityId: Some(op.username.clone()),
        username: Some(op.username.clone()),
        provider: op.provider.clone(),
        groups: op.groups.clone(),
        is_admin: op.is_admin,
        has_wheel: op.has_wheel,
        iat: now,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Falha ao assinar JWT: {}", e))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.validate_aud = false;
    validation.required_spec_claims.remove("iss");

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => Err(format!("Token JWT inválido ou expirado: {}", e)),
    }
}

pub fn extract_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth_val) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let trimmed = token.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    if let Some(cookie_val) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_val.to_str() {
            for part in cookie_str.split(';') {
                let part = part.trim();
                for prefix in ["ASTRAL_SESSION=", "astral_token=", "sso_token="] {
                    if let Some(token) = part.strip_prefix(prefix) {
                        let trimmed = token.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

// ==============================================================================
// HANDLERS DE AUTENTICAÇÃO
// ==============================================================================

pub async fn login_handler(
    Json(payload): Json<LoginPayload>,
) -> Response {
    let secret = std::env::var("ASTRAL_AUTH_SECRET")
        .unwrap_or_else(|_| "astral-core-auth-secret-change-in-prod-2026".to_string());

    let provider = payload.provider.to_lowercase();
    let auth_result = match provider.as_str() {
        "linux" => authenticate_linux(&payload.username, &payload.password),
        "ad" => authenticate_samba_ad(&payload.username, &payload.password),
        "postgres" => authenticate_postgres(&payload.username, &payload.password),
        _ => {
            if let Ok(op) = authenticate_samba_ad(&payload.username, &payload.password) {
                Ok(op)
            } else if let Ok(op) = authenticate_linux(&payload.username, &payload.password) {
                Ok(op)
            } else {
                authenticate_postgres(&payload.username, &payload.password)
            }
        }
    };

    match auth_result {
        Ok(op) => {
            match issue_jwt(&op, &secret) {
                Ok(jwt) => {
                    let mut resp_headers = HeaderMap::new();
                    let cookie_str = format!(
                        "ASTRAL_SESSION={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=28800",
                        jwt
                    );
                    if let Ok(cookie_val) = HeaderValue::from_str(&cookie_str) {
                        resp_headers.insert(header::SET_COOKIE, cookie_val);
                    }

                    (
                        StatusCode::OK,
                        resp_headers,
                        Json(ApiResponse {
                            success: true,
                            message: "Autenticação realizada com sucesso".to_string(),
                            data: Some(LoginResponse {
                                token: jwt,
                                token_type: "Bearer".to_string(),
                                expires_in: 28800,
                                operator: op,
                            }),
                        }),
                    )
                        .into_response()
                }
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()> {
                        success: false,
                        message: err,
                        data: None,
                    }),
                )
                    .into_response(),
            }
        }
        Err(err) => (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()> {
                success: false,
                message: err,
                data: None,
            }),
        )
            .into_response(),
    }
}

pub async fn me_handler(operator: AuthenticatedOperator) -> Json<ApiResponse<AuthenticatedOperator>> {
    Json(ApiResponse {
        success: true,
        message: "Operador autenticado".to_string(),
        data: Some(operator),
    })
}

pub async fn logout_handler() -> Response {
    let mut resp_headers = HeaderMap::new();
    let clear_session = "ASTRAL_SESSION=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    let clear_sso = "sso_token=; Path=/; SameSite=Lax; Max-Age=0";
    let clear_astral = "astral_token=; Path=/; SameSite=Lax; Max-Age=0";
    if let Ok(v) = HeaderValue::from_str(clear_session) {
        resp_headers.append(header::SET_COOKIE, v);
    }
    if let Ok(v) = HeaderValue::from_str(clear_sso) {
        resp_headers.append(header::SET_COOKIE, v);
    }
    if let Ok(v) = HeaderValue::from_str(clear_astral) {
        resp_headers.append(header::SET_COOKIE, v);
    }

    (
        StatusCode::OK,
        resp_headers,
        Json(ApiResponse::<()> {
            success: true,
            message: "Sessão encerrada com sucesso".to_string(),
            data: None,
        }),
    )
        .into_response()
}

pub async fn providers_handler() -> Json<ApiResponse<serde_json::Value>> {
    let ad_configured = is_ad_configured();
    let recommended = if ad_configured { "ad" } else { "linux" };

    Json(ApiResponse {
        success: true,
        message: "Provedores de autenticação suportados".to_string(),
        data: Some(serde_json::json!({
            "ad_configured": ad_configured,
            "recommended": recommended,
            "providers": [
                {
                    "id": "ad",
                    "name": "Samba Active Directory / SSO",
                    "description": "Autenticação corporativa via Kerberos/LDAP ou token SSO da Astral Platform"
                },
                {
                    "id": "linux",
                    "name": "Usuário Local Linux (PAM Real)",
                    "description": "Autenticação segura via PAM para root ou usuários com privilégios wheel/sudo"
                },
                {
                    "id": "postgres",
                    "name": "PostgreSQL Superusuário",
                    "description": "Acesso de administração técnica de banco de dados (restrito ao escopo DB)"
                }
            ]
        })),
    })
}

// ==============================================================================
// AXUM EXTRACTOR PARA ROTAS PROTEGIDAS
// ==============================================================================

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedOperator
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ApiResponse<()>>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token_opt = extract_token(&parts.headers).or_else(|| {
            if let Some(query) = parts.uri.query() {
                for pair in query.split('&') {
                    if let Some((k, v)) = pair.split_once('=') {
                        if k == "token" || k == "sso_token" || k == "astral_token" {
                            return Some(v.to_string());
                        }
                    }
                }
            }
            None
        });

        let token = match token_opt {
            Some(t) => t,
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse {
                        success: false,
                        message: "Autenticação obrigatória. Forneça Bearer Token ou Cookie de sessão válido."
                            .to_string(),
                        data: None,
                    }),
                ));
            }
        };

        let secret = std::env::var("ASTRAL_AUTH_SECRET")
            .unwrap_or_else(|_| "astral-core-auth-secret-change-in-prod-2026".to_string());

        let claims = match verify_jwt(&token, &secret) {
            Ok(c) => c,
            Err(e) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Sessão expirada ou token inválido: {}", e),
                        data: None,
                    }),
                ));
            }
        };

        let username = claims.username.or_else(|| claims.identityId).unwrap_or(claims.sub);

        Ok(AuthenticatedOperator {
            username,
            provider: claims.provider,
            is_admin: claims.is_admin,
            groups: claims.groups,
            has_wheel: claims.has_wheel,
        })
    }
}
