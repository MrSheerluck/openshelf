#!/usr/bin/env bash
set -euo pipefail

# Run this on your VPS after starting garage:
#   cd /opt/garage && docker compose up -d
#   cd /opt/openshelf && bash scripts/setup-garage-bucket.sh

CONTAINER="garage"
BUCKET="${S3_BUCKET:-openshelf}"
KEY_NAME="openshelf-key"

echo "Creating bucket '$BUCKET'..."
docker exec "$CONTAINER" /garage bucket create "$BUCKET" 2>/dev/null || echo "  (bucket may already exist)"

echo "Creating API key '$KEY_NAME'..."
KEY_OUTPUT=$(docker exec "$CONTAINER" /garage key create "$KEY_NAME" 2>/dev/null)
if [ -z "$KEY_OUTPUT" ]; then
  echo "  (key may already exist, fetching...)"
  KEY_OUTPUT=$(docker exec "$CONTAINER" /garage key info "$KEY_NAME")
fi

echo "Granting read/write/owner on '$BUCKET' to '$KEY_NAME'..."
docker exec "$CONTAINER" /garage bucket allow --read --write --owner "$BUCKET" --key "$KEY_NAME"

ACCESS_KEY=$(echo "$KEY_OUTPUT" | grep "Key ID:" | awk '{print $3}')
SECRET_KEY=$(echo "$KEY_OUTPUT" | grep "Secret key:" | awk '{print $3}')

echo ""
echo "============================================"
echo "Add these to your openshelf .env file:"
echo "============================================"
echo "S3_BUCKET=$BUCKET"
echo "S3_ENDPOINT=http://garage:3900"
echo "S3_REGION=garage"
echo "S3_ACCESS_KEY=$ACCESS_KEY"
echo "S3_SECRET_KEY=$SECRET_KEY"
echo "============================================"
