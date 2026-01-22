#!/bin/bash
# Restore Script for Labuh

set -e

BACKUP_DIR="${BACKUP_DIR:-/var/backups/labuh}"
LABUH_DIR="${LABUH_DIR:-/opt/labuh}"

# Check if backup file is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <backup_file>"
    echo ""
    echo "Available backups:"
    ls -lh "$BACKUP_DIR"/labuh_backup_*.tar.gz 2>/dev/null || echo "  No backups found in $BACKUP_DIR"
    exit 1
fi

BACKUP_FILE="$1"

# Check if backup exists
if [ ! -f "$BACKUP_FILE" ]; then
    echo "❌ Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "🔄 Restoring Labuh from backup..."
echo "   Backup: $BACKUP_FILE"
echo ""
read -p "⚠️  This will overwrite current data. Continue? [y/N] " confirm
if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
    echo "Cancelled."
    exit 0
fi

# Stop service
echo "⏸️  Stopping Labuh service..."
systemctl stop labuh 2>/dev/null || true

# Backup current database (just in case)
if [ -f "$LABUH_DIR/labuh.db" ]; then
    echo "📦 Backing up current database..."
    cp "$LABUH_DIR/labuh.db" "$LABUH_DIR/labuh.db.pre-restore"
fi

# Extract backup
echo "📂 Extracting backup..."
tar -xzf "$BACKUP_FILE" -C "$LABUH_DIR"

# Set permissions
echo "🔒 Setting permissions..."
chown -R labuh:labuh "$LABUH_DIR" 2>/dev/null || true

# Restart service
echo "▶️  Starting Labuh service..."
systemctl start labuh 2>/dev/null || true

echo ""
echo "✅ Restore completed!"
echo "   Previous database saved as: labuh.db.pre-restore"
