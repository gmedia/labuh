# Labuh - Lightweight PaaS Platform

> **Labuh** (dari bahasa Melayu/Indonesia: berlabuh = to dock/berth) adalah platform PaaS sederhana dan ringan untuk deploy, pull, dan build container images.

## 🎯 Goals

- **Ringan**: Dirancang untuk server dengan resource terbatas
- **Simple**: Fokus pada fitur esensial tanpa kompleksitas berlebihan
- **Modern**: UI yang clean dan responsive dengan dashboard yang informatif

---

## 🛠️ Tech Stack

### Backend

- **Language**: Rust (untuk performa dan memory efficiency)
- **Container Runtime**: Docker API via bollard (works with containerd via Docker shim)
- **HTTP Framework**: [Axum](https://docs.rs/axum/latest/axum/) (async, performant, ergonomic)
- **Database**: SQLite via [SQLx](https://docs.rs/sqlx/latest/sqlx/) (async, compile-time checked queries)
- **Authentication**: JWT via [jsonwebtoken](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/)

### Frontend

- **Framework**: [SvelteKit](https://kit.svelte.dev/)
- **UI Components**: [shadcn-svelte](https://www.shadcn-svelte.com/)
- **Styling**: TailwindCSS (included with shadcn-svelte)
- **Icons**: [Lucide Icons](https://lucide.dev/)

### Infrastructure

- **Database**: SQLite (single file, zero configuration)
- **Container Runtime**: Docker/containerd
- **Reverse Proxy**: Caddy (optional, untuk production)

---

## 📋 Features Roadmap

### Phase 1: Foundation (MVP) ✅

- [x] **Backend Setup**
  - [x] Project structure dengan Axum
  - [x] SQLite database setup dengan SQLx + migrations
  - [x] Configuration management (env vars)
  - [x] Basic error handling dan logging
  - [x] Health check endpoint

- [x] **Authentication**
  - [x] User registration
  - [x] Login dengan JWT
  - [x] Middleware authentication
  - [x] Session management

- [x] **Frontend Setup**
  - [x] SvelteKit project initialization
  - [x] shadcn-svelte integration
  - [x] Base layout dengan sidebar navigation
  - [x] Auth pages (login/register)
  - [x] Dark/Light mode toggle

### Phase 2: Container Management ✅

- [x] **Container Runtime Integration**
  - [x] Docker API client connection (bollard)
  - [x] List running containers
  - [x] Container logs (streaming via SSE)
  - [x] Container stats (CPU, memory, network)

- [x] **Image Management**
  - [x] Pull image dari registry
  - [x] List local images
  - [x] Delete image
  - [x] Image details (size, created date)

- [x] **Container Lifecycle**
  - [x] Create container dari image
  - [x] Start/Stop/Restart container
  - [x] Delete container
  - [x] Environment variables configuration

### Phase 3: Application Deployment ✅

- [x] **Projects/Apps**
  - [x] Create project/application
  - [x] Link project ke container
  - [x] Project settings (env vars, port)
  - [x] Deploy/Stop/Restart endpoints

- [x] **Networking**
  - [x] Caddy integration for routing

### Phase 4: Dashboard & Monitoring ✅

- [x] **Dashboard**
  - [x] System overview (CPU, memory, disk usage)
  - [x] Running containers count
  - [x] Quick actions

- [x] **Monitoring**
  - [x] Real-time container metrics
  - [x] Container detail page with stats

- [x] **Logs**
  - [x] Centralized log viewer
  - [x] Log search dan filter
  - [x] Log download
  - [x] Container detail logs

### Phase 5: Polish & Production Ready ✅

- [x] **Security**
  - [x] CORS configuration
  - [x] JWT authentication

- [x] **Documentation**
  - [x] README.md
  - [x] User guide (in frontend /docs)
  - [x] Deployment guide (in frontend /docs)
  - [x] API reference (in frontend /docs)

- [x] **DevOps**
  - [x] Docker Compose setup
  - [x] Dockerfiles (backend + frontend)
  - [x] Systemd service file
  - [x] Installation script
  - [x] Backup & restore scripts

---

## 🚀 Getting Started

```bash
# Development
cargo run             # Backend
cd frontend && npm run dev  # Frontend

# Production
docker-compose up -d
```

---

## 📚 Documentation

Visit `/docs` in the frontend for:

- User Guide
- Deployment Guide
- API Reference

---

## 📝 Notes

### Why Docker API (bollard) instead of direct containerd?

- Bollard provides stable, well-documented API
- Works with both Docker and containerd (via Docker shim)
- For containerd-only: use `nerdctl` with Docker API compatibility
- Easier deployment and broader compatibility

### Why SQLite?

- Zero configuration, single file database
- Tidak perlu database server terpisah
- Mudah untuk backup (cukup copy file)

### Why Axum?

- Built on top of Tokio (battle-tested async runtime)
- Type-safe routing dan extractors
- Excellent performance
