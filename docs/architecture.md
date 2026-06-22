# Architecture

## Overview

OpenShelf is a single-user, self-hosted ebook reader with a SvelteKit frontend and a Rust/Axum backend. Book files live in S3-compatible storage (Cloudflare R2 or Garage). Metadata, bookmarks, annotations, and reader settings live in SQLite.

```
                       [Docker container]
[Browser] ──► [Caddy :8080] ──► [SvelteKit SPA static]    (everything except /api/*)
                  │
                  └──► [Axum API :3001] ──► [SQLite at /data/app.db]
                                          │
                                          └──► [R2 / Garage S3]
                                                (EPUB files + covers)
```

- The frontend is a **single-page application**; all library, reader, and settings interaction happens on one app shell.
- Caddy serves the static SvelteKit build and reverse-proxies `/api/*` to the Axum backend on the same container, port 3001.
- The backend exposes a **REST API** on port 3001. All non-auth routes require a valid JWT cookie.
- Authentication uses a **JWT in an httpOnly cookie** (30-day expiry) issued after argon2 password verification.
- In dev (no Docker), the Vite dev server on `:3000` proxies `/api/*` to the backend on `:3001` so the same code path works in both environments.
- When `DOMAIN` is set, the entrypoint script rewrites the Caddy config to use the real domain, and Caddy auto-provisions TLS via Let's Encrypt.

## Project Structure

```
backend/src/
├── main.rs          # Entry point, router, CORS, app state, 100 MB body limit
├── auth.rs          # require_auth middleware, /me, /logout
├── users.rs         # signup, signin, status; argon2 password hashing
├── books.rs         # Book CRUD, EPUB cover extraction, progress, bookmarks, annotations
├── storage.rs       # S3/R2 client wrapper (R2 preset + generic S3 endpoint)
├── db.rs            # SQLite connection + inline migrations
└── bin/gen_hash.rs  # CLI: prints an argon2 hash for a given password

frontend/src/
├── lib/
│   ├── api.ts              # fetch() wrapper with credentials: include
│   ├── auth.svelte.ts      # Auth state ($state rune), login/logout
│   ├── pdf.ts              # pdf.js worker setup
│   └── reader/
│       ├── epub.svelte.ts        # epub.js controller: pagination, themes, CFI
│       ├── bookmarks.svelte.ts   # Bookmark CRUD + UI state
│       ├── highlights.svelte.ts  # Highlight CRUD + UI state
│       ├── settings.svelte.ts    # Per-book typography & reader settings
│       ├── themes.ts             # 6 themes + 5 font families
│       └── types.ts              # Shared reader types
└── routes/
    ├── +layout.svelte         # Auth guard, redirects to /login
    ├── +page.svelte           # Library grid, upload, settings
    ├── login/+page.svelte     # First-run signup / signin form
    └── read/[id]/+page.svelte # Reader view
```

## Data Flow

### Authentication

1. User visits `/`: `+layout.svelte` calls `checkAuth()`, which hits `GET /api/auth/me`.
2. If `me` returns 401, the user is redirected to `/login`.
3. `/login` calls `GET /api/auth/status` to learn whether a user already exists.
   - No user → form calls `POST /api/auth/signup` (only allowed if `users` table is empty).
   - User exists → form calls `POST /api/auth/signin`.
4. On success, the backend sets a 30-day JWT httpOnly cookie (`token`, `SameSite=Lax`).
5. All subsequent API requests include the cookie automatically: the frontend's `api()` helper sets `credentials: "include"` on every fetch.
6. The backend's `require_auth` middleware decodes the JWT on protected routes and rejects with 401 if missing/invalid. The login, signup, status, and book cover endpoints are exempt.

### Book upload

1. User drops an `.epub` onto the library page.
2. Frontend POSTs `multipart/form-data` to `POST /api/books` with the file.
3. Backend validates the file, computes an ID, and:
   - Extracts the cover image from the EPUB (parses `META-INF/container.xml` → OPF → `meta name="cover"`).
   - Uploads the cover to S3/R2.
   - Uploads the EPUB to S3/R2.
   - Inserts a row into the `books` table with metadata (title, author, file paths, etc.).
4. Frontend receives the new `Book` object and prepends it to the library grid.

### Reading

1. User clicks a book → navigates to `/read/{id}`.
2. Frontend requests `GET /api/books/{id}/file` (proxied through the backend, which streams from S3).
3. epub.js parses the EPUB and renders it. Cover, chapter images, and embedded fonts are fetched from `GET /api/books/{id}/cover` and `GET /api/books/{id}/resource/{path}`.
4. The reader controller (`epub.svelte.ts`) holds the current CFI position. On page turn, it debounces a `POST /api/books/{id}/progress` to save it.
5. Bookmarks are saved via `POST /api/books/{id}/bookmarks` (chapter index + CFI + label).
6. Highlights are saved via `POST /api/books/{id}/annotations` (chapter index + CFI + selected text + color + optional note).

### Bookmarks & annotations

