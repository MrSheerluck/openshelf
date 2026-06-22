# Deployment

OpenShelf runs in a single Docker container: Rust backend + SvelteKit frontend + Caddy reverse proxy. SQLite stores all data in a persistent Docker volume.

---

## Option 1: Local Docker

Run entirely on your own machine. No domain, no cloud accounts.

### Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (Mac/Windows) or Docker Engine (Linux)

### Steps

```bash
git clone https://github.com/MrSheerluck/openshelf.git
cd openshelf
cp .env.example .env
```

Edit `.env` and set a `JWT_SECRET`:

```
JWT_SECRET=your-random-secret-here
```

Start:

```bash
docker compose up -d
```

The first build compiles Rust (10–20 min on Apple Silicon, longer on Intel). Open **http://localhost:8080** and set your admin password on first visit.

### Custom port

```bash
docker compose up -d   # default: http://localhost:8080
```

Or edit `docker-compose.yml` to change the left-hand port number from `8080` to whatever you want.

### Media uploads (optional)

Add R2 credentials to `.env`:
```
S3_BUCKET=openshelf
R2_ACCOUNT_ID=your-id
S3_ACCESS_KEY=your-key
S3_SECRET_KEY=your-secret
```

Then restart: `docker compose down && docker compose up -d`.

> **Note for local use:** The repo's `docker-compose.yml` uses `build: .` (local build). For VPS deployments using the GHCR image, edit the compose file to replace `build: .` with `image: ghcr.io/YOUR_USERNAME/openshelf:latest`.

---

## Option 2: Cloud VPS

### DigitalOcean

#### Method A: Cloud-init (automatic)

