#!/usr/bin/env bash
set -euo pipefail

# Chatalot backup script
# SSHes into a remote server running Chatalot, dumps the PostgreSQL database,
# pulls it to a local backup directory, and optionally syncs uploaded files.
#
# Designed to run from a machine that can SSH to the Chatalot server (e.g. local
# prod pulling from a public VPS, or a workstation pulling from any server).
#
# Configuration (set in environment or scripts/deploy.env):
#   SOURCE_HOST     - SSH target running Chatalot (e.g. root@chatalot.seglamater.app)
#   SOURCE_DIR      - Remote Chatalot directory (e.g. /srv/chatalot)
#   BACKUP_DIR      - Local directory to store backups (e.g. /srv/ab9/chatalot_bak)
#   BACKUP_KEEP     - Number of DB backups to retain (default: 30)
#
# Usage:
#   ./scripts/backup.sh              # Full backup (DB + files)
#   ./scripts/backup.sh --db-only    # Database only
#   ./scripts/backup.sh --list       # List existing backups
#
# Cron example (daily at 3am):
#   0 3 * * * SOURCE_HOST=root@chatalot.seglamater.app BACKUP_DIR=/srv/ab9/chatalot_bak /srv/ab9/chatalot/scripts/backup.sh --db-only >> /var/log/chatalot-backup.log 2>&1

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Load config from deploy.env if present
if [[ -f "$SCRIPT_DIR/deploy.env" ]]; then
    # shellcheck source=/dev/null
    source "$SCRIPT_DIR/deploy.env"
fi

SOURCE_HOST="${SOURCE_HOST:?Set SOURCE_HOST (e.g. root@chatalot.seglamater.app)}"
SOURCE_DIR="${SOURCE_DIR:-/srv/chatalot}"
BACKUP_DIR="${BACKUP_DIR:?Set BACKUP_DIR (e.g. /srv/ab9/chatalot_bak)}"
BACKUP_KEEP="${BACKUP_KEEP:-30}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[backup]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*"; }
warn()  { echo -e "${YELLOW}[backup]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*"; }
error() { echo -e "${RED}[backup]${NC} $(date '+%Y-%m-%d %H:%M:%S') $*" >&2; }

# --- List mode ---
if [[ "${1:-}" == "--list" ]]; then
    echo "Backups in ${BACKUP_DIR}/db/:"
    ls -lh "${BACKUP_DIR}/db/"chatalot-db-*.sql.gz 2>/dev/null || echo "  (none)"
    echo
    if [[ -d "${BACKUP_DIR}/files" ]]; then
        FILE_SIZE=$(du -sh "${BACKUP_DIR}/files" 2>/dev/null | cut -f1)
        echo "File backup: ${FILE_SIZE}"
    fi
    exit 0
fi

# --- Backup mode ---

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
DB_DUMP="chatalot-db-${TIMESTAMP}.sql.gz"

# Ensure local backup dirs exist
mkdir -p "${BACKUP_DIR}/db" "${BACKUP_DIR}/files"

# Step 1: Dump database on remote server and pull it
info "Dumping database on ${SOURCE_HOST}..."
ssh "$SOURCE_HOST" "docker exec chatalot-db pg_dump -U chatalot --format=custom chatalot" | gzip > "${BACKUP_DIR}/db/${DB_DUMP}"
DB_SIZE=$(du -h "${BACKUP_DIR}/db/${DB_DUMP}" | cut -f1)
info "Database dump: ${DB_SIZE} -> ${BACKUP_DIR}/db/${DB_DUMP}"

# Step 2: Sync uploaded files (unless --db-only)
if [[ "${1:-}" != "--db-only" ]]; then
    # Find the file storage volume path on the remote
    FILE_VOL=$(ssh "$SOURCE_HOST" "docker inspect chatalot-server --format '{{range .Mounts}}{{if eq .Destination \"/app/data/files\"}}{{.Source}}{{end}}{{end}}'" 2>/dev/null || true)
    if [[ -n "$FILE_VOL" ]]; then
        info "Syncing uploaded files from ${SOURCE_HOST}:${FILE_VOL}..."
        rsync -az --delete -e ssh "${SOURCE_HOST}:${FILE_VOL}/" "${BACKUP_DIR}/files/"
        FILE_SIZE=$(du -sh "${BACKUP_DIR}/files" 2>/dev/null | cut -f1)
        info "File sync complete (${FILE_SIZE})"
    else
        warn "Could not find file storage volume on remote, skipping file backup."
    fi
fi

# Step 3: Prune old DB backups (keep last N)
CURRENT_COUNT=$(ls -1 "${BACKUP_DIR}/db/"chatalot-db-*.sql.gz 2>/dev/null | wc -l)
if [[ "$CURRENT_COUNT" -gt "$BACKUP_KEEP" ]]; then
    PRUNE_COUNT=$((CURRENT_COUNT - BACKUP_KEEP))
    info "Pruning ${PRUNE_COUNT} old backup(s) (keeping last ${BACKUP_KEEP})..."
    ls -1t "${BACKUP_DIR}/db/"chatalot-db-*.sql.gz | tail -n +"$((BACKUP_KEEP + 1))" | xargs rm -v 2>&1 | while read -r line; do info "  removed: $line"; done
fi

# Step 4: Summary
FINAL_COUNT=$(ls -1 "${BACKUP_DIR}/db/"chatalot-db-*.sql.gz 2>/dev/null | wc -l)
TOTAL_SIZE=$(du -sh "${BACKUP_DIR}" 2>/dev/null | cut -f1)
info "Backup complete! ${FINAL_COUNT} DB backup(s), total size: ${TOTAL_SIZE}"
