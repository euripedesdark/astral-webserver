# Astral Web Server

**Astral Web Server** es el módulo centralizado de administración de infraestructura web de **Astral Platform**, unificando la gestión de **Nginx** y **Apache Tomcat** en una única interfaz full stack desarrollada en **Rust** (backend con Axum) y **React + PrimeReact + Tailwind CSS** (frontend moderno en Vite).

---

## Funcionalidades Principales

1. **Descubrimiento Automático de Infraestructura**:
   - Detección instantánea de servidores web instalados (Nginx y Apache Tomcat).
   - Identificación de versiones, estado systemd, PIDs activos, puertos en escucha, consumo de CPU/RAM y tiempo de actividad.

2. **Panel de Control Unificado**:
   - Métricas en tiempo real de Nginx (workers, conexiones, peticiones/segundo, hosts virtuales).
   - Métricas de Tomcat (aplicaciones desplegadas, sesiones activas, hilos, memoria heap JVM, recolector de basura).
   - Telemetría de hardware (CPU, Memoria, Disco y Red).

3. **Módulo Nginx**:
   - Control de servicios: Iniciar, Detener, Reiniciar y Recargar (`reload`).
   - Editor de configuración con validación previa de sintaxis vía `nginx -t` antes de guardar.
   - Gestión integral de Hosts Virtuales (`conf.d`, `sites-available`, `sites-enabled`).
   - Gestión de Proxy Inverso (`proxy_pass`, `upstream`).
   - Gestión de certificados SSL/TLS (`/etc/nginx/ssl`, `/etc/letsencrypt`, caducidad, emisor).
   - Visualización y filtrado en tiempo real de `access.log` y `error.log`.

4. **Módulo Apache Tomcat**:
   - Control de ciclo de vida (`systemctl start/stop/restart tomcat`).
   - Lectura y edición de `server.xml`, `context.xml`, `web.xml` y `tomcat-users.xml`.
   - Gestión de conectores HTTP/AJP/HTTPS.
   - Despliegue, subida de WARs, inicio, detención y eliminación de aplicaciones web (`webapps/`).
   - Métricas y ajustes de la JVM (`JAVA_OPTS`, `CATALINA_OPTS`, Heap, Metaspace, GC).
   - Registros en tiempo real de `catalina.out` con filtros por nivel (INFO, WARN, ERROR, DEBUG).

5. **Central Unificada de Seguridad**:
   - Control de acceso por IP y rangos CIDR (Allow / Deny).
   - Encabezados de seguridad (HSTS, CSP, X-Frame-Options, XSS Protection, Referrer-Policy).
   - Políticas de límite de peticiones (`limit_req`).
   - Protección y restricción por IP para Tomcat Manager.

6. **Respaldos y Reversión Automática**:
   - Instantáneas con hash SHA-256 (`SHA256SUMS`) antes de cualquier modificación.
   - Reversión automática en caso de error sintáctico.

7. **Autenticación y Permisos**:
   - Samba Active Directory / SSO o autenticación local Linux con PAM Real (`libpam.so.0`) para operadores con permisos `wheel`/`sudo` o `root`.
   - Superusuarios de PostgreSQL aislados estrictamente de las funciones del sistema operativo.
   - Terminal web seguro integrado.
