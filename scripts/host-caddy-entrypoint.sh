#!/bin/sh
# Use this script as the container's entrypoint when running openshelf
# behind a host-level Caddy (or any external reverse proxy that terminates
# TLS). The image's default entrypoint always sets FRONTEND_URL based on
# the DOMAIN env var, which is wrong when the public origin is the host
# Caddy, not the in-container Caddy.
#
# This script:
#   1. Uses an externally-set FRONTEND_URL if provided (set it in the
#      docker-compose environment block to the public origin, e.g.
#      https://books.example.com).
#   2. Falls back to DOMAIN-based FRONTEND_URL if FRONTEND_URL is unset.
#   3. Does NOT modify the in-container Caddyfile, so Caddy stays on
#      plain HTTP at :8080 and the host Caddy handles TLS.
#
# Mount this into the container and override the entrypoint in
# docker-compose:
#
#   volumes:
#     - ./host-caddy-entrypoint.sh:/custom-entrypoint.sh:ro
#   entrypoint: ["/custom-entrypoint.sh"]
#
# Other apps with the same pattern (e.g. openslate) need a copy of this
# script with their own backend binary name (/usr/local/bin/<binary>).

set -e

trap 'kill $API_PID 2>/dev/null; exit' TERM INT

if [ -z "$FRONTEND_URL" ]; then
  if [ -n "$DOMAIN" ]; then
    export FRONTEND_URL="https://${DOMAIN}"
  else
    export FRONTEND_URL="http://localhost:8080"
  fi
fi

echo "FRONTEND_URL=${FRONTEND_URL}"
echo "Starting API server..."

/usr/local/bin/openshelf-backend &
API_PID=$!

echo "Waiting for API server..."
for i in $(seq 1 30); do
  if curl -sf http://localhost:3001/api/health > /dev/null 2>&1; then
    echo "API server ready"
    break
  fi
  sleep 1
done

echo "Starting Caddy..."
exec /usr/local/bin/caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
