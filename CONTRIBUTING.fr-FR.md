# Contribuer à Astral Web Server

## Code de Conduite
Ce projet et l’ensemble des contributeurs sont régis par le Code de Conduite.

## Configuration de Développement

### Prérequis
* Rust 1.80+ (cargo, rustc)
* Node.js 20+ & npm 10+
* Nginx 1.24+
* Apache Tomcat 10.1+

### Compilation et Exécution

```bash
# Backend Rust
cargo test
cargo build --release

# Frontend React
cd web
npm install
npm run build
```

## Structure du Projet

```
astral-webserver/
├── Cargo.toml                 # Dépendances et métadonnées Rust
├── Cargo.lock                 # Versions verrouillées des dépendances
├── src/                       # Code source du backend Axum
│   ├── main.rs                # Point d'entrée, initialisation du serveur et SPA
│   ├── config.rs              # Configuration des ports, répertoires et secrets
│   ├── audit.rs               # Piste d'audit opérationnelle en mémoire tampon circulaire
│   ├── auth.rs                # PAM réel dynamique (root/wheel/sudo) et Samba AD / JWT
│   ├── discovery.rs           # Détection des démons Nginx, Tomcat et télémétrie
│   ├── nginx.rs               # Contrôle de service, vhosts et validation (nginx -t)
│   ├── tomcat.rs              # Contrôle de service, connecteurs, JVM et déploiement WAR
│   ├── security.rs            # Centre de sécurité, filtrage IP CIDR et en-têtes HTTP
│   ├── monitoring.rs          # Télémétrie CPU/RAM/Disque et flux WebSocket
│   ├── backup.rs              # Instantanés SHA-256 et restauration de configurations
│   ├── terminal.rs            # Console opérationnelle d'administration
│   └── api.rs                 # Routeur REST unifié (/api/*)
├── web/                       # Frontend SPA React 18 + Vite + Tailwind CSS
│   ├── src/
│   │   ├── main.jsx           # Montage de l'application React
│   │   ├── App.jsx            # Interface unifiée avec sous-routes (/infra/webserver/*)
│   │   ├── i18n.js            # Dictionnaire multilingue (pt-BR, en-US, es-ES, fr-FR)
│   │   ├── index.css          # Styles Tailwind et mode sombre
│   │   └── components/
│   │       └── DonationModal.jsx # Modal de dons PIX et Wise
│   ├── package.json           # Dépendances npm
│   ├── vite.config.js         # Configuration Vite (base : /webserver/)
│   └── dist/                  # Artefacts de production compilés
├── .github/workflows/         # Pipelines CI/CD GitHub Actions
└── docs/i18n/                 # Documentation multilingue
```

## Lignes Directrices pour les Pull Requests
1. Créez une branche descriptive à partir de main.
2. Ajoutez des tests unitaires pour les nouvelles fonctionnalités.
3. Assurez-vous que npm run build et cargo test réussissent sans erreur.
4. Soumettez la Pull Request avec un résumé clair des modifications.
