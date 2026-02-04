# Labuh ⚓

> **Labuh** (dari bahasa Melayu/Indonesia: _berlabuh_) — Lightweight and professional PaaS for modern servers and edge devices.

Labuh is a simple yet powerful Platform-as-a-Service (PaaS) designed to deploy, manage, and scale containerized applications with ease. Built with Rust for maximum performance and minimum memory footprint.

## 📖 Documentation & Landing Page

For detailed guides, installation instructions, and architecture overview, please visit our official website:

👉 **[https://gmedia.github.io/labuh/](https://gmedia.github.io/labuh/)** (Or your GitHub Pages URL)

## ✨ Highlights

- **Universal Multi-Arch**: Runs seamlessly on x86_64 servers and ARM64 edge devices (like STBs).
- **Single Binary API**: All-in-one backend that serves both the API and the Dashboard.
- **Docker Compose Native**: Deploy production-ready stacks using standard YAML files.
- **Zero-Config SSL**: Automatic HTTPS via integrated Caddy server.
- **Modern Dashboard**: Clean and reactive UI built with Svelte 5.
- **CI/CD Automation**: Version-controlled deployments via secure webhooks.

## 📂 Project Structure

```text
.
├── backend/            # Rust API & SQLite Migrations
├── frontend/           # SvelteKit Dashboard (SPA)
├── website/            # VitePress Documentation & Landing Page
├── deploy/             # Installation & systemd scripts
├── Cargo.toml          # Root Workspace
└── docker-compose.yml  # Local development setup
```

## 🚀 Quick Start

To get started immediately on your Linux server, run our auto-installer:

```bash
curl -fsSL https://raw.githubusercontent.com/gmedia/labuh/main/deploy/quick-install.sh | sudo bash
```

For manual setup and development instructions, refer to the [Installation Guide](https://gmedia.github.io/labuh/guide/installation.html).

## 🛠️ Tech Stack

- **Backend**: Rust (Axum, SQLx, Bollard)
- **Frontend**: Svelte 5, TailwindCSS v4, shadcn-svelte
- **Proxy**: Caddy (Automatic SSL)
- **Database**: SQLite

## 📝 License

Distributed under the MIT License. See `LICENSE` for more information.

---
### 👤 Author & Maintainer
* **Original Creator:** [HasanH47](https://github.com/HasanH47)
* **Published by:** [Gmedia](https://github.com/gmedia)

Licensed under the [MIT License](LICENSE).
