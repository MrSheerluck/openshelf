# API Reference

Base URL: `http://localhost:3001` in dev, or `http://localhost:8080/api` (or `https://yourdomain.com/api` in production) via the in-container Caddy reverse proxy.

All responses are JSON unless otherwise noted. Protected routes require a valid JWT cookie (`token`) issued by the auth endpoints.

The body limit is **100 MB** (configurable in `main.rs`); large file uploads go through `POST /api/books`.

---

## Authentication

### `GET /api/auth/status`

Check whether any user exists. Used by the login page to decide between signup and signin. **Public.**

**Response** (200):
```json
{ "has_users": true }
```

---

### `POST /api/auth/signup`

Create the first (and only) user. Only succeeds when `users` is empty. **Public.**

**Request body:**
```json
{ "password": "your-password" }
```

**Response** (200):
```json
{ "success": true }
```

Sets the `token` httpOnly cookie (30-day expiry).

**Errors:**
- `409 Conflict`: a user already exists
- `500 Internal Server Error`: hashing or DB failure

---

### `POST /api/auth/signin`

Log in with the existing user's password. **Public.**

**Request body:**
```json
{ "password": "your-password" }
```

**Response** (200):
```json
{ "success": true }
```

Sets the `token` httpOnly cookie (30-day expiry).

**Errors:** `401 Unauthorized`: wrong password, or no user yet.

---

### `POST /api/auth/logout`

*Requires auth.*

Clear the auth cookie.

**Response** (200):
```json
{ "success": true }
```

---

### `GET /api/auth/me`

*Requires auth.*

Returns `200 OK` if the session is authenticated. Used by the frontend on layout mount to decide whether to redirect to `/login`.

**Response** (200):
```json
{ "authenticated": true }
```

---

## Health

### `GET /api/health`

**Public.** Liveness check; also pings the database.

**Response** (200):
```json
{ "status": "ok", "db": "connected" }
```

`db` is `"connected"` or an error string.

---

## Books

*All book endpoints require auth, except `GET /api/books/{id}/cover`.*

### `GET /api/books`

List all books, ordered by `updated_at DESC`.

**Response** (200):
```json
[
  {
    "id": "uuid",
    "title": "The Example",
    "author": null,
    "description": null,
    "publisher": null,
    "isbn": null,
    "language": null,
    "cover_path": "covers/abc.jpg",
    "file_path": "books/abc.epub",
    "format": "epub",
    "file_size": 1234567,
    "page_count": null,
    "current_page": null,
    "created_at": "2026-01-01 00:00:00",
    "updated_at": "2026-01-01 00:00:00"
  }
]
```

---

### `POST /api/books`

Upload a book. On upload, the backend extracts the cover image from the EPUB and uploads it as a separate S3 object.

**Request:** `multipart/form-data`
| Field | Description |
|-------|-------------|
| `file` | The book file (`.epub` in v0.1) |

**Response** (200): Created `Book` object (same shape as list item).

**Errors:**
- `503 Service Unavailable`: `{"error":"Storage not configured"}` if R2/S3 credentials are missing
- `400 Bad Request`: no file in form, or invalid EPUB

---

### `GET /api/books/{id}`

Get a single book's metadata.

**Response** (200): `Book` object.

**Errors:** `404 Not Found`: no book with that ID.

---

### `DELETE /api/books/{id}`

Delete a book. Removes the SQLite row, the EPUB, and the cover from S3. Bookmarks and annotations cascade-delete via `ON DELETE CASCADE`.

**Response** (200):
```json
{ "success": true }
```

**Errors:** `404 Not Found`.

---

### `GET /api/books/{id}/file`

Stream the raw book file from S3. Used by epub.js to load the book.

**Response** (200): `application/epub+zip` (or appropriate MIME), body is the file bytes.

**Public**: the auth middleware exempts this route so the EPUB content can be loaded by the reader even if the cookie is being set.

**Errors:** `404 Not Found`.

---

### `GET /api/books/{id}/cover`

Serve the extracted cover image. **Public** (exempt from auth).

**Response** (200): Image bytes with the correct `Content-Type` (`image/jpeg`, `image/png`, etc.).

**Errors:** `404 Not Found`: no book, no cover, or cover extraction failed.

---

### `GET /api/books/{id}/resource/{path}`

Stream an embedded resource (image, font, CSS) out of the EPUB ZIP without re-uploading the whole file. The path is the relative path inside the EPUB (e.g. `OEBPS/images/cover.jpg`).

**Response** (200): The resource bytes with the MIME type inferred from the file extension.

**Errors:** `404 Not Found`: no book or no resource at that path.

---

### `POST /api/books/{id}/progress`

Save the current reading position. Called debounced on every page turn.

