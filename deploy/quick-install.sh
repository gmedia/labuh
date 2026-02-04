#!/bin/bash
# Labuh - Quick Install Script
# Usage: curl -fsSL https://raw.githubusercontent.com/gmedia/labuh/main/deploy/quick-install.sh | bash
#
# Or with options:
# curl -fsSL ... | bash -s -- --runtime docker
# curl -fsSL ... | bash -s -- --runtime containerd

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Defaults
RUNTIME=""
LABUH_VERSION="latest"
INSTALL_DIR="/opt/labuh"
LABUH_USER="labuh"
GITHUB_REPO="gmedia/labuh"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --runtime)
            RUNTIME="$2"
            shift 2
            ;;
        --version)
            LABUH_VERSION="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}"
echo "  _          _           _     "
echo " | |    __ _| |__  _   _| |__  "
echo " | |   / _\` | '_ \| | | | '_ \ "
echo " | |__| (_| | |_) | |_| | | | |"
echo " |_____\__,_|_.__/ \__,_|_| |_|"
echo -e "${NC}"
echo "Lightweight PaaS Platform Installer"
echo "===================================="
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   exit 1
fi

# Detect OS
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
        OS_VERSION=$VERSION_ID
    elif [ -f /etc/redhat-release ]; then
        OS="rhel"
    else
        OS=$(uname -s)
    fi

    echo -e "${GREEN}✓ Detected OS: $OS $OS_VERSION${NC}"

    # Map for Docker repo URL
    case $OS in
        ubuntu|pop|mint|neon)
            DOCKER_OS="ubuntu"
            ;;
        debian|kali|raspbian)
            DOCKER_OS="debian"
            ;;
        fedora)
            DOCKER_OS="fedora"
            ;;
        centos|rhel|rocky)
            DOCKER_OS="centos"
            ;;
        *)
            if [[ "$ID_LIKE" == *"ubuntu"* ]]; then
                DOCKER_OS="ubuntu"
            elif [[ "$ID_LIKE" == *"debian"* ]]; then
                DOCKER_OS="debian"
            else
                DOCKER_OS="$OS"
            fi
            ;;
    esac
}

# Detect architecture
detect_arch() {
    ARCH=$(uname -m)
    case $ARCH in
        x86_64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac
    echo -e "${GREEN}✓ Architecture: $ARCH${NC}"
}

# Install essential dependencies
install_dependencies() {
    echo -e "${YELLOW}Installing base dependencies...${NC}"

    case $OS in
        ubuntu|debian)
            apt-get update
            apt-get install -y openssl curl ca-certificates tar gzip gnupg2 lsb-release
            ;;
        fedora|rhel|centos|rocky)
            dnf install -y openssl curl ca-certificates tar gzip gnupg2
            ;;
        *)
            echo -e "${YELLOW}Warning: OS $OS not explicitly supported for dependency auto-install.${NC}"
            echo -e "Ensuring openssl and curl are available..."
            if ! command -v openssl &> /dev/null || ! command -v curl &> /dev/null; then
                echo -e "${RED}Error: Required tools (openssl, curl) missing. Please install them manually.${NC}"
                exit 1
            fi
            ;;
    esac
    echo -e "${GREEN}✓ Base dependencies installed${NC}"
}

# Check if Docker is installed
check_docker() {
    if command -v docker &> /dev/null; then
        return 0
    fi
    return 1
}

# Install Docker
install_docker() {
    echo -e "${YELLOW}Installing Docker...${NC}"

    case $OS in
        ubuntu|debian|kali|raspbian|pop|mint|neon)
            apt-get update
            apt-get install -y ca-certificates curl gnupg lsb-release
            install -m 0755 -d /etc/apt/keyrings
            curl -fsSL https://download.docker.com/linux/$DOCKER_OS/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg --yes
            chmod a+r /etc/apt/keyrings/docker.gpg

            # Use VERSION_CODENAME if available, fallback to lsb_release
            CODENAME=${VERSION_CODENAME:-$(lsb_release -cs)}

            echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/$DOCKER_OS $CODENAME stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
            apt-get update
            apt-get install -y docker-ce docker-ce-cli containerd.io
            ;;
        fedora|rhel|centos|rocky)
            dnf -y install dnf-plugins-core
            dnf config-manager --add-repo https://download.docker.com/linux/fedora/docker-ce.repo
            dnf install -y docker-ce docker-ce-cli containerd.io
            ;;
        *)
            echo -e "${RED}Unsupported OS for Docker auto-install. Please install Docker manually.${NC}"
            exit 1
            ;;
    esac

    systemctl enable docker
    systemctl start docker
    echo -e "${GREEN}✓ Docker installed and started${NC}"
}

