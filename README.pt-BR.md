# Astral Web Server

**Astral Web Server** é o módulo centralizado de gerenciamento de infraestrutura web da **Astral Platform**, unificando a administração do **Nginx** e do **Apache Tomcat** em uma única interface full stack desenvolvida em **Rust** (backend com Axum) e **React + PrimeReact + Tailwind CSS** (frontend moderno em Vite).

---

## Funcionalidades Principais

1. **Descoberta Automática de Infraestrutura**:
   - Detecção instantânea de servidores web instalados (Nginx e Apache Tomcat).
   - Identificação de versões, status systemd, PIDs ativos, portas em escuta, consumo de CPU/RAM e uptime.

2. **Dashboard Unificado**:
   - Métricas em tempo real de Nginx (workers, conexões, requests/s, virtual hosts).
   - Métricas de Tomcat (aplicações implantadas, sessões, threads, heap JVM, garbage collector).
   - Telemetria de hardware (CPU, Memória, Disco e Rede).

3. **Módulo Nginx**:
   - Controle de serviços: Iniciar, Parar, Reiniciar e Recarregar (`reload`).
   - Editor de configuração com validação prévia de sintaxe via `nginx -t` antes de salvar.
   - Gestão completa de Virtual Hosts (`conf.d` / `sites-available` / `sites-enabled`).
   - Gerenciamento de Proxy Reverso (`proxy_pass`, `upstream`).
   - Gerenciamento de SSL/TLS (`/etc/nginx/ssl`, `/etc/letsencrypt`, expiração, emissor).
   - Visualização e filtragem em tempo real de `access.log` e `error.log`.

4. **Módulo Apache Tomcat**:
   - Controle de ciclo de vida (`systemctl start/stop/restart tomcat`).
   - Leitura e edição de `server.xml`, `context.xml`, `web.xml` e `tomcat-users.xml`.
   - Gerenciamento de conectores HTTP/AJP/HTTPS.
   - Deploy, upload de WARs, inicialização, parada e remoção de aplicações web (`webapps/`).
   - Métricas e ajustes da JVM (`JAVA_OPTS`, `CATALINA_OPTS`, Heap, Metaspace, GC).
   - Logs em tempo real de `catalina.out` e logs do sistema com filtros por nível (INFO, WARN, ERROR, DEBUG).

5. **Central Unificada de Segurança**:
   - Controle de acesso por IP e faixas CIDR (Allow / Deny).
   - Injeção e gestão de headers de segurança (HSTS, CSP, X-Frame-Options, XSS Protection, Referrer-Policy).
   - Políticas de limitação de taxa de requisições (`limit_req`).
   - Restrição e proteção de acesso ao Tomcat Manager.

6. **Backup, Versionamento e Rollback Automático**:
   - Criação de snapshots com hashes criptográficos SHA-256 (`SHA256SUMS`) antes de qualquer alteração de configuração.
   - Rollback automático e instantâneo em caso de falha de validação sintática.

7. **Segurança de Acesso e Matriz de Permissões**:
   - Autenticação corporativa via **Samba Active Directory / SSO** ou autenticação local Linux com **PAM Real** (`libpam.so.0`) para operadores com privilégios `wheel`/`sudo` ou `root`.
   - Superusuários PostgreSQL são estritamente isolados ao escopo do banco de dados.
   - Terminal operacional para intervenções de baixo nível.

---

## Estrutura de Rotas

O sistema opera como uma SPA centralizada sob o contexto `/infra/webserver`:
- `/infra/webserver/dashboard` — Painel consolidado de métricas e status
- `/infra/webserver/nginx` — Configuração, controle de serviço e proxy
- `/infra/webserver/tomcat` — Aplicações WAR, JVM, conectores e XMLs
- `/infra/webserver/vhosts` — Gerenciador de Virtual Hosts
- `/infra/webserver/reverse-proxy` — Regras de proxy reverso e balanceamento
- `/infra/webserver/ssl` — Certificados SSL/TLS e renovações
- `/infra/webserver/security` — Central unificada de segurança e ACL
- `/infra/webserver/monitoring` — Gráficos históricos e telemetria
- `/infra/webserver/logs` — Visualizador unificado de logs
- `/infra/webserver/backups` — Snapshots de configuração e restauração
- `/infra/webserver/terminal` — Console shell seguro

---

## Compilação e Execução

### Backend (Rust)
```bash
cargo check
cargo test
cargo build --release
```

### Frontend (React + Vite)
```bash
cd web
npm install
npm run build
```
