#!/bin/bash
# Labuh - Quick Install Script
# Usage: curl -fsSL https://raw.githubusercontent.com/gmedia/labuh/main/deploy/quick-install.sh | bash

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
RAW_URL="https://raw.githubusercontent.com/${GITHUB_REPO}/main"

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

# Detect existing installation
check_existing() {
    if [[ -f "$INSTALL_DIR/labuh" ]]; then
        echo -e "${YELLOW}Labuh is already installed in $INSTALL_DIR.${NC}"
        read -p "Do you want to update to the latest version instead? [Y/n] " -n 1 -r < /dev/tty
        echo ""
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            echo -e "${BLUE}Running update mechanism...${NC}"
            curl -fsSL "${RAW_URL}/deploy/update.sh" -o /tmp/labuh-update.sh
            bash /tmp/labuh-update.sh
            exit 0
        fi
        echo -e "${BLUE}Proceeding with fresh installation logic...${NC}"
    fi
}

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

    case $OS in
        ubuntu|pop|mint|neon) DOCKER_OS="ubuntu" ;;
        debian|kali|raspbian) DOCKER_OS="debian" ;;
        fedora) DOCKER_OS="fedora" ;;
        centos|rhel|rocky) DOCKER_OS="centos" ;;
        *)
            if [[ "$ID_LIKE" == *"ubuntu"* ]]; then DOCKER_OS="ubuntu"
            elif [[ "$ID_LIKE" == *"debian"* ]]; then DOCKER_OS="debian"
            else DOCKER_OS="$OS"
            fi
            ;;
    esac
}

# Detect architecture
detect_arch() {
    ARCH_RAW=$(uname -m)
    case $ARCH_RAW in
        x86_64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $ARCH_RAW${NC}"
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
            if ! command -v openssl &> /dev/null || ! command -v curl &> /dev/null; then
                echo -e "${RED}Error: Required tools (openssl, curl) missing.${NC}"
                exit 1
            fi
            ;;
    esac
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
    esac
    systemctl enable docker
    systemctl start docker
    echo -e "${GREEN}✓ Docker installed and started${NC}"
}

ensure_runtime() {
    if command -v docker &> /dev/null; then
        RUNTIME="docker"
        echo -e "${GREEN}✓ Docker is already installed${NC}"
    else
        echo -e "${YELLOW}Docker is required for Labuh but not found. Installing...${NC}"
        install_docker
        RUNTIME="docker"
    fi
}

install_labuh() {
    echo -e "${YELLOW}Downloading Labuh ${LABUH_VERSION}...${NC}"
    mkdir -p "$INSTALL_DIR"
    if [[ "$LABUH_VERSION" == "latest" ]]; then
        DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/labuh-linux-${ARCH}.tar.gz"
    else
        DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LABUH_VERSION}/labuh-linux-${ARCH}.tar.gz"
    fi
    curl -fsSL "$DOWNLOAD_URL" -o /tmp/labuh.tar.gz
    # Clean up old frontend to avoid stale files
    rm -rf "$INSTALL_DIR/frontend"
    rm -rf "$INSTALL_DIR/migrations"
    tar -xzf /tmp/labuh.tar.gz -C "$INSTALL_DIR"
    rm /tmp/labuh.tar.gz
    chmod +x "$INSTALL_DIR/labuh"
}

setup_systemd() {
    echo -e "${YELLOW}Setting up systemd service and components...${NC}"

    # User Management
    if ! id "$LABUH_USER" &>/dev/null; then
        useradd --system --create-home --home-dir /home/$LABUH_USER --shell /bin/sh "$LABUH_USER"
    else
        usermod -s /bin/sh "$LABUH_USER"
        mkdir -p /home/$LABUH_USER
        chown "$LABUH_USER:$LABUH_USER" /home/$LABUH_USER
    fi
    [[ "$RUNTIME" == "docker" ]] && usermod -aG docker "$LABUH_USER"

    # Helper Scripts
    echo -e "${YELLOW}Downloading helper scripts...${NC}"
    for script in backup.sh restore.sh update.sh; do
        curl -fsSL "${RAW_URL}/deploy/${script}" -o "$INSTALL_DIR/${script}"
        chmod +x "$INSTALL_DIR/${script}"
    done

    # Environment
    if [[ ! -f "$INSTALL_DIR/.env" ]]; then
        echo -e "${YELLOW}Creating .env from example...${NC}"
        curl -fsSL "${RAW_URL}/backend/.env.example" -o "$INSTALL_DIR/.env"
        JWT_SECRET=$(openssl rand -base64 32)
        sed -i "s|JWT_SECRET=.*|JWT_SECRET=${JWT_SECRET}|" "$INSTALL_DIR/.env"
        sed -i "s|DATABASE_URL=.*|DATABASE_URL=sqlite:${INSTALL_DIR}/labuh.db?mode=rwc|" "$INSTALL_DIR/.env"
        sed -i "s|FRONTEND_DIR=.*|FRONTEND_DIR=${INSTALL_DIR}/frontend/build|" "$INSTALL_DIR/.env"
        chown "$LABUH_USER:$LABUH_USER" "$INSTALL_DIR/.env"
        chmod 600 "$INSTALL_DIR/.env"
    fi

    # Caddy
    if [[ ! -f "$INSTALL_DIR/Caddyfile" ]] || [[ ! -s "$INSTALL_DIR/Caddyfile" ]]; then
        echo -e "${YELLOW}Downloading default Caddyfile...${NC}"
        curl -fsSL "${RAW_URL}/deploy/Caddyfile" -o "$INSTALL_DIR/Caddyfile"
    fi

    # Service
    echo -e "${YELLOW}Configuring systemd service...${NC}"
    curl -fsSL "${RAW_URL}/deploy/labuh.service" -o /etc/systemd/system/labuh.service
    if [[ "$INSTALL_DIR" != "/opt/labuh" ]]; then
        sed -i "s|/opt/labuh|${INSTALL_DIR}|g" /etc/systemd/system/labuh.service
    fi

    chown -R "$LABUH_USER:$LABUH_USER" "$INSTALL_DIR"
    systemctl daemon-reload
    systemctl enable labuh
    echo -e "${GREEN}✓ Systemd service and components configured${NC}"
}

main() {
    detect_os
    detect_arch
    check_existing

    install_dependencies
    ensure_runtime
    install_labuh
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