- Each bookmark points at a `(chapter_index, cfi)` pair inside the EPUB. The `cfi` is the EPUB Canonical Fragment Identifier, generated by epub.js: it uniquely addresses a position in the book and survives re-pagination.
- Annotations are the same position data, plus the highlighted text, a color, and an optional user note.
- `GET /api/books/{id}/annotations/export` returns all annotations for a book as a Markdown file (handy for export to other tools).

### Per-book reader settings

- `GET /api/books/{id}/settings` and `PUT /api/books/{id}/settings` store the reader's typography preferences (font family, size, line height, margin, alignment, theme) under a JSON blob keyed by book ID.
- The reader applies these on mount and writes back on change.

## Database Schema

Migrations live inline in `backend/src/db.rs` and run on every startup. `CREATE TABLE IF NOT EXISTS` makes them idempotent; the one-shot `current_page` type fix is guarded by a `pragma_table_info` check.

```sql
books (
  id TEXT PRIMARY KEY,          -- UUID v4
  title TEXT NOT NULL,
  author TEXT,
  description TEXT,
  publisher TEXT,
  isbn TEXT,
  language TEXT,
  cover_path TEXT,              -- S3 key for cover image
  file_path TEXT NOT NULL,      -- S3 key for the EPUB/PDF
  format TEXT NOT NULL DEFAULT 'epub',  -- epub, pdf, mobi, cbz
  file_size INTEGER,            -- bytes
  page_count INTEGER,           -- total pages
  current_page TEXT,            -- last reading CFI (TEXT, not INTEGER, to hold CFIs)
  created_at TEXT DEFAULT (datetime('now')),
  updated_at TEXT DEFAULT (datetime('now'))
);

bookmarks (
  id TEXT PRIMARY KEY,
  book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  chapter_index INTEGER NOT NULL,  -- which spine item
  cfi TEXT NOT NULL,              -- EPUB CFI for exact position
  label TEXT,                     -- user-given name
  created_at TEXT DEFAULT (datetime('now'))
);

annotations (
  id TEXT PRIMARY KEY,
  book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  chapter_index INTEGER NOT NULL,
  cfi TEXT NOT NULL,
  text TEXT NOT NULL,              -- the highlighted text
  note TEXT,                      -- user's note attached to the highlight
  color TEXT DEFAULT 'yellow',    -- highlight color
  created_at TEXT DEFAULT (datetime('now')),
  updated_at TEXT DEFAULT (datetime('now'))
);

tags (id TEXT PRIMARY KEY, name UNIQUE NOT NULL);
book_tags (book_id REFERENCES books(id) ON DELETE CASCADE,
            tag_id REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (book_id, tag_id));

preferences (key TEXT PRIMARY KEY, value TEXT);  -- global key-value (theme sync)

users (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL,
  password_hash TEXT NOT NULL,    -- argon2 PHC string
  created_at TEXT DEFAULT (datetime('now')),
  updated_at TEXT DEFAULT (datetime('now'))
);
```

All primary keys are UUIDs (v4). Timestamps are SQLite `datetime('now')` text. Foreign keys cascade on delete: removing a book removes its bookmarks, annotations, and tag links. `book_tags` is a join table for the many-to-many between books and tags.

## Design Decisions

**SQLite over PostgreSQL:** Simpler self-hosting story. One file, no separate database server. The single-user workload doesn't need connection pooling: a single `Arc<Mutex<Connection>>` is enough.

**S3-compatible storage instead of filesystem:** Book files can be large (multi-MB EPUBs with embedded media). S3-compatible storage keeps the database container stateless and lets you back up books independently of metadata. Cloudflare R2 has no egress fees, making it ideal for self-hosted readers that stream content.

**epub.js in the browser instead of server-side rendering:** The reader is fully client-side. The backend only serves the raw EPUB bytes; the browser parses, paginates, and themes. This keeps the backend simple and lets the reader handle pagination per device/zoom level.

**JWT in httpOnly cookies over Bearer tokens:** Simpler for a SPA: no need to manage tokens in JavaScript. The cookie is sent automatically. Protects against XSS token theft. `SameSite=Lax` so it works for normal navigations but blocks cross-site POSTs.

**argon2 over bcrypt:** Modern, memory-hard, and the de-facto recommendation. The user table stores PHC strings, so the parameters can be tuned without code changes.

**Single user only:** Eliminates the complexity of user management, permissions, sharing, and per-user quotas. The first visit creates the only user; subsequent signups return 409 Conflict.

**Svelte 5 Runes everywhere:** `$state`, `$derived`, `$effect`, `$props` for all reactive state. No stores, no classes. This is the Svelte 5 idiom and the project sticks to it.

**One Docker container, not two:** The SvelteKit frontend is built into a static bundle, the Rust backend compiles to a single binary, and Caddy sits in front. The multi-stage `Dockerfile` packages all three into a `debian:bookworm-slim` image. One container to build, one to ship, one to run: a single `docker compose up -d` is the whole deploy story.
