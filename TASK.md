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

### Phase 10: Webhooks & CI/CD ✅

- [x] **Webhook Endpoints**
  - [x] `/api/webhooks/deploy/:project-id/:token`
  - [x] Token-based authentication
  - [x] Trigger: pull latest image & restart container
  - [x] Webhook logs/history

- [x] **GitHub Actions CI/CD**
  - [x] Backend CI (Format, Clippy, Build, Test)
  - [x] Frontend CI (Type check, Build)
  - [x] Automated multi-arch releases (x86_64, aarch64)

- [x] **Deployment Automation**
  - [x] Scheduled pulls (cron-like)
  - [x] Health check after deploy
  - [x] Rollback on failure

### Phase 11: Resource Management ✅

- [x] **Resource Limits**
  - [x] CPU limit per container
  - [x] Memory limit per container

- [x] **Resource Monitoring**
  - [x] Historical metrics (store in SQLite)
  - [x] Resource usage graphs (SVG-based reactive charts)

### Phase 12: Multi-User & Teams ✅

- [x] **User Roles**
  - [x] Admin (full access)
  - [x] Developer (projects, containers)
  - [x] Viewer (read-only)

- [x] **Teams**
  - [x] Create team/organization
  - [x] Invite users to team
  - [x] Project ownership transfer

### Phase 13: Advanced Features ✅

- [x] **Dynamic Templates**
  - [x] Pre-built app templates (WordPress, Ghost, Redis)
  - [x] Database-backed template registry
  - [x] Import templates via JSON or URL
  - [x] One-click deploy from gallery

- [x] **Stack Backup & Restore**
  - [x] Export stack metadata and compose as JSON
  - [x] Portable backup files
  - [x] One-click restore to any team

- [x] **Git Integration**
  - [x] Clone and deploy from public Git repositories
  - [x] Support for custom branches
  - [x] Git sync & redeploy functionality
  - [x] Build from Dockerfile
  - [x] Build logs streaming

- [x] **Terminal & Logs**
  - [x] Interactive Terminal Exec (xterm.js + WebSockets)
  - [x] Real-time build log viewer (SSE)
  - [x] Individual service build support (Hammer icon)

### Phase 14: Maintenance & Code Cleanup ✅

- [x] **System Stability**
  - [x] Periodic resource metrics pruning (30 days)
  - [x] Strict Clippy linting enforcement
  - [x] Zero-warning backend compilation
- [x] **Team Management**
  - [x] Complete member role management
  - [x] Team deletion logic

### Phase 15: Frontend Refactoring (Pragmatic Feature-Sliced) ✅

- [x] **Infrastructure Setup**
  - [x] Create `$lib/features` directory structure
  - [x] Define standardized pattern for `.svelte.ts` logic

- [x] **Core Features Refactor**
  - [x] Extract `StackController.svelte.ts` and componentize Stack details
  - [x] Extract `TeamController.svelte.ts` and componentize Team management
  - [x] Extract `ContainerController.svelte.ts` and componentize Container details/list
  - [x] Extract `DashboardController.svelte.ts` and componentize Dashboard overview
  - [x] Extract `TemplateController.svelte.ts` and componentize Templates gallery
  - [x] Extract `SettingsController.svelte.ts` and componentize Profile/Appearance/Registries
  - [x] Extract `LogsController.svelte.ts` and componentize Logs viewer
  - [x] Extract `AuthController.svelte.ts` and componentize Login/Register

- [x] **Global State & Stability**
  - [x] Audit `$lib/stores` and move feature-specific state to controllers
  - [x] Resolve all TypeScript errors (svelte-check)
  - [x] Verify production build and reactivity stability

### Phase 20: Docker Swarm & Multi-Node Support (Planned)

- [ ] **Swarm Orchestration**
  - [ ] Initialize/Join Swarm cluster UI
  - [ ] Node management (List, Promote, Demote, Remove)
  - [ ] Node health and resource monitoring
- [ ] **Swarm Services**
  - [ ] Support `docker stack deploy` equivalent via `bollard`
  - [ ] Manage Swarm Services (Service mode: Replicated vs Global)
  - [ ] Zero-downtime rolling updates support
- [ ] **Advanced Networking**
  - [ ] Multi-node Overlay networks
  - [ ] Service Mesh / Ingress integration

### Phase 21: Advanced Domain Management (Planned)

- [ ] **Dedicated Domain Dashboard**
  - [ ] Centralized view for all domains across all stacks
  - [ ] Domain health monitoring (SSL status, DNS resolution check)
- [ ] **Multi-Provider DNS Integration**
  - [ ] Cloudflare API integration (Manage DNS records directly from Labuh)
  - [ ] cPanel API support for automated staging/production entry
- [ ] **Cloudflare Tunnel (Zero Trust) Support**
  - [ ] Automated `cloudflared` tunnel setup and management
  - [ ] Deploy shared `labuh-tunnel` container on `labuh-network`
  - [ ] Link tunnels to internal project services by routing to container names (zero host ports)
  - [ ] Dynamic tunnel configuration via Cloudflare API

  > [!NOTE]
  > **Ingress Architecture**: Domains via DNS/cPanel will route through **Caddy** (Public IP 80/443), while Tunnel-based domains will route directly via the **cloudflared** container to app containers (Private/No Host Ports).

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
