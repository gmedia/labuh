#!/bin/bash
# Labuh - Update Script
# Usage: sudo bash /opt/labuh/update.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

GITHUB_REPO="gmedia/labuh"
RAW_URL="https://raw.githubusercontent.com/${GITHUB_REPO}/main"
INSTALL_DIR="/opt/labuh"
LABUH_USER="labuh"

echo -e "${BLUE}Updating Labuh PaaS...${NC}"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   exit 1
fi

# Detect architecture
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

echo -e "${YELLOW}Downloading latest release for ${ARCH}...${NC}"

DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/labuh-linux-${ARCH}.tar.gz"
TMP_DIR=$(mktemp -d)

curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/labuh.tar.gz"
tar -xzf "${TMP_DIR}/labuh.tar.gz" -C "$TMP_DIR"

if [[ ! -f "${TMP_DIR}/labuh" ]]; then
    echo -e "${RED}Error: Downloaded archive does not contain 'labuh' binary${NC}"
    rm -rf "$TMP_DIR"
    exit 1
fi

echo -e "${YELLOW}Stopping labuh service...${NC}"
systemctl stop labuh || true

echo -e "${YELLOW}Replacing binary and updating components...${NC}"
cp "${TMP_DIR}/labuh" "$INSTALL_DIR/labuh"
chmod +x "$INSTALL_DIR/labuh"

# Update migrations
if [[ -d "${TMP_DIR}/migrations" ]]; then
    echo -e "${YELLOW}Updating database migrations...${NC}"
    rm -rf "$INSTALL_DIR/migrations"
    cp -r "${TMP_DIR}/migrations" "$INSTALL_DIR/"
fi

# Update frontend
if [[ -d "${TMP_DIR}/frontend" ]]; then
    echo -e "${YELLOW}Updating dashboard frontend...${NC}"
    rm -rf "$INSTALL_DIR/frontend"
    cp -r "${TMP_DIR}/frontend" "$INSTALL_DIR/"
fi

chown -R "$LABUH_USER:$LABUH_USER" "$INSTALL_DIR/labuh" "$INSTALL_DIR/migrations" "$INSTALL_DIR/frontend" 2>/dev/null || true

echo -e "${YELLOW}Updating helper scripts and service configuration...${NC}"
curl -fsSL "${RAW_URL}/deploy/backup.sh" -o "$INSTALL_DIR/backup.sh"
curl -fsSL "${RAW_URL}/deploy/restore.sh" -o "$INSTALL_DIR/restore.sh"
curl -fsSL "${RAW_URL}/deploy/labuh.service" -o "/etc/systemd/system/labuh.service"

chmod +x "$INSTALL_DIR/backup.sh" "$INSTALL_DIR/restore.sh"
chown "$LABUH_USER:$LABUH_USER" "$INSTALL_DIR/backup.sh" "$INSTALL_DIR/restore.sh"

echo -e "${YELLOW}Reloading systemd daemon...${NC}"
systemctl daemon-reload

echo -e "${YELLOW}Starting labuh service...${NC}"
systemctl start labuh

rm -rf "$TMP_DIR"

echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}✓ Labuh updated successfully!${NC}"
echo -e "${GREEN}================================${NC}"
systemctl status labuh --no-pager
