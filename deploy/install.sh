#!/bin/bash
set -e

# Labuh Installation Script
# This script installs Labuh as a systemd service

INSTALL_DIR="/opt/labuh"
SERVICE_USER="labuh"
SERVICE_GROUP="labuh"

echo "🚀 Installing Labuh..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root (sudo)"
    exit 1
fi

# Create user and group
if ! id "$SERVICE_USER" &>/dev/null; then
    echo "Creating user $SERVICE_USER..."
    useradd --system --shell /bin/false --home-dir "$INSTALL_DIR" "$SERVICE_USER"
fi

# Create installation directory
echo "Creating installation directory..."
mkdir -p "$INSTALL_DIR"

# Copy binary
if [ -f "./target/release/labuh" ]; then
    echo "Copying release binary..."
    cp ./target/release/labuh "$INSTALL_DIR/"
elif [ -f "./labuh" ]; then
    echo "Copying binary..."
    cp ./labuh "$INSTALL_DIR/"
else
    echo "Error: Binary not found. Please build first with 'cargo build --release'"
    exit 1
fi

# Copy .env file if exists
if [ -f "./.env" ]; then
    echo "Copying .env file..."
    cp ./.env "$INSTALL_DIR/.env"
elif [ -f "./.env.example" ]; then
    echo "Copying .env.example as .env..."
    cp ./.env.example "$INSTALL_DIR/.env"
    echo "⚠️  Please edit /opt/labuh/.env with your configuration"
fi

# Set permissions
echo "Setting permissions..."
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR"
chmod 755 "$INSTALL_DIR/labuh"
chmod 600 "$INSTALL_DIR/.env"

# Copy systemd service
echo "Installing systemd service..."
cp ./deploy/labuh.service /etc/systemd/system/

# Reload systemd
systemctl daemon-reload

# Enable service
systemctl enable labuh

echo ""
echo "✅ Labuh installed successfully!"
echo ""
echo "To start the service:"
echo "  sudo systemctl start labuh"
echo ""
echo "To check status:"
echo "  sudo systemctl status labuh"
echo ""
echo "To view logs:"
echo "  sudo journalctl -u labuh -f"
echo ""
echo "Configuration file: $INSTALL_DIR/.env"
