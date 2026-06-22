#!/usr/bin/env bash
set -euo pipefail

# Removes the OpenShelf auto-update cron job installed by install-cron.sh.
# Idempotent: safe to run when the cron job is already absent.

existing="$(crontab -l 2>/dev/null || true)"

if ! echo "${existing}" | grep -Fq "docker compose pull"; then
  echo "No OpenShelf cron job installed. Nothing to do."
  exit 0
fi

tmp="$(mktemp)"
echo "${existing}" | grep -Fv "docker compose pull" | grep -v '^$' > "${tmp}" || true

if [ -s "${tmp}" ]; then
  crontab "${tmp}"
else
  crontab -r 2>/dev/null || true
fi
rm -f "${tmp}"

echo "Removed OpenShelf cron job. Verify with: crontab -l"
