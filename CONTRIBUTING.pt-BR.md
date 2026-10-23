# Contribuindo para o Astral Web Server

## Código de Conduta
Este projeto e todos os colaboradores são regidos pelo Código de Conduta. Espera-se respeito e colaboração técnica em todas as interações.

## Configuração de Desenvolvimento

### Pré-requisitos
* Rust 1.80+ (cargo, rustc)
* Node.js 20+ & npm 10+
* Nginx 1.24+
* Apache Tomcat 10.1+

### Compilação e Execução

```bash
# Backend Rust
cargo test
cargo build --release

# Frontend React
cd web
npm install
npm run build
```

## Estrutura do Projeto

```
astral-webserver/
├── Cargo.toml                 # Definição de dependências e metadados Rust
├── Cargo.lock                 # Versões exatas travadas de dependências
├── src/                       # Código-fonte do backend Axum
│   ├── main.rs                # Ponto de entrada, inicialização do servidor e SPA
│   ├── config.rs              # Gerenciador de portas, diretórios e segredos
│   ├── audit.rs               # Trilha de auditoria operacional em buffer circular
│   ├── auth.rs                # PAM real dinâmico (root/wheel/sudo) e Samba AD / JWT
│   ├── discovery.rs           # Descoberta de daemons Nginx, Tomcat e telemetria
│   ├── nginx.rs               # Controle de serviço, vhosts e sintaxe (nginx -t)
│   ├── tomcat.rs              # Controle de serviço, conectores, JVM e deploy WAR
│   ├── security.rs            # Central de segurança, CIDR IP e cabeçalhos HTTP
│   ├── monitoring.rs          # Telemetria de CPU/RAM/Disco e stream WebSocket
│   ├── backup.rs              # Snapshots SHA-256 e rollback de configurações
│   ├── terminal.rs            # Console operacional restrito a administradores
│   └── api.rs                 # Roteador REST unificado (/api/*)
├── web/                       # Frontend SPA React 18 + Vite + Tailwind CSS
│   ├── src/
│   │   ├── main.jsx           # Ponto de montagem React
│   │   ├── App.jsx            # Interface unificada com todas as sub-rotas (/infra/webserver/*)
│   │   ├── i18n.js            # Dicionário multilíngue completo (pt-BR, en-US, es-ES, fr-FR)
│   │   ├── index.css          # Estilos Tailwind e tema escuro
│   │   └── components/
│   │       └── DonationModal.jsx # Modal de doações PIX e Wise
│   ├── package.json           # Dependências npm
│   ├── vite.config.js         # Configuração do Vite (base: /webserver/)
│   └── dist/                  # Build de produção compilado
├── .github/workflows/         # Pipeline CI/CD GitHub Actions
└── docs/i18n/                 # Documentação multilíngue
```

## Diretrizes para Pull Requests
1. Crie uma branch descritiva a partir de main.
2. Escreva testes unitários para novas funcionalidades no backend.
3. Garanta que npm run build e cargo test passem sem erros.
4. Submeta o Pull Request com descrição detalhada das modificações.