1. Log into [DigitalOcean](https://cloud.digitalocean.com), click **Create → Droplets**.
2. Choose **Ubuntu 24.04 LTS**.
3. Pick the cheapest plan: **$4/mo Basic** (1 vCPU, 512 MB RAM, 10 GB SSD).
4. Scroll to **Advanced Options → User Data** and paste the entire contents of [`scripts/cloud-init.yaml`](../scripts/cloud-init.yaml).
5. Click **Create Droplet**. Wait 3–5 minutes.

Open `http://<droplet-ip>:8080`. On first visit, set your admin password.

> **Note:** The $4 droplet has 512 MB RAM which is **not enough to compile Rust** during `docker build`. The cloud-init script will fail at the build step. See [Solving the RAM problem](#solving-the-ram-problem-on-the-cheapest-vps) below.

#### Method B: Manual (with GHCR)

Follow the [generic GHCR setup](#setup-with-ghcr-image-recommended-for-any-vps) below.

### Hetzner

**Recommended plan:** CX23 (2 vCPU, 4 GB RAM, 40 GB SSD, ~€5/mo). The 4 GB RAM means you can build directly if you want, but using the GHCR pre-built image is still faster.

1. Create a server at [Hetzner Cloud Console](https://console.hetzner.cloud):
   - Image: **Ubuntu 24.04**
   - Plan: **CX23**
   - Add your SSH key under **Security → SSH Keys** (generate one with `ssh-keygen -t ed25519`)

2. SSH into the server:
   ```bash
   ssh root@<server-ip>
   ```

3. Follow the [generic GHCR setup](#setup-with-ghcr-image-recommended-for-any-vps) below.

### Setup with GHCR image (recommended for any VPS)

If your repo has CI/CD set up (GitHub Actions builds and pushes to GHCR on every push to `main`), you can pull the pre-built image directly with no Rust compilation needed.

SSH into your server and run:

```bash
apt update && apt install -y docker.io docker-compose-v2 git
systemctl enable --now docker
git clone https://github.com/YOUR_USERNAME/openshelf /opt/openshelf
cd /opt/openshelf
cp .env.example .env
sed -i "s/JWT_SECRET=.*/JWT_SECRET=$(openssl rand -hex 32)/" .env
sed -i 's#build: .#image: ghcr.io/YOUR_USERNAME/openshelf:latest#' docker-compose.yml
echo "YOUR_GH_PAT" | docker login ghcr.io -u YOUR_USERNAME --password-stdin
docker compose up -d
```

Open `http://<server-ip>:8080` and set your admin password.

To create a GitHub PAT: **GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)**. Check `read:packages` scope (only read is needed for pulling).

---

## Custom Domain + HTTPS

### 1. Point DNS

In your domain's DNS provider (Cloudflare, Namecheap, GoDaddy, etc.), add an **A record**:

| Type | Name  | Value             |
|------|-------|-------------------|
| A    | books | `<server-ip>`    |

This routes `books.yourdomain.com` to your server. DNS propagation takes 2–5 minutes.

### 2. Update .env on the VPS

```bash
cd /opt/openshelf
sed -i 's/DOMAIN=/DOMAIN=books.yourdomain.com/' .env
```

### 3. Restart

```bash
docker compose down && docker compose up -d
```

Caddy automatically contacts Let's Encrypt, proves you own the domain, and provisions a TLS certificate. No manual cert setup, no renewal cron jobs. Caddy handles everything.

Your app is now at `https://books.yourdomain.com`.

### How it works

- The entrypoint script detects `DOMAIN` is set and rewrites the Caddy config from `:8080` to `books.yourdomain.com`.
- Caddy (with a proper domain name) automatically uses ports 80/443 and provisions TLS.
- `docker-compose.yml` already exposes ports 80, 443, and 8080 with no changes needed.
- Caddy stores certificates in `/data/caddy/` (persisted in the Docker volume).

---

## Solving the RAM problem (cheapest VPS only)

The cheapest DigitalOcean droplet (512 MB RAM) cannot compile Rust. The compiler needs 1–2 GB. Hetzner CX23 (4 GB RAM) does not have this issue. Solutions, in order of simplicity:

### 1. Build on your machine, push to registry (recommended)

As shown in [Method B](#method-b-manual-with-ghcr) above. Build once locally, push to GitHub Container Registry (free for public repos), pull on the VPS. The VPS never compiles anything.

```bash
# Local: build once
docker buildx build --platform linux/amd64 -t ghcr.io/you/openshelf:latest --push .

# VPS: just pull
docker compose pull && docker compose up -d
```

### 2. Upgrade to a larger plan

2 GB RAM or more, enough to compile. Simplest approach but costs more per month.

### 3. Use swap space

Add 2 GB of swap as a fallback (much slower builds, but works):

```bash
fallocate -l 2G /swapfile
chmod 600 /swapfile
mkswap /swapfile
swapon /swapfile
echo '/swapfile none swap sw 0 0' >> /etc/fstab
```

---

## Environment Variables

All config lives in `/opt/openshelf/.env`:

| Variable | Required | Description |
|----------|----------|-------------|
| `JWT_SECRET` | Yes | Random string for signing auth tokens. Generate with `openssl rand -hex 32`. |
| `DOMAIN` | No | Your domain for HTTPS (e.g. `books.example.com`). |
| `R2_ACCOUNT_ID` | For R2 | Cloudflare account ID. When set, uses the R2 global endpoint. |
| `S3_BUCKET` | For uploads | S3-compatible bucket name. |
| `S3_REGION` | No | S3 region. Default: `auto`. |
| `S3_ACCESS_KEY` | For uploads | S3 access key ID. |
| `S3_SECRET_KEY` | For uploads | S3 secret access key. |
| `S3_ENDPOINT` | For non-R2 | Custom S3 endpoint URL (e.g. Garage, MinIO). Do not set at the same time as `R2_ACCOUNT_ID`. |

After changing any value, run: `docker compose down && docker compose up -d`.

The container also reads `DATABASE_PATH` (default `/data/app.db`) and `PORT` (default `3001`) but you shouldn't need to change these.

---

## Updates

### Option A: Automated (CI + cron job) (recommended)

Once set up, updates are fully hands-off: push to `main` and the VPS updates itself within minutes.

**How it works:**
- A GitHub Actions workflow builds and pushes the Docker image to GHCR on every push to `main`.
- A cron job on the VPS runs every 2 minutes, pulls the latest GHCR image, and restarts the container if a new version was found.

**One-time setup:**

1. Add a GitHub personal access token as a repo secret (so CI can push the image):
   - **GitHub → Repo Settings → Secrets and variables → Actions → New repository secret**
   - Name: `GHCR_PAT`
   - Value: a [classic PAT](https://github.com/settings/tokens) with `read:packages` and `write:packages` scopes.

2. On your VPS, add a cron job that pulls the latest image every 2 minutes:

   ```bash
   (crontab -l 2>/dev/null; echo '*/2 * * * * cd /opt/openshelf && docker compose pull && docker compose up -d') | crontab -
   ```

   Or run the repo's helper: `cd /opt/openshelf && ./scripts/install-cron.sh`.

3. On your VPS, edit `/opt/openshelf/docker-compose.yml` and replace `build: .` with `image: ghcr.io/YOUR_USERNAME/openshelf:latest` so the cron job pulls from GHCR instead of rebuilding locally.

From then on: `git push` to `main` → CI builds & pushes → cron pulls within 2 min → auto-updates. Nothing else needed.

### Option B: Manual (push from your machine)

Build and push from your local machine, then pull on the VPS:

```bash
# On your machine
docker buildx build --platform linux/amd64 -t ghcr.io/you/openshelf:latest --push .

# On the VPS
cd /opt/openshelf && docker compose pull && docker compose up -d
```

### Option C: Build on VPS (only if you have enough RAM)

```bash
cd /opt/openshelf && git pull && docker compose up -d --build
```

Your SQLite database in the Docker volume is preserved across all update methods.

---

## Backups

The SQLite database lives in a Docker volume. Back it up:

```bash
cd /opt/openshelf
docker compose exec openshelf cp /data/app.db /data/app-backup.db
docker compose cp openshelf:/data/app-backup.db ./backup-$(date +%Y%m%d).db
```

Or find the volume location directly:

```bash
docker compose down
cp $(docker volume inspect openshelf_data --format '{{.Mountpoint}}')/app.db ./backup.db
docker compose up -d
```

Book files live in your S3-compatible service, not on the VPS. Back them up according to the provider's docs.

---

## Troubleshooting

**"This site can't be reached":** Run `docker compose logs --tail 30` and check for errors. Common causes:
- R2 credentials as empty strings. Make sure they're filled in `.env`, not left as `=`.
- Caddyfile error after editing. The entrypoint handles the `DOMAIN` substitution.
- Port not exposed. Verify `docker compose ps` shows port mappings.

**Can't log in:** The first visit should show "Set your admin password." If you already did, use the same password.

**Build fails on VPS:** Out of memory. See [Solving the RAM problem](#solving-the-ram-problem-on-the-cheapest-vps).

**Wrong platform error:** `linux/arm64/v8` vs `linux/amd64/v3`. You built on Apple Silicon but need AMD64. Use `--platform linux/amd64` in your build command.

**Book uploads don't work:** Check `docker compose logs | grep -i s3`. Verify all four `R2_*` / `S3_*` variables are set in `.env` with non-empty values.

**Cron job never updates the VPS:** Check `crontab -l` shows the OpenShelf line and that `/var/log/openshelf-update.log` doesn't show auth errors. If you see `unauthorized` or `403`, your `GHCR_PAT` on the repo is wrong or doesn't have `write:packages` scope.

### Resetting the admin password

The simplest way: delete the user row, then visit `/login` again. The page sees no users and shows the signup form.

```bash
docker compose exec openshelf sqlite3 /data/app.db "DELETE FROM users;"
docker compose restart openshelf
```

Then open `http://your-host:8080/login` and set a new password.

> **Note:** The `gen-hash` CLI helper in `backend/src/bin/gen_hash.rs` is dev-only: the production Docker image only ships the main backend binary. To seed a hash without going through the signup form, run `cargo run --bin gen-hash -- 'your-password'` on a development checkout and `UPDATE` the row with the PHC string.