# Prompt for runtime selection (simplified for Docker)
ensure_runtime() {
    if [[ -n "$RUNTIME" ]]; then
        return
    fi

    if check_docker; then
        RUNTIME="docker"
        echo -e "${GREEN}✓ Docker is already installed${NC}"
        return
    fi

    echo ""
    echo -e "${YELLOW}Docker is required for Labuh but not found.${NC}"
    echo "This script will now install Docker CE automatically."
    echo ""

    install_docker
    RUNTIME="docker"
}

# Download and install Labuh binary
install_labuh() {
    echo -e "${YELLOW}Downloading Labuh ${LABUH_VERSION}...${NC}"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download binary
    if [[ "$LABUH_VERSION" == "latest" ]]; then
        DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/labuh-linux-${ARCH}.tar.gz"
    else
        DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LABUH_VERSION}/labuh-linux-${ARCH}.tar.gz"
    fi

    curl -fsSL "$DOWNLOAD_URL" -o /tmp/labuh.tar.gz
    tar -xzf /tmp/labuh.tar.gz -C "$INSTALL_DIR"
    rm /tmp/labuh.tar.gz

    chmod +x "$INSTALL_DIR/labuh"

    echo -e "${GREEN}✓ Labuh installed to $INSTALL_DIR${NC}"
}

# Create systemd service
setup_systemd() {
    echo -e "${YELLOW}Setting up systemd service...${NC}"

    # Create user if not exists
    if ! id "$LABUH_USER" &>/dev/null; then
        useradd --system --no-create-home --shell /usr/sbin/nologin "$LABUH_USER"
    fi

    # Add user to docker group
    if [[ "$RUNTIME" == "docker" ]]; then
        usermod -aG docker "$LABUH_USER"
    fi

    # Create .env file
    if [[ ! -f "$INSTALL_DIR/.env" ]]; then
        JWT_SECRET=$(openssl rand -base64 32)
        cat > "$INSTALL_DIR/.env" << EOF
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite:$INSTALL_DIR/labuh.db?mode=rwc
JWT_SECRET=$JWT_SECRET
JWT_EXPIRATION_HOURS=24
CADDY_ADMIN_API=http://localhost:2019
BASE_DOMAIN=localhost
FRONTEND_DIR=$INSTALL_DIR/frontend/build
RUST_LOG=info
EOF
        chown "$LABUH_USER:$LABUH_USER" "$INSTALL_DIR/.env"
        chmod 600 "$INSTALL_DIR/.env"
    fi

    # Create Caddyfile if not exists or if it is a directory
    if [[ -d "$INSTALL_DIR/Caddyfile" ]]; then
        echo -e "${YELLOW}Warning: Caddyfile is a directory, removing...${NC}"
        rm -rf "$INSTALL_DIR/Caddyfile"
    fi

    if [[ ! -f "$INSTALL_DIR/Caddyfile" ]] || [[ ! -s "$INSTALL_DIR/Caddyfile" ]]; then
        cat > "$INSTALL_DIR/Caddyfile" << EOF
{
    admin 0.0.0.0:2019
}

:80 {
    handle /api/* {
        reverse_proxy labuh:3000
    }

    handle {
        reverse_proxy labuh:3000
    }
}
EOF
        chown "$LABUH_USER:$LABUH_USER" "$INSTALL_DIR/Caddyfile"
    fi

    # Create systemd service file
    cat > /etc/systemd/system/labuh.service << EOF
[Unit]
Description=Labuh PaaS Platform
After=network.target docker.service containerd.service
Wants=docker.service

[Service]
Type=simple
User=$LABUH_USER
Group=$LABUH_USER
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/.env
ExecStart=$INSTALL_DIR/labuh
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$INSTALL_DIR

[Install]
WantedBy=multi-user.target
EOF

    # Set ownership
    chown -R "$LABUH_USER:$LABUH_USER" "$INSTALL_DIR"

    # Reload and enable service
    systemctl daemon-reload
    systemctl enable labuh

    echo -e "${GREEN}✓ Systemd service configured${NC}"
}

# Main installation flow
main() {
    detect_os
    detect_arch
    install_dependencies

    echo ""
    echo "Checking container runtime..."
    ensure_runtime

    # Install Labuh
    install_labuh

    # Setup systemd
    setup_systemd

    echo ""
    echo -e "${GREEN}================================${NC}"
    echo -e "${GREEN}✓ Labuh installation complete!${NC}"
    echo -e "${GREEN}================================${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Edit configuration: sudo nano $INSTALL_DIR/.env"
    echo "  2. Start Labuh: sudo systemctl start labuh"
    echo "  3. Check status: sudo systemctl status labuh"
    echo "  4. View logs: sudo journalctl -u labuh -f"
    echo ""
    echo "Dashboard will be available at: http://localhost:3000"
    echo ""
}

main
