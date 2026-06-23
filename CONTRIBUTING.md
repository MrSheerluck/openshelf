# Contributing to OpenShelf

Thanks for your interest in contributing! OpenShelf is a self-hosted ebook reader built with SvelteKit (frontend) and Rust/Axum (backend).

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before participating.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable, 1.85+)
- [Bun](https://bun.sh/) v1.x (or Node.js + npm)
- SQLite is bundled via `rusqlite`'s `bundled` feature: no system install needed

### Setup

```bash
# Backend
cd backend
cp .env.example .env   # create from template
cargo run              # starts on localhost:3001

# Frontend (in a second terminal)
cd frontend
bun install
bun run dev            # starts on localhost:3000
```

The frontend talks to the backend at `http://localhost:3001` by default. Override with `VITE_API_URL` if needed.

> **Note:** The `.env` file contains secrets like `JWT_SECRET`. Never commit real secrets. Always start from `.env.example` and keep `.env` in `.gitignore`.

### Environment Variables

Copy `.env.example` to `.env` at the repo root and configure:

| Variable | Description | Default |
|---|---|---|
| `JWT_SECRET` | Secret for signing JWT tokens | (generate a random string: required in production) |
| `DATABASE_PATH` | SQLite database file path | `./app.db` |
| `FRONTEND_URL` | Frontend origin for CORS | `http://localhost:3000` |
| `S3_BUCKET` | S3-compatible bucket name | n/a |
| `S3_REGION` | S3 region (for non-R2 endpoints) | `auto` |
| `S3_ENDPOINT` | Custom S3 endpoint (e.g. Garage) | n/a |
| `S3_ACCESS_KEY` | S3 access key ID | n/a |
| `S3_SECRET_KEY` | S3 secret access key | n/a |
| `R2_ACCOUNT_ID` | Cloudflare account ID (uses R2 endpoint preset) | n/a |
| `DOMAIN` | Custom domain for Caddy HTTPS | n/a |

> S3/R2 credentials are optional. Without them, book upload endpoints will return `503 Storage not configured`. The auth flow and reader still work over an existing library.

## Project Structure

```
openshelf/
├── backend/                # Rust backend (Axum + SQLite)
│   ├── src/
│   │   ├── main.rs         # Entry point, router, CORS, app state
│   │   ├── auth.rs         # JWT middleware, /me, /logout
│   │   ├── users.rs        # signup, signin, password hashing (argon2)
│   │   ├── books.rs        # Book CRUD, EPUB parsing, progress, bookmarks, annotations
│   │   ├── storage.rs      # S3/R2 client wrapper
│   │   ├── db.rs           # SQLite connection + migrations
│   │   └── bin/gen_hash.rs # CLI helper to print an argon2 hash
│   ├── migrations/         # (migrations are inline in db.rs for now)
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/               # SvelteKit frontend
│   └── src/
│       ├── lib/
│       │   ├── api.ts              # fetch() wrapper with credentials
│       │   ├── auth.svelte.ts      # Auth state, signin/signup/logout

│       │   └── reader/
│       │       ├── epub.svelte.ts        # epub.js controller
│       │       ├── bookmarks.svelte.ts   # Bookmark state
│       │       ├── highlights.svelte.ts  # Highlight state
│       │       ├── settings.svelte.ts   # Per-book typography/theme
│       │       ├── themes.ts             # 6 reader themes + font list
│       │       └── types.ts
│       └── routes/
│           ├── +layout.svelte     # Auth guard, redirect to /login
│           ├── +page.svelte       # Library grid, upload, settings
│           ├── login/+page.svelte # First-run signup / signin form
│           └── read/[id]/+page.svelte  # Reader view
├── docs/                   # Documentation (you are here)
├── scripts/
│   ├── setup.sh            # Generates .env with a random JWT_SECRET
│   ├── install-cron.sh     # Installs the VPS auto-update cron
│   ├── uninstall-cron.sh   # Removes the auto-update cron
│   └── cloud-init.yaml     # One-click VPS setup (DigitalOcean user-data)
├── Dockerfile              # Multi-stage: bun frontend + Rust backend + Caddy
├── Caddyfile               # Proxies /api/* to backend, serves frontend otherwise
├── docker-entrypoint.sh    # Starts backend in background, then Caddy in foreground
├── docker-compose.yml
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── LICENSE
└── README.md
```

## Code Style

- **Rust:** Follow `cargo fmt` conventions. Use `?` and `map_err` over `unwrap`/`panic` in route handlers. Prefer `rusqlite::params![]` for parameter binding. Group handlers per resource (`books.rs`, `users.rs`, `auth.rs`).
- **TypeScript/Svelte:** Use Svelte 5 Runes (`$state`, `$derived`, `$effect`, `$props`). No classes: use module-level functions and runes. Type everything with TypeScript. Strict mode is on.
- **CSS:** Tailwind utility classes + CSS custom properties for theming. Reader themes live in `frontend/src/lib/reader/themes.ts` and are applied via inline `style` attributes on the reader container.

## Making Changes

1. **Open an issue first** for bugs or feature requests so we can discuss before you write code.
2. **Fork and branch**: use a descriptive branch name like `fix/cover-extraction-fallback` or `feat/pdf-renderer`.
3. **Keep PRs focused**: one feature or fix per PR. Small PRs are reviewed faster.
4. **Test your changes**: verify the app still builds and runs:

   ```bash
   cd backend && cargo build           # backend compiles
   cd frontend && bun run check         # frontend type-checks
   ```

(We don't have automated tests yet: help add them!)

## Pull Request Process

1. Ensure your code compiles (`cargo build`) and type-checks (`bun run check`).
2. If your change adds a feature, update the relevant doc (`docs/features.md`, `docs/api-reference.md`) and tick the box in `ROADMAP.md`.
3. If your change alters env vars or deployment, update `docs/setup.md` and `docs/deployment.md`.
4. PRs need at least one maintainer review before merging.

## Generating a password hash

If you want to seed a user manually outside the normal signup flow:

```bash
cd backend
cargo run --bin gen-hash -- 'your-password'
# prints an argon2 PHC string
```

## Good First Issues

Check the issues labeled `good first issue` for well-scoped starter tasks.

## AI Usage Policy

Using AI assistants (Copilot, ChatGPT, Claude, etc.) is completely fine. However, you are responsible for every change you submit. Make sure you understand what the AI generated, why it works, and that it fits the project's code style. Blindly accepting AI output without review is not acceptable.

## Code of Conduct

Be respectful and constructive. We're building something cool: let's keep it fun for everyone.
