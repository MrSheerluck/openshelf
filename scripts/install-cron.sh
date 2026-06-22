#!/usr/bin/env bash
set -euo pipefail

# Installs a cron job that pulls the latest OpenShelf images from GHCR and
# restarts the containers every 2 minutes. Use this on a VPS that's already
# running OpenShelf via docker compose.
#
# Usage:
#   ./scripts/install-cron.sh
#
# Idempotent: running it again replaces the existing entry rather than
# duplicating it. Run `./scripts/uninstall-cron.sh` (or `crontab -e`) to remove.

APP_DIR="${OPENSHELF_DIR:-/opt/openshelf}"
CRON_LINE="*/2 * * * * cd ${APP_DIR} && docker compose pull && docker compose up -d >> /var/log/openshelf-update.log 2>&1"

echo "Installing cron job to update OpenShelf every 2 minutes"
echo "  App directory: ${APP_DIR}"
echo "  Cron line:     ${CRON_LINE}"
echo

if [ ! -d "${APP_DIR}" ]; then
  echo "Error: ${APP_DIR} does not exist." >&2
  echo "Clone the repo there first, or set OPENSHELF_DIR=/path/to/openshelf" >&2
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "Error: docker is not installed or not on PATH." >&2
  exit 1
fi

if ! command -v crontab >/dev/null 2>&1; then
  echo "Error: crontab is not installed. Install it with: apt install -y cron" >&2
  exit 1
fi

tmp="$(mktemp)"
existing="$(crontab -l 2>/dev/null || true)"

# Match the OpenShelf cron line specifically (by the app dir we install to),
# so we never accidentally clobber other apps' cron entries.
if echo "${existing}" | grep -Fq "cd ${APP_DIR}"; then
  echo "Existing OpenShelf cron job found — replacing it."
  filtered="$(echo "${existing}" | grep -Fv "cd ${APP_DIR}" || true)"
else
  filtered="${existing}"
fi

{
  echo "${filtered}"
  echo "${CRON_LINE}"
} | grep -v '^$' > "${tmp}"

crontab "${tmp}"
rm -f "${tmp}"

touch /var/log/openshelf-update.log
chmod 644 /var/log/openshelf-update.log

echo
echo "Done. Verify with: crontab -l"
echo "Logs go to:       /var/log/openshelf-update.log"
echo
echo "Note: this cron job only works once docker-compose.yml uses GHCR images"
echo "instead of build: directives. See docs/deployment.md for that step."
