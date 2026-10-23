# Contribuyendo a Astral Web Server

## Código de Conducta
Este proyecto y sus participantes se rigen por el Código de Conducta.

## Configuración de Desarrollo

### Requisitos Previos
* Rust 1.80+ (cargo, rustc)
* Node.js 20+ & npm 10+
* Nginx 1.24+
* Apache Tomcat 10.1+

### Compilación y Ejecución

```bash
# Backend Rust
cargo test
cargo build --release

# Frontend React
cd web
npm install
npm run build
```

## Estructura del Proyecto

```
astral-webserver/
├── Cargo.toml                 # Dependencias y metadatos de Rust
├── Cargo.lock                 # Versiones bloqueadas de dependencias
├── src/                       # Código fuente backend Axum
│   ├── main.rs                # Punto de entrada, servidor HTTP y SPA
│   ├── config.rs              # Configuración de puertos, rutas y secretos
│   ├── audit.rs               # Registro de auditoría operativa
│   ├── auth.rs                # PAM real dinámico (root/wheel/sudo) y Samba AD / JWT
│   ├── discovery.rs           # Detección de demonios Nginx, Tomcat y telemetría
│   ├── nginx.rs               # Control de servicio, vhosts y validación (nginx -t)
│   ├── tomcat.rs              # Control de servicio, conectores, JVM y despliegue WAR
│   ├── security.rs            # Centro de seguridad, listas CIDR y cabeceras HTTP
│   ├── monitoring.rs          # Telemetría en vivo y flujo WebSocket
│   ├── backup.rs              # Snapshots SHA-256 y rollback de configuraciones
│   ├── terminal.rs            # Consola operativa administrativa
│   └── api.rs                 # Enrutador REST unificado (/api/*)
├── web/                       # Frontend SPA React 18 + Vite + Tailwind CSS
│   ├── src/
│   │   ├── main.jsx           # Montaje React
│   │   ├── App.jsx            # Interfaz SPA unificada (/infra/webserver/*)
│   │   ├── i18n.js            # Diccionario multilenguaje (pt-BR, en-US, es-ES, fr-FR)
│   │   ├── index.css          # Estilos Tailwind y tema oscuro
│   │   └── components/
│   │       └── DonationModal.jsx # Modal de donaciones PIX y Wise
│   ├── package.json           # Dependencias npm
│   ├── vite.config.js         # Configuración Vite (base: /webserver/)
│   └── dist/                  # Compilación estática de producción
├── .github/workflows/         # Flujos CI/CD de GitHub Actions
└── docs/i18n/                 # Documentación multilenguaje
```

## Directrices para Pull Requests
1. Cree una rama descriptiva a partir de main.
2. Añada pruebas unitarias para nuevas funcionalidades.
3. Compruebe que npm run build y cargo test se ejecuten sin errores.
4. Envíe el Pull Request con documentación de cambios.
