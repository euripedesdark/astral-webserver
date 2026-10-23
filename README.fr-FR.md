# Astral Web Server

**Astral Web Server** est le module unifié d'administration d'infrastructure web de la plateforme **Astral Platform**, centralisant la gestion de **Nginx** et **Apache Tomcat** au sein d'une interface moderne conçue en **Rust** (backend Axum) et **React + PrimeReact + Tailwind CSS** (frontend Vite).

---

## Fonctionnalités Principales

1. **Découverte Automatique de l'Infrastructure**:
   - Détection en temps réel des serveurs web installés (Nginx et Apache Tomcat).
   - Identification des versions, états systemd, PID actifs, ports d'écoute, utilisation CPU/RAM et uptime.

2. **Tableau de Bord Unifié**:
   - Métriques en direct de Nginx (workers, connexions, requêtes/s, hôtes virtuels).
   - Métriques de Tomcat (applications déployées, sessions actives, threads, mémoire JVM, garbage collector).
   - Télémétrie des ressources matérielles (CPU, Mémoire, Disque, Réseau).

3. **Module Nginx**:
   - Contrôle du service : Démarrage, Arrêt, Redémarrage et Rechargement (`reload`).
   - Éditeur de configuration avec validation syntaxique `nginx -t` préalable obligatoire.
   - Gestion des hôtes virtuels (`conf.d`, `sites-available`, `sites-enabled`).
   - Gestion du proxy inverse (`proxy_pass`, `upstream`).
   - Gestion des certificats SSL/TLS (`/etc/nginx/ssl`, `/etc/letsencrypt`, validité, émetteur).
   - Visualisation et filtrage en direct des journaux `access.log` et `error.log`.

4. **Module Apache Tomcat**:
   - Gestion du service (`systemctl start/stop/restart tomcat`).
   - Lecture et modification sécurisée de `server.xml`, `context.xml`, `web.xml` et `tomcat-users.xml`.
   - Gestion des connecteurs HTTP/AJP/HTTPS.
   - Déploiement et téléversement de fichiers WAR, arrêt, démarrage et suppression (`webapps/`).
   - Paramétrage de la JVM (`JAVA_OPTS`, `CATALINA_OPTS`, Heap, Metaspace, GC).
   - Suivi des journaux `catalina.out` avec filtrage par niveau (INFO, WARN, ERROR, DEBUG).

5. **Centre Unifié de Sécurité**:
   - Contrôle d'accès par adresse IP et plages CIDR (Allow / Deny).
   - En-têtes de sécurité (HSTS, CSP, X-Frame-Options, XSS Protection, Referrer-Policy).
   - Limitation de débit de requêtes (`limit_req`).
   - Sécurisation et restriction d'accès au Tomcat Manager.

6. **Sauvegarde et Restauration Automatique**:
   - Instantanés avec sommes de contrôle SHA-256 (`SHA256SUMS`) avant toute modification.
   - Retour automatique en arrière (rollback) en cas d'erreur de validation syntaxique.

7. **Authentification et Sécurité des Accès**:
   - Samba Active Directory / SSO ou authentification locale Linux via PAM Réel (`libpam.so.0`) pour les comptes `root` et membres du groupe `wheel`/`sudo`.
   - Isolation stricte des superutilisateurs PostgreSQL.
   - Console terminal web intégrée.
