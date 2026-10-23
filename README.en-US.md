# Astral Web Server

**Astral Web Server** is the centralized web infrastructure management module of the **Astral Platform**, unifying the administration of **Nginx** and **Apache Tomcat** into a single modern full-stack interface built in **Rust** (Axum backend) and **React + PrimeReact + Tailwind CSS** (Vite frontend).

---

## Core Capabilities

1. **Automatic Infrastructure Discovery**:
   - Automatic detection of installed web servers (Nginx and Apache Tomcat).
   - Identification of versions, systemd unit statuses, active PIDs, listening ports, CPU/RAM utilization, and uptime.

2. **Unified Dashboard**:
   - Real-time Nginx telemetry (workers, connections, requests/sec, virtual hosts).
   - Real-time Tomcat telemetry (deployed web applications, active sessions, worker threads, JVM heap, GC).
   - Server resource telemetry (CPU, Memory, Disk, Network).

3. **Nginx Module**:
   - Full service control: Start, Stop, Restart, and Reload.
   - Interactive configuration editor with mandatory `nginx -t` pre-validation before persistence.
   - Virtual Host management (`conf.d`, `sites-available`, `sites-enabled`).
   - Reverse proxy routing (`proxy_pass`, `upstream`).
   - SSL/TLS certificate management (`/etc/nginx/ssl`, `/etc/letsencrypt`, validity, issuer).
   - Real-time log streamer for `access.log` and `error.log` with IP, status code, and domain filters.

4. **Apache Tomcat Module**:
   - Lifecycle management (`systemctl start/stop/restart tomcat`).
   - Safe parsing and editing of `server.xml`, `context.xml`, `web.xml`, and `tomcat-users.xml`.
   - Connector administration (HTTP, HTTPS, AJP).
   - Application management: WAR deploy, upload, start, stop, reload, and undeploy (`webapps/`).
   - JVM tuning (`JAVA_OPTS`, `CATALINA_OPTS`, Heap, Metaspace, GC metrics).
   - Real-time log inspector for `catalina.out` with level filtering (INFO, WARN, ERROR, DEBUG).

5. **Centralized Security Suite**:
   - IP and CIDR range access control (Allow / Deny).
   - Security header enforcement (HSTS, CSP, X-Frame-Options, XSS Protection, Referrer-Policy).
   - Rate limiting policies (`limit_req`).
   - Hardening and IP isolation for Tomcat Manager.

6. **Backup, Versioning & Automatic Rollback**:
   - Cryptographic SHA-256 snapshots (`SHA256SUMS`) created prior to any file edit.
   - Automatic rollback in case of syntax or validation failure.

7. **Authentication & RBAC**:
   - Samba Active Directory / SSO or real PAM (`libpam.so.0` FFI) local authentication for `root` and `wheel`/`sudo` users.
   - Strict isolation of PostgreSQL superusers.
   - Integrated web terminal for authorized system operators.

---

## Route Structure

All views share a single context under `/infra/webserver`:
- `/infra/webserver/dashboard` — Unified metrics overview
- `/infra/webserver/nginx` — Nginx service and configuration
- `/infra/webserver/tomcat` — Tomcat applications, JVM, and connectors
- `/infra/webserver/vhosts` — Virtual host manager
- `/infra/webserver/reverse-proxy` — Reverse proxy and routing rules
- `/infra/webserver/ssl` — SSL/TLS certificates and renewal
- `/infra/webserver/security` — Access control, rate limiting, and headers
- `/infra/webserver/monitoring` — Real-time telemetry and charts
- `/infra/webserver/logs` — Unified log viewer
- `/infra/webserver/backups` — Configuration snapshots and restore
- `/infra/webserver/terminal` — Embedded shell terminal
