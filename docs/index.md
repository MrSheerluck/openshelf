# OpenShelf Documentation

OpenShelf is a self-hosted ebook reader web app. Fast, simple, private. Your library, on your hardware, accessible from any device.

**Single user only.** No sign-ups, no sharing, no complexity. Just your books.

## Getting Started

- [Setup Guide](setup.md): Prerequisites, environment variables, running without Docker
- [Deployment](deployment.md): Docker, VPS, custom domain + HTTPS
- [Architecture](architecture.md): Project structure, data flow, design decisions
- [Features](features.md): Reader, themes, bookmarks, highlights, what's coming
- [API Reference](api-reference.md): Full REST API endpoint documentation

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | SvelteKit + TypeScript + epub.js + pdf.js |
| Backend | Rust (Axum) + SQLite (rusqlite) |
| Database | SQLite (bundled) |
| Auth | argon2 password hash, JWT in httpOnly cookie |
| Storage | S3-compatible (Cloudflare R2 or Garage) |
| Reverse proxy | Caddy (auto HTTPS, planned) |

## Quick Links

- [Contributing Guide](../CONTRIBUTING.md)
- [Code of Conduct](../CODE_OF_CONDUCT.md)
- [Project Roadmap](../ROADMAP.md)
- [License](../LICENSE)
