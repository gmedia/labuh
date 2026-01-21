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
- **Container Runtime**: containerd via [containerd-rs](https://docs.rs/containerd/latest/containerd/)
- **HTTP Framework**: [Axum](https://docs.rs/axum/latest/axum/) (async, performant, ergonomic)
- **Database**: SQLite via [SQLx](https://docs.rs/sqlx/latest/sqlx/) (async, compile-time checked queries)
- **Authentication**: JWT via [jsonwebtoken](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/)
- **Configuration**: [config-rs](https://docs.rs/config/latest/config/) + environment variables

### Frontend

- **Framework**: [SvelteKit](https://kit.svelte.dev/)
- **UI Components**: [shadcn-svelte](https://www.shadcn-svelte.com/)
- **Styling**: TailwindCSS (included with shadcn-svelte)
- **Icons**: [Lucide Icons](https://lucide.dev/)
- **State Management**: Svelte stores + server-side load functions

### Infrastructure

- **Database**: SQLite (single file, zero configuration)
- **Container Runtime**: containerd (lebih ringan dari Docker daemon)
- **Reverse Proxy**: Caddy/Traefik (optional, untuk production)

---

## 📋 Features Roadmap

### Phase 1: Foundation (MVP)

- [ ] **Backend Setup**
  - [ ] Project structure dengan Axum
  - [ ] SQLite database setup dengan SQLx + migrations
  - [ ] Configuration management (env vars, config file)
  - [ ] Basic error handling dan logging
  - [ ] Health check endpoint

- [ ] **Authentication**
  - [ ] User registration (untuk admin pertama)
  - [ ] Login dengan JWT
  - [ ] Middleware authentication
  - [ ] Session management

- [ ] **Frontend Setup**
  - [ ] SvelteKit project initialization
  - [ ] shadcn-svelte integration
  - [ ] Base layout dengan sidebar navigation
  - [ ] Auth pages (login/register)
  - [ ] Dark/Light mode toggle

### Phase 2: Container Management

- [ ] **Container Runtime Integration**
  - [ ] Containerd client connection
  - [ ] List running containers
  - [ ] Container logs (streaming)
  - [ ] Container stats (CPU, memory, network)

- [ ] **Image Management**
  - [ ] Pull image dari registry (Docker Hub, private registry)
  - [ ] List local images
  - [ ] Delete image
  - [ ] Image details (layers, size, created date)

- [ ] **Container Lifecycle**
  - [ ] Create container dari image
  - [ ] Start/Stop/Restart container
  - [ ] Delete container
  - [ ] Environment variables configuration
  - [ ] Port mapping
  - [ ] Volume mounts

### Phase 3: Application Deployment

- [ ] **Projects/Apps**
  - [ ] Create project/application
  - [ ] Link project ke container
  - [ ] Project settings (env vars, domains)
  - [ ] Deployment history

- [ ] **Build from Source (Optional)**
  - [ ] Git repository integration
  - [ ] Dockerfile-based build dengan buildkit
  - [ ] Build logs streaming
  - [ ] Auto-deploy on push (webhook)

- [ ] **Networking**
  - [ ] Custom domains per project
  - [ ] SSL/TLS certificates (via Caddy/Let's Encrypt)
  - [ ] Internal networking antar containers

### Phase 4: Dashboard & Monitoring

- [ ] **Dashboard**
  - [ ] System overview (CPU, memory, disk usage)
  - [ ] Running containers count
  - [ ] Recent deployments
  - [ ] Quick actions

- [ ] **Monitoring**
  - [ ] Real-time container metrics
  - [ ] Resource usage graphs
  - [ ] Alerts (container down, high resource usage)

- [ ] **Logs**
  - [ ] Centralized log viewer
  - [ ] Log search dan filter
  - [ ] Log download

### Phase 5: Polish & Production Ready

- [ ] **Security Hardening**
  - [ ] Rate limiting
  - [ ] CORS configuration
  - [ ] Secure headers
  - [ ] Audit logging

- [ ] **Documentation**
  - [ ] API documentation (OpenAPI/Swagger)
  - [ ] User guide
  - [ ] Deployment guide

- [ ] **DevOps**
  - [ ] Docker image untuk Labuh sendiri
  - [ ] Systemd service file
  - [ ] Backup & restore scripts

---

## 📁 Project Structure (Planned)

```
labuh/
├── backend/                    # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs          # Configuration
│   │   ├── db/                # Database layer
│   │   │   ├── mod.rs
│   │   │   └── migrations/
│   │   ├── handlers/          # HTTP handlers
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── containers.rs
│   │   │   ├── images.rs
│   │   │   └── projects.rs
│   │   ├── models/            # Data models
│   │   ├── services/          # Business logic
│   │   │   ├── mod.rs
│   │   │   ├── containerd.rs  # Containerd client
│   │   │   └── auth.rs
│   │   └── middleware/        # Auth, logging, etc
│   ├── Cargo.toml
│   └── .env.example
│
├── frontend/                   # SvelteKit frontend
│   ├── src/
│   │   ├── routes/
│   │   │   ├── +layout.svelte
│   │   │   ├── +page.svelte   # Dashboard
│   │   │   ├── login/
│   │   │   ├── containers/
│   │   │   ├── images/
│   │   │   └── projects/
│   │   ├── lib/
│   │   │   ├── components/    # UI components
│   │   │   ├── api/           # API client
│   │   │   └── stores/        # Svelte stores
│   │   └── app.css
│   ├── package.json
│   └── svelte.config.js
│
├── migrations/                 # SQLite migrations
├── docker-compose.yml         # Development setup
├── Dockerfile                 # Production build
└── README.md
```

---

## 🗄️ Database Schema (Draft)

```sql
-- Users
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT,
    role TEXT DEFAULT 'user',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Projects/Applications
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    container_id TEXT,
    image TEXT,
    status TEXT DEFAULT 'stopped',
    port INTEGER,
    env_vars TEXT,  -- JSON
    domains TEXT,   -- JSON array
    user_id TEXT REFERENCES users(id),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Deployments history
CREATE TABLE deployments (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    image TEXT NOT NULL,
    status TEXT DEFAULT 'pending',
    logs TEXT,
    started_at DATETIME,
    finished_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Activity/Audit logs
CREATE TABLE activity_logs (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id),
    action TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    details TEXT,  -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 🔌 API Endpoints (Draft)

### Authentication

```
POST   /api/auth/register     # Register new user
POST   /api/auth/login        # Login, returns JWT
POST   /api/auth/logout       # Logout
GET    /api/auth/me           # Get current user
```

### Containers

```
GET    /api/containers        # List containers
GET    /api/containers/:id    # Get container details
POST   /api/containers        # Create container
POST   /api/containers/:id/start   # Start container
POST   /api/containers/:id/stop    # Stop container
POST   /api/containers/:id/restart # Restart container
DELETE /api/containers/:id    # Delete container
GET    /api/containers/:id/logs    # Get container logs (SSE)
GET    /api/containers/:id/stats   # Get container stats (SSE)
```

### Images

```
GET    /api/images            # List images
POST   /api/images/pull       # Pull image from registry
DELETE /api/images/:id        # Delete image
GET    /api/images/:id        # Get image details
```

### Projects

```
GET    /api/projects          # List projects
POST   /api/projects          # Create project
GET    /api/projects/:id      # Get project details
PUT    /api/projects/:id      # Update project
DELETE /api/projects/:id      # Delete project
POST   /api/projects/:id/deploy    # Deploy project
GET    /api/projects/:id/deployments # List deployments
```

### System

```
GET    /api/health            # Health check
GET    /api/system/stats      # System stats (CPU, memory, disk)
```

---

## 📝 Notes & Considerations

### Why Containerd over Docker?

- Lebih ringan karena tidak perlu Docker daemon
- Langsung interact dengan container runtime
- Cocok untuk embedded/lightweight systems
- Docker sendiri menggunakan containerd di belakang layar

### Why SQLite?

- Zero configuration, single file database
- Tidak perlu database server terpisah
- Cukup untuk single-server deployment
- Mudah untuk backup (cukup copy file)
- WAL mode untuk better concurrency

### Why Axum?

- Built on top of Tokio (battle-tested async runtime)
- Type-safe routing dan extractors
- Great ergonomics dengan tower middleware ecosystem
- Excellent performance

### Containerd Access

- Perlu akses ke containerd socket (`/run/containerd/containerd.sock`)
- Backend harus running sebagai user dengan permission ke socket
- Atau: setup user namespace remapping

---

## 🚀 Getting Started (TODO)

```bash
# Development
cd backend && cargo run
cd frontend && npm run dev

# Production
docker-compose up -d
```

---

## 📚 References

- [containerd-rs docs](https://docs.rs/containerd/latest/containerd/)
- [Axum examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [SvelteKit docs](https://kit.svelte.dev/docs)
- [shadcn-svelte docs](https://www.shadcn-svelte.com/docs)
- [SQLx docs](https://github.com/launchbadge/sqlx)