**Request body:**
```json
{ "current_page": "epubcfi(/6/14[chap03]!/4/2/1:0)" }
```

`current_page` is a string holding an EPUB CFI. (The DB column is `TEXT` rather than `INTEGER` so it can hold CFIs.)

**Response** (200):
```json
{ "success": true }
```

---

## Bookmarks

*All bookmark endpoints require auth.*

### `GET /api/books/{id}/bookmarks`

List all bookmarks for a book, ordered by `chapter_index` then `created_at`.

**Response** (200):
```json
[
  {
    "id": "uuid",
    "book_id": "uuid",
    "chapter_index": 0,
    "cfi": "epubcfi(/6/2[cover]!/4/2/1:0)",
    "label": "First page",
    "created_at": "2026-01-01 00:00:00"
  }
]
```

---

### `POST /api/books/{id}/bookmarks`

Create a bookmark at the current position.

**Request body:**
```json
{
  "chapter_index": 0,
  "cfi": "epubcfi(/6/2[cover]!/4/2/1:0)",
  "label": "Cool quote"
}
```

`label` is optional.

**Response** (200): Created bookmark object.

---

### `DELETE /api/books/{id}/bookmarks/{bookmark_id}`

Delete a single bookmark.

**Response** (200):
```json
{ "success": true }
```

---

## Annotations

*All annotation endpoints require auth.*

### `GET /api/books/{id}/annotations`

List all annotations for a book, ordered by `chapter_index` then `created_at`.

**Response** (200):
```json
[
  {
    "id": "uuid",
    "book_id": "uuid",
    "chapter_index": 0,
    "cfi": "epubcfi(/6/2[cover]!/4/2/1:0)",
    "text": "highlighted passage",
    "note": "why this is interesting",
    "color": "yellow",
    "created_at": "2026-01-01 00:00:00",
    "updated_at": "2026-01-01 00:00:00"
  }
]
```

---

### `POST /api/books/{id}/annotations`

Create a highlight (with optional note) at the current position.

**Request body:**
```json
{
  "chapter_index": 0,
  "cfi": "epubcfi(/6/2[cover]!/4/2/1:0)",
  "text": "highlighted passage",
  "note": "why this is interesting",
  "color": "yellow"
}
```

`note` and `color` are optional. `color` defaults to `"yellow"` server-side if omitted.

**Response** (200): Created annotation object.

---

### `PUT /api/books/{id}/annotations/{ann_id}`

Update an existing annotation. Used to edit notes or change highlight color.

**Request body:** (any subset of)
```json
{
  "note": "updated note",
  "color": "blue"
}
```

**Response** (200): Updated annotation object.

---

### `DELETE /api/books/{id}/annotations/{ann_id}`

Delete a single annotation.

**Response** (200):
```json
{ "success": true }
```

---

### `GET /api/books/{id}/annotations/export`

Export all annotations for a book as a single Markdown file.

**Response** (200): `text/markdown` body, with `Content-Disposition: attachment; filename="book-title-annotations.md"`. The body looks like:

```markdown
# Annotations from "The Example"

## Chapter 1

> highlighted passage
>
>: *Note: why this is interesting*

---

## Chapter 2

> another quote
```

**Errors:** `404 Not Found`: book not found, or no annotations to export.

---

## Reader Settings

*Require auth.*

### `GET /api/books/{id}/settings`

Get the reader's typography preferences for this book.

**Response** (200):
```json
{
  "fontFamily": "literata",
  "fontSize": 120,
  "lineHeight": 1.5,
  "margin": 20,
  "align": "left",
  "theme": "sepia"
}
```

If no settings have been saved yet, the response is `200` with the `defaultTypography` from `frontend/src/lib/reader/themes.ts`.

---

### `PUT /api/books/{id}/settings`

Save reader settings. Replaces all settings for this book.

**Request body:** (all fields optional except at least one)
```json
{
  "fontFamily": "andika",
  "fontSize": 110,
  "lineHeight": 1.6,
  "margin": 15,
  "align": "justify",
  "theme": "dark"
}
```

Valid `fontFamily` values: `"literata"`, `"andika"`, `"shantell"`, `"noto"`, `"libertinus"`.
Valid `theme` values: `"light"`, `"cream"`, `"sepia"`, `"green"`, `"dark"`, `"night"`.

**Response** (200): Updated settings object.

---

## Conventions

- All IDs are UUID v4 strings.
- Timestamps are SQLite `datetime('now')` strings (`YYYY-MM-DD HH:MM:SS`), UTC.
- Pagination is not yet implemented; lists return all rows. The v0.4 roadmap adds `?limit` and `?offset` to `GET /api/books`.
- All errors return JSON of the form `{"error": "message"}` (where applicable) with the appropriate HTTP status code.
