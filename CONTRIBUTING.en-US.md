# Contributing to Astral Web Server

## Code of Conduct
This project and all contributors are governed by the Code of Conduct. Respectful and productive collaboration is required.

## Development Setup

### Prerequisites
* Rust 1.80+ (cargo, rustc)
* Node.js 20+ & npm 10+
* Nginx 1.24+
* Apache Tomcat 10.1+

### Build and Execution

```bash
# Backend Rust
cargo test
cargo build --release

# Frontend React
cd web
npm install
npm run build
```

## Project Structure

```
astral-webserver/
├── Cargo.toml                 # Rust dependencies and project metadata
├── Cargo.lock                 # Exact locked dependency versions
├── src/                       # Axum backend source code
│   ├── main.rs                # Entrypoint, server initialization and static SPA server
│   ├── config.rs              # Port, directory and secret configurations
│   ├── audit.rs               # Operational audit trail in circular buffer
│   ├── auth.rs                # Dynamic real PAM (root/wheel/sudo) & Samba AD / JWT
│   ├── discovery.rs           # Daemon discovery for Nginx, Tomcat & telemetry
│   ├── nginx.rs               # Service control, vhosts & syntax validation (nginx -t)
│   ├── tomcat.rs              # Service control, connectors, JVM & WAR deployment
│   ├── security.rs            # Security center, IP CIDR & HTTP security headers
│   ├── monitoring.rs          # CPU/RAM/Disk telemetry & live WebSocket stream
│   ├── backup.rs              # SHA-256 snapshots & configuration rollback
│   ├── terminal.rs            # Operational administration console
│   └── api.rs                 # Unified REST router (/api/*)
├── web/                       # React 18 SPA Frontend + Vite + Tailwind CSS
│   ├── src/
│   │   ├── main.jsx           # React mount entrypoint
│   │   ├── App.jsx            # Unified SPA with all subroutes (/infra/webserver/*)
│   │   ├── i18n.js            # Complete dictionary (pt-BR, en-US, es-ES, fr-FR)
│   │   ├── index.css          # Tailwind CSS styles and dark theme
│   │   └── components/
│   │       └── DonationModal.jsx # PIX & Wise donation modal
│   ├── package.json           # npm dependencies
│   ├── vite.config.js         # Vite configuration (base: /webserver/)
│   └── dist/                  # Compiled production build
├── .github/workflows/         # CI/CD GitHub Actions pipeline
└── docs/i18n/                 # Multilingual documentation
```

## Pull Request Guidelines
1. Create a descriptive feature branch from main.
2. Write unit tests for new backend capabilities.
3. Ensure both npm run build and cargo test complete cleanly.
4. Submit a Pull Request with a clear summary and motivation.
