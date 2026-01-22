#!/bin/bash
# Backup Script for Labuh

set -e

BACKUP_DIR="${BACKUP_DIR:-/var/backups/labuh}"
LABUH_DIR="${LABUH_DIR:-/opt/labuh}"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/labuh_backup_$DATE.tar.gz"

echo "🔄 Starting Labuh backup..."

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Stop service during backup for consistency
echo "⏸️  Stopping Labuh service..."
systemctl stop labuh 2>/dev/null || true

# Create backup
echo "📦 Creating backup archive..."
tar -czf "$BACKUP_FILE" -C "$LABUH_DIR" labuh.db .env 2>/dev/null || {
    tar -czf "$BACKUP_FILE" -C "$LABUH_DIR" labuh.db 2>/dev/null
}

# Restart service
echo "▶️  Starting Labuh service..."
systemctl start labuh 2>/dev/null || true

# Keep only last 7 backups
echo "🧹 Cleaning old backups..."
cd "$BACKUP_DIR" && ls -t labuh_backup_*.tar.gz | tail -n +8 | xargs -r rm

echo "✅ Backup completed: $BACKUP_FILE"
echo "   Size: $(du -h "$BACKUP_FILE" | cut -f1)"

# List recent backups
echo ""
echo "📋 Recent backups:"
ls -lh "$BACKUP_DIR"/labuh_backup_*.tar.gz 2>/dev/null | tail -5
