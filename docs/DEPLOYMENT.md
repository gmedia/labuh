# Labuh - Deployment Guide

## Prerequisites

- Linux server (Ubuntu 20.04+ recommended)
- Docker or containerd installed
- 1GB RAM minimum (2GB recommended)
- Rust toolchain (for building from source)

## Option 1: Docker Compose (Recommended)

### Quick Start

```bash
# Clone repository
git clone https://github.com/HasanH47/labuh.git
cd labuh

# Configure environment
cp .env.example .env
nano .env  # Edit JWT_SECRET and other settings

# Start services
docker-compose up -d

# Check status
docker-compose ps
```

### Services Started

| Service  | Port    | Description   |
| -------- | ------- | ------------- |
| labuh    | 3000    | Backend API   |
| caddy    | 80, 443 | Reverse proxy |
| frontend | 5173    | Web UI        |

### Accessing the Dashboard

- Open `http://your-server-ip` in browser
- Register the first admin user
- Start managing containers!

## Option 2: Systemd Service

### Build from Source

```bash
# Clone and build
git clone https://github.com/HasanH47/labuh.git
cd labuh
cargo build --release

# Run install script
sudo ./deploy/install.sh
```

### Configure

```bash
# Edit configuration
sudo nano /opt/labuh/.env
```

Required settings:

```env
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite:/opt/labuh/labuh.db?mode=rwc
JWT_SECRET=your-super-secret-key-change-this
JWT_EXPIRATION_HOURS=24
CADDY_ADMIN_API=http://localhost:2019
```

### Start Service

```bash
sudo systemctl start labuh
sudo systemctl enable labuh  # Enable on boot
```

### Check Logs

```bash
sudo journalctl -u labuh -f
```

## Option 3: Manual Run

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/labuh
```

## Reverse Proxy Setup (Caddy)

### Install Caddy

```bash
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

### Configure Caddyfile

```caddyfile
# /etc/caddy/Caddyfile
your-domain.com {
    reverse_proxy localhost:3000

    handle /dashboard/* {
        reverse_proxy localhost:5173
    }
}
```

### Reload Caddy

```bash
sudo systemctl reload caddy
```

## Firewall Setup

```bash
# Allow required ports
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 3000/tcp  # Optional if behind reverse proxy
```

## Troubleshooting

### Container Runtime Not Found

Ensure Docker is running:

```bash
sudo systemctl status docker
```

### Database Connection Failed

Check permissions:

```bash
sudo chown labuh:labuh /opt/labuh/labuh.db
```

### Port Already in Use

Check what's using the port:

```bash
sudo ss -tulpn | grep :3000
```

## Backup & Restore

### Backup

```bash
# Backup database
cp /opt/labuh/labuh.db /backups/labuh-$(date +%Y%m%d).db

# Or with systemd running
sqlite3 /opt/labuh/labuh.db ".backup '/backups/labuh.db'"
```

### Restore

```bash
sudo systemctl stop labuh
cp /backups/labuh-20260121.db /opt/labuh/labuh.db
sudo chown labuh:labuh /opt/labuh/labuh.db
sudo systemctl start labuh
```
