# OpenShelf

A self-hosted ebook reader web app. Fast, simple, private. Your library, on your hardware, accessible from any device.

**Single user only.** No sign-ups, no sharing, no complexity. Just your books.

## Deploy

### Docker (local)

```bash
git clone https://github.com/MrSheerluck/openshelf
cd openshelf
cp .env.example .env     # set JWT_SECRET
docker compose up -d     # open http://localhost:8080
```

### DigitalOcean VPS

**Recommended:** Build the image on your machine (the cheapest VPS plans don't have enough RAM to compile Rust), push to GitHub Container Registry (free), then pull on the VPS:

```bash
# On your machine
docker buildx build --platform linux/amd64 -t ghcr.io/you/openshelf:latest --push .

# On the VPS
git clone https://github.com/MrSheerluck/openshelf /opt/openshelf
cd /opt/openshelf
cp .env.example .env && sed -i "s/JWT_SECRET=.*/JWT_SECRET=$(openssl rand -hex 32)/" .env
# edit .env to fill in R2 credentials
echo "your-token" | docker login ghcr.io -u you --password-stdin
docker compose up -d
```

**Custom domain:** Add `DOMAIN=books.example.com` to `.env`, point an A record to your IP, restart. Caddy auto-provisions HTTPS.

Full step-by-step guide: [docs/deployment.md](docs/deployment.md)

> **Already running another app on this VPS?** See [docs/multi-app.md](docs/multi-app.md) for the host-Caddy pattern that lets openshelf share a box with other apps at `https://<app>.yourdomain.com`.

---

## Features

- **Self-hosted library.** EPUB upload, cover extraction, and metadata parsing. Stored in S3-compatible storage (Cloudflare R2 or Garage).
- **Paginated reader.** epub.js-powered reader with themes, typography controls, and a clean Apple Books-inspired UI.
- **Bookmarks.** Save your spot in any chapter, jump back from a sidebar list.
- **Highlights & notes.** Multi-color highlights with optional notes, exportable as Markdown.
- **Reading progress.** Per-book CFI position, syncs to the server so progress follows you between devices.
- **Six reader themes.** Light, Cream, Sepia, Mint, Dark, Night.
- **Per-book typography.** Font family, size, line height, margin, alignment.
- **Simple auth.** Single password, log in from any device (argon2 hash, JWT cookies).
- **Self-contained.** One Docker container, SQLite database, no external services required for the core app.

---

## Stack

| Layer | Technology |
|-------|-----------|
| Frontend | SvelteKit + TypeScript + epub.js |
| Backend | Rust (Axum) + SQLite (rusqlite) |
| Database | SQLite (bundled) |
| Auth | argon2 password hash, JWT in httpOnly cookie |
| Storage | S3-compatible (Cloudflare R2 or Garage) |
| Reverse proxy | Caddy (auto HTTPS, in-container) |

---

## Documentation

- [Setup Guide](docs/setup.md): Prerequisites, env vars, running without Docker
- [Architecture](docs/architecture.md): Project structure, data flow, design decisions
- [Features](docs/features.md): Reader, themes, bookmarks, highlights, roadmap
- [API Reference](docs/api-reference.md): Full REST API docs
- [Deployment](docs/deployment.md): Docker, VPS, custom domain + HTTPS

---

## Development

```bash
# Backend
cd backend && cargo run

# Frontend (separate terminal)
cd frontend && bun install && bun run dev
```

The backend runs on `http://localhost:3001`, the SvelteKit dev server on `http://localhost:3000`. In dev, the Vite dev server proxies `/api/*` requests to the backend. In Docker, Caddy does the same thing.

---

## License

[MIT](LICENSE)
