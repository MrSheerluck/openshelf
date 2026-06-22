# Setup Guide

This guide covers running OpenShelf without Docker: for local development or bare-metal installs. For the easiest setup, use Docker; see [Deployment](deployment.md).

## Prerequisites

- **Rust** 1.85+. [rustup.rs](https://rustup.rs/)
- **Bun** v1.x. [bun.sh](https://bun.sh/) (or Node.js + npm)
- **SQLite** is bundled: no system install needed
- **S3-compatible storage** (optional, only needed for book uploads):
  - Cloudflare R2 account, or
  - Garage / MinIO / any S3-compatible service running locally

## Clone and Install

```bash
git clone https://github.com/MrSheerluck/openshelf.git
cd openshelf
```

## Backend Setup

### 1. Generate a secret and configure env

The repo ships with a helper script that writes a `.env` with a random `JWT_SECRET`:

```bash
./scripts/setup.sh
```

Or do it manually:

```bash
cp .env.example .env
sed -i "s/JWT_SECRET=.*/JWT_SECRET=$(openssl rand -hex 32)/" .env
```

Edit `.env` and fill in the S3/R2 credentials if you plan to upload books (see [Environment Variables](#environment-variables) below).

### 2. Run the backend

```bash
cd backend
cargo run
```

The API starts on `http://localhost:3001`. Verify with:

```bash
curl http://localhost:3001/api/health
# → {"status":"ok","db":"connected"}
```

On first run, SQLite migrations run automatically. The database file (`app.db` by default) is created in the working directory.

## Frontend Setup

### 1. Configure the API URL

```bash
cd frontend
echo 'VITE_API_URL=http://localhost:3001' > .env
```

The default value already points at the local backend, so this step is only needed if you want a different URL.

### 2. Install dependencies and run

```bash
bun install
bun run dev
```

The frontend starts on `http://localhost:3000`. Open it in your browser: the first visit shows a "set your password" form, which calls `POST /api/auth/signup` and creates the only user.

## Environment Variables

All variables are read at backend startup. Fill them in `.env` at the repo root (or `backend/.env` works too via `dotenvy`).

| Variable | Required | Description |
|----------|----------|-------------|
| `JWT_SECRET` | Yes (production) | Random string for signing JWT tokens. Generate with `openssl rand -hex 32`. A dev fallback of `dev-secret-change-me` is used if unset: never ship this. |
| `DATABASE_PATH` | No | SQLite file path. Default: `./app.db` |
| `FRONTEND_URL` | No | Frontend origin for CORS. Default: `http://localhost:3000` |
| `S3_BUCKET` | For uploads | S3-compatible bucket name |
| `S3_REGION` | For non-R2 | S3 region. Default: `auto` |
| `S3_ENDPOINT` | For non-R2 | Custom S3 endpoint URL (e.g. `http://localhost:3900` for Garage) |
| `S3_ACCESS_KEY` | For uploads | S3 access key ID |
| `S3_SECRET_KEY` | For uploads | S3 secret access key |
| `R2_ACCOUNT_ID` | For R2 | Cloudflare account ID: uses the R2 endpoint preset |
| `DOMAIN` | No | Custom domain for Caddy HTTPS (when the proxy is enabled) |

> R2 vs Garage: set `R2_ACCOUNT_ID` to use Cloudflare R2's global endpoint, or leave it blank and set `S3_ENDPOINT` for a generic S3-compatible service. **Do not set both.**

### Storage configurations

**Cloudflare R2 (recommended for production):**
```env
R2_ACCOUNT_ID=your-cloudflare-account-id
S3_BUCKET=openshelf
S3_ACCESS_KEY=your-r2-access-key
S3_SECRET_KEY=your-r2-secret-key
```

**Garage (self-hosted S3):**
```env
S3_ENDPOINT=http://localhost:3900
S3_BUCKET=openshelf
S3_REGION=garage
S3_ACCESS_KEY=your-key
S3_SECRET_KEY=your-secret
```

**No storage (development without uploads):**
Leave all `R2_*` and `S3_*` vars blank. The reader will still work over an existing library; only upload endpoints will fail with `503 Storage not configured`.

## Quick start: Docker

If you just want to run the app, this is faster than the manual setup:

```bash
cp .env.example .env
# edit .env and set JWT_SECRET
docker compose up -d
```

Browse to `http://localhost:8080`. See [Deployment](deployment.md) for the full Docker guide, including VPS and custom-domain setup.

## Project Structure

```
openshelf/
├── backend/                 # Rust backend
│   ├── src/
│   │   ├── main.rs          # Entry point, router, CORS, app state
│   │   ├── auth.rs          # JWT middleware, /me, /logout
│   │   ├── users.rs         # signup, signin, password hashing
│   │   ├── books.rs         # Books, progress, bookmarks, annotations
│   │   ├── storage.rs       # S3/R2 client wrapper
│   │   ├── db.rs            # SQLite connection + migrations
│   │   └── bin/gen_hash.rs  # CLI helper to print an argon2 hash
│   └── Cargo.toml
├── frontend/                # SvelteKit frontend
│   ├── src/
│   │   ├── lib/             # api client, auth state, reader modules
│   │   └── routes/          # SvelteKit pages (login, library, reader)
│   └── package.json
├── docs/                    # Documentation (you are here)
├── scripts/setup.sh         # Writes .env with random JWT_SECRET
├── Dockerfile               # Single-image build: bun frontend + Rust backend + Caddy
├── Caddyfile                # Proxies /api/* to backend, serves frontend otherwise
├── docker-entrypoint.sh     # Starts backend, then Caddy in foreground
├── docker-compose.yml
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── LICENSE
└── README.md
```

## Troubleshooting

**`cargo build` fails on a memory-constrained VPS:** Rust compilation needs 1–2 GB RAM. Use the GHCR pre-built image approach in [Deployment](deployment.md#solving-the-ram-problem-on-the-cheapest-vps) instead.

**CORS errors in the browser console:** Make sure `FRONTEND_URL` matches the origin you're loading the frontend from (e.g. `http://localhost:3000` for local dev). The backend only allows that exact origin with credentials.

**`Storage not configured` on upload:** Fill in the S3/R2 vars in `.env` and restart the backend.

**Login keeps failing:** The first visit creates the user. If you cleared the database, the next signup will work again. If you forgot the password, delete the row from the `users` table and re-signup, or use `cargo run --bin gen-hash` to seed a new hash manually.

## Multi-app note

`scripts/install-cron.sh` matches the OpenShelf cron entry by the app directory (`cd ${APP_DIR}`), not by a generic string like `docker compose pull`. That means it is safe to run on a VPS that already has cron entries for other apps (for example openslate). Running it will only ever add, replace, or remove the OpenShelf line. See [multi-app.md](multi-app.md) for the full multi-app hosting pattern.
