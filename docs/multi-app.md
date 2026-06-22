# Multiple apps on one host

The default [deployment guide](deployment.md) assumes openshelf is the only app on the VPS. If you already have another app running (for example [openslate](https://github.com/MrSheerluck/openslate)) and want openshelf to share the box, you need this pattern.

## When you need this

- You want both apps reachable at `https://<app>.yourdomain.com` (no port in the URL)
- You do not want to pay for a second VPS
- The first app already binds ports 80 and 443 (every app's "easy" setup does this), so openshelf cannot do the same

## The pattern

A reverse proxy on the host terminates TLS for every subdomain, and each app runs on its own private localhost port with plain HTTP.

```
[Client] --HTTPS--> [Host Caddy :80/:443]
                          |
                          |-- notes.sheerluck.dev ---> [openslate :8080]
                          |
                          +-- books.sheerluck.dev ---> [openshelf :8443]
                                                          (openshelf's in-container Caddy
                                                           serves plain HTTP at :8080)
```

The in-container Caddy inside each app stops doing TLS. The host Caddy takes over.

## Setup

### 1. Install Caddy on the host

```bash
apt install -y caddy
```

### 2. Write the host Caddyfile

```caddy
# /etc/caddy/Caddyfile

<first-app>.yourdomain.com {
    reverse_proxy localhost:8080
}

<second-app>.yourdomain.com {
    reverse_proxy localhost:8443
}
```

Adjust the host ports to match what you set in each app's `docker-compose.yml`. Caddy auto-provisions a Let's Encrypt cert per subdomain.

### 3. Configure each app to serve plain HTTP

For openshelf, the in-container Caddy stays on `:8080` with plain HTTP. Two things change in `docker-compose.yml`:

- Set `DOMAIN=` (empty) so the in-container Caddy does not try ACME
- Mount [`scripts/host-caddy-entrypoint.sh`](../scripts/host-caddy-entrypoint.sh) as the entrypoint, and set `FRONTEND_URL` to the public origin

```yaml
services:
  openshelf:
    image: ghcr.io/mrsheerluck/openshelf:latest
    ports:
      - "8443:8080"   # host port :8443 -> container's in-container Caddy :8080
    volumes:
      - data:/data
      - ./host-caddy-entrypoint.sh:/custom-entrypoint.sh:ro
    env_file:
      - .env
    environment:
      - JWT_SECRET=${JWT_SECRET}
      - DOMAIN=
      - FRONTEND_URL=https://books.yourdomain.com   # must match the public origin
    entrypoint: ["/custom-entrypoint.sh"]
    restart: unless-stopped

volumes:
  data:
```

The same applies to the other app (for openslate, the binary name is `/usr/local/bin/api` instead of `/usr/local/bin/openshelf-backend`, so make a copy of the script with that one line changed).

### 4. Verify

```bash
systemctl reload caddy
sleep 3
curl -sI https://<first-app>.yourdomain.com | head -2
curl -sI https://<second-app>.yourdomain.com | head -2
```

Both should return `HTTP/2 200` with a real Let's Encrypt cert.

## Gotchas

### `FRONTEND_URL` must match the public origin

The backend uses `FRONTEND_URL` to set the CORS `Access-Control-Allow-Origin` header. If you forget to set it, or set it to `http://localhost:8080`, the browser blocks every cross-origin request with a CORS error. The `host-caddy-entrypoint.sh` script picks up `FRONTEND_URL` from the docker-compose environment block, so you have to set it there.

### Each app needs its own host port

The host Caddy proxies by hostname, but it still needs a unique port to forward to. Pick any unused port; `8080` and `8443` are conventional but not required.

### The openslate image has `80:80` and `443:443` baked in

If you keep the openslate cron job (`docker compose pull openslate && docker compose up -d openslate`), the next auto-update will try to bind those ports and fail because the host Caddy owns them. Patch the cron to strip the conflicting port mappings on every run:

```cron
*/2 * * * * cd /opt/openslate && sed -i '/- "80:80"/d; /- "443:443"/d' docker-compose.yml && docker compose pull openslate && docker compose up -d openslate
```

openshelf's image does not have those port mappings, so its cron does not need this patch.

### The custom entrypoint is a workaround

`host-caddy-entrypoint.sh` is a volume-mount override. If a future openshelf image update changes the entrypoint shape, the override keeps working (it is a self-contained script, not a patch on the image's entrypoint). The proper long-term fix is to PR a smarter entrypoint into both repos that natively respects `FRONTEND_URL` and skips the Caddyfile rewrite when `DOMAIN` is empty. Until that lands, the volume-mount approach is the cleanest workaround.

## Worked example: openslate + openshelf on Hetzner

This is the exact configuration from the live setup at the time of writing.

**Host ports:**
- openslate: `8080:8080`
- openshelf: `8443:8080`

**Host Caddyfile (`/etc/caddy/Caddyfile`):**

```caddy
notes.sheerluck.dev {
    reverse_proxy localhost:8080
}

books.sheerluck.dev {
    reverse_proxy localhost:8443
}
```

**openslate `docker-compose.yml`:**

```yaml
services:
  openslate:
    image: ghcr.io/mrsheerluck/openslate:latest
    ports:
      - "8080:8080"
    volumes:
      - data:/data
      - ./host-caddy-entrypoint.sh:/custom-entrypoint.sh:ro
    env_file:
      - .env
    environment:
      - JWT_SECRET=${JWT_SECRET}
      - DOMAIN=
      - FRONTEND_URL=https://notes.sheerluck.dev
      - R2_BUCKET=${R2_BUCKET}
      - R2_ACCOUNT_ID=${R2_ACCOUNT_ID}
      - R2_ACCESS_KEY=${R2_ACCESS_KEY}
      - R2_SECRET_KEY=${R2_SECRET_KEY}
    entrypoint: ["/custom-entrypoint.sh"]
    restart: unless-stopped

volumes:
  data:
```

(For openslate specifically, the `host-caddy-entrypoint.sh` needs `/usr/local/bin/api` instead of `/usr/local/bin/openshelf-backend` on the line that starts the backend.)

**openshelf `docker-compose.yml`:** see step 3 above.

**Crontab:**

```cron
*/2 * * * * cd /opt/openslate && sed -i '/- "80:80"/d; /- "443:443"/d' docker-compose.yml && docker compose pull openslate && docker compose up -d openslate
*/2 * * * * cd /opt/openshelf && docker compose pull && docker compose up -d >> /var/log/openshelf-update.log 2>&1
```
