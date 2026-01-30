# Labuh - Lightweight PaaS Platform

> **Labuh** (dari bahasa Melayu/Indonesia: berlabuh = to dock/berth) adalah platform PaaS sederhana dan ringan untuk deploy, pull, dan build container images.

## 🎯 Goals

- **Ringan**: Dirancang untuk server dengan resource terbatas
- **Simple**: Fokus pada fitur esensial tanpa kompleksitas berlebihan
- **Modern**: UI yang clean dan responsive dengan dashboard yang informatif

---

## ✅ Completed Features

### Phase 1-5: Foundation to Production Ready

- [x] Backend (Axum, SQLx, JWT Auth, Docker API via bollard)
- [x] Frontend (SvelteKit, shadcn-svelte, dark mode)
- [x] Container Management (CRUD, logs, stats, SSE streaming)
- [x] Image Management (list, pull, delete)
- [x] Projects (CRUD, deploy, stop, restart)
- [x] Dashboard (system stats, monitoring)
- [x] Documentation (user guide, deployment, API reference)
- [x] DevOps (Docker Compose, systemd, backup/restore)

---

## 🚀 Next Features (Roadmap)

### Phase 6: Environment & Configuration ✅

- [x] **Environment Variable Management**
  - [x] Per-container env vars editor
  <!-- - [ ] Per-project env vars (inherited by containers) not relevant cause project has been deleted -->
  - [x] Secret masking in UI
  - [x] Import from .env file

- [x] **Port Management**
  - [x] Expose ports per container
  - [x] Port mapping UI (host:container)
  - [x] Auto-detect exposed ports from image

### Phase 7: Docker Compose Support ✅

- [x] **Docker Compose Import**
  - [x] Paste docker-compose.yml to create stack
  - [x] Parse and create multiple containers
  - [x] Handle networks (create shared network)
  - [x] Handle volumes
  - [x] Stack management (start/stop all)

- [x] **Stack View**
  - [x] Group containers by stack/compose file
  - [x] Stack-level logs viewer
  - [x] Stack health overview

### Phase 8: Domain & Routing (Caddy) ✅

- [x] **Domain Management**
  - [x] Add custom domains per project
  - [x] Subdomain auto-generation (app-name.labuh.local)
  - [x] DNS verification
  - [x] SSL auto-provisioning (Let's Encrypt via Caddy)

- [x] **Caddy Integration**
  - [x] Auto-create Caddy container (port 80, 443)
  - [x] Dynamic route updates via Caddy API
  - [x] Handle Docker networks (connect Caddy to container networks)
  - [x] Reverse proxy configuration UI
  - [x] Basic Auth protection per route

- [x] **Networking**
  - [x] Create labuh-network for all containers
  - [x] Connect Caddy to all project networks
  - [x] Internal DNS resolution (container-name.labuh)

### Phase 9: Private Registry & Auth ✅

- [x] **Registry Credentials**
  - [x] Store Docker Hub credentials
  - [x] Store GitHub Container Registry (ghcr.io) tokens
  - [x] Store custom registry credentials
  - [x] Credential management UI
  - [x] Per-project registry config (using global user credentials for now)

- [x] **Authenticated Pull**
  - [x] Use stored credentials when pulling
  - [x] Support for private images

### Phase 10: Webhooks & CI/CD

- [x] **Webhook Endpoints**
  - [x] `/api/webhooks/deploy/:project-id/:token`
  - [x] Token-based authentication
  - [x] Trigger: pull latest image & restart container
  - [x] Webhook logs/history

- [x] **GitHub Actions CI/CD**
  - [x] Backend CI (Format, Clippy, Build, Test)
  - [x] Frontend CI (Type check, Build)
  - [x] Automated multi-arch releases (x86_64, aarch64)

- [ ] **GitHub Integration**
  - [ ] GitHub webhook receiver
  - [ ] Trigger on push to branch
  - [ ] Auto-deploy on tag/release

- [ ] **Deployment Automation**
  - [ ] Scheduled pulls (cron-like)
  - [ ] Health check after deploy
  - [ ] Rollback on failure
  - [ ] Deployment notifications (Discord/Slack/Email)

### Phase 11: Resource Management

- [ ] **Resource Limits**
  - [ ] CPU limit per container
  - [ ] Memory limit per container
  - [ ] Storage quota per project

- [ ] **Resource Monitoring**
  - [ ] Historical metrics (store in SQLite)
  - [ ] Resource usage graphs (24h, 7d, 30d)
  - [ ] Alerts for high usage

### Phase 12: Multi-User & Teams

- [ ] **User Roles**
  - [ ] Admin (full access)
  - [ ] Developer (projects, containers)
  - [ ] Viewer (read-only)

- [ ] **Teams**
  - [ ] Create team/organization
  - [ ] Invite users to team
  - [ ] Project ownership transfer

### Phase 13: Advanced Features

- [ ] **Templates**
  - [ ] Pre-built app templates (WordPress, Ghost, etc)
  - [ ] One-click deploy from template
  - [ ] Community templates

- [ ] **Backup & Restore**
  - [ ] Container data backup
  - [ ] Volume backup
  - [ ] Scheduled backups

- [ ] **Git Integration**
  - [ ] Clone repo and build with Dockerfile
  - [ ] Build logs streaming
  - [ ] Auto-rebuild on push

---

## 🏗️ Architecture Migration ✅

- [x] **Migrate to Hexagonal Architecture**
  - [x] Setup directory structure (`domain/`, `usecase/`, `infrastructure/`, `api/`)
  - [x] Define Domain Ports (Traits)
  - [x] Implement Infrastructure Adapters (SQLite, Docker)
  - [x] Refactor Services to UseCases
  - [x] Update API Handlers
  - [x] Final Wiring in `main.rs`

---

## 📝 Implementation Notes

### Caddy Container Setup

```yaml
# When Labuh starts, ensure Caddy container exists:
labuh-caddy:
  image: caddy:2-alpine
  ports:
    - "80:80"
    - "443:443"
  networks:
    - labuh-network
  volumes:
    - caddy_data:/data
    - caddy_config:/config
```

### Docker Compose Parsing

- Use `serde_yaml` to parse compose file
- Create containers via bollard API
- Create networks and connect containers
- Store compose file in SQLite for stack management

### Webhook Flow

```
GitHub Push → Webhook → Labuh API → Pull Latest Image → Restart Container → Notify
```

---

## 🛠️ Tech Stack

- **Backend**: Rust, Axum, SQLx, bollard
- **Frontend**: SvelteKit, shadcn-svelte, TailwindCSS
- **Database**: SQLite
- **Reverse Proxy**: Caddy
- **Container Runtime**: Docker / containerd

---

## 🚀 Getting Started

```bash
# Development
cargo run             # Backend
cd frontend && npm run dev  # Frontend

# Production
docker-compose up -d
```
