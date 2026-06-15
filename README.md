# OpenShelf

A self-hosted ebook reader web app.

**Stack:** SvelteKit (frontend), Rust/Axum (backend), SQLite (metadata), Garage (file storage)

## Roadmap

- **v0.1** — EPUB support with basic reader and library management
- **Future** — MOBI, PDF, CBZ/CBR, and audiobook support

## Dev

```bash
docker compose up --build
```

- Frontend: `http://localhost:3000`
- Backend health: `http://localhost:3001/health`
