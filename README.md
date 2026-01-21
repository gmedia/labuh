# Labuh - Lightweight PaaS Platform

> **Labuh** (dari bahasa Melayu/Indonesia: berlabuh = to dock/berth) adalah platform PaaS sederhana dan ringan untuk deploy, pull, dan build container images.

## 🎯 Features

- **Authentication**: JWT-based auth with Argon2 password hashing
- **Container Management**: List, create, start/stop/restart, remove containers
- **Image Management**: Pull, list, delete images
- **Projects**: CRUD operations for application projects
- **System Monitoring**: CPU, memory, disk, and uptime stats
- **Reverse Proxy**: Caddy with auto SSL and dynamic routing
- **Modern UI**: SvelteKit + shadcn-svelte with dark mode

## 🛠️ Tech Stack

### Backend

- **Rust** with Axum (async web framework)
- **SQLite** via SQLx (async, compile-time checked queries)
- **Docker API** via bollard

### Frontend

- **SvelteKit** with TypeScript
- **shadcn-svelte** for UI components
- **TailwindCSS v4**

## 🚀 Quick Start

### Development

```bash
# Backend
cd labuh
cp .env.example .env
cargo run

# Frontend (separate terminal)
cd frontend
npm install
npm run dev
```

### Production (Docker Compose)

```bash
docker-compose up -d
```

This starts:

- **Labuh backend** on port 3000
- **Caddy reverse proxy** on ports 80/443
- **Frontend** on port 5173

## 📋 API Endpoints

### Public

- `GET /api/health` - Health check
- `GET /api/system/stats` - System statistics

### Authentication

- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login, returns JWT
- `GET /api/auth/me` - Get current user (protected)

### Containers (protected)

- `GET /api/containers` - List containers
- `POST /api/containers` - Create container
- `POST /api/containers/:id/start` - Start container
- `POST /api/containers/:id/stop` - Stop container
- `POST /api/containers/:id/restart` - Restart container
- `DELETE /api/containers/:id` - Delete container
- `GET /api/containers/:id/logs` - Get container logs
- `GET /api/containers/:id/stats` - Get container stats

### Images (protected)

- `GET /api/images` - List images
- `POST /api/images/pull` - Pull image
- `DELETE /api/images/:id` - Delete image

### Projects (protected)

- `GET /api/projects` - List projects
- `POST /api/projects` - Create project
- `GET /api/projects/:id` - Get project
- `PUT /api/projects/:id` - Update project
- `DELETE /api/projects/:id` - Delete project

## 🗄️ Database

SQLite with migrations for:

- Users
- Projects
- Deployments
- Activity logs

## 📁 Project Structure

```
labuh/
├── src/                    # Rust backend
│   ├── main.rs
│   ├── config.rs
│   ├── db/                 # Database
│   ├── models/             # Data models
│   ├── handlers/           # HTTP handlers
│   ├── services/           # Business logic
│   └── middleware/         # Auth middleware
├── migrations/             # SQLite migrations
├── frontend/               # SvelteKit frontend
│   ├── src/
│   │   ├── routes/         # Pages
│   │   └── lib/            # Components, stores, API
│   └── package.json
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
└── Caddyfile
```

## 📝 Environment Variables

```env
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite:./labuh.db?mode=rwc
JWT_SECRET=your-secret-key
JWT_EXPIRATION_HOURS=24
CADDY_ADMIN_API=http://localhost:2019
```

## 📚 License

MIT
