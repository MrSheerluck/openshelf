# Features

This page describes what OpenShelf does today and what's planned. For the precise current state, see the checkboxes in [ROADMAP.md](../ROADMAP.md).

## Library

- **EPUB upload**: drag-and-drop or file picker. Files are stored in S3-compatible storage (Cloudflare R2 or Garage), metadata in SQLite.
- **Cover extraction**: when an EPUB is uploaded, the backend parses `META-INF/container.xml` → the OPF package document → the cover image manifest entry, and uploads the cover as a separate S3 object. Covers show in the library grid without a separate render step.
- **Metadata**: title, author, description, publisher, ISBN, language are stored in the `books` table (currently set to `null` from uploads: the next version extracts them from the OPF `<metadata>` block).
- **Delete**: removes the SQLite row, the cover, and the EPUB from S3 in one call. Bookmarks and annotations cascade-delete via `ON DELETE CASCADE`.
- **Search & sort** *(planned for v0.3)*: filter by title, author, and reading state.

## Reader

- **epub.js** in the browser. The backend serves the raw EPUB bytes; the browser parses, paginates, and themes.
- **Paginated reading**: fixed page count per spread, with a CSS column layout underneath. CFI-based pagination means the same position is recoverable across reloads.
- **Page navigation**: prev/next buttons, keyboard arrows, touch swipe.
- **Chapter navigation**: jump to any spine item by CFI.
- **Reading progress**: current CFI is debounce-saved to `POST /api/books/{id}/progress` after every page turn. Reopening the book jumps to the saved position.
- **Resource proxying**: embedded images, fonts, and CSS inside the EPUB are fetched through `GET /api/books/{id}/resource/{path}`, which streams the relevant entry out of the ZIP without re-uploading the whole file.
- **Covers**: `GET /api/books/{id}/cover` returns the extracted cover image (and is exempt from auth, so the login page can show a library preview in the future).

## Themes

Six built-in themes, switchable from the reader toolbar.

| Theme | Background | Text | Notes |
|-------|-----------|------|-------|
| White  | `#ffffff` | `#1a1a1a` | Bright default |
| Cream  | `#f7f3e9` | `#3a3226` | Warm off-white |
| Sepia  | `#f4ecd8` | `#3a2f1c` | Classic paper |
| Mint   | `#e4ede4` | `#2d3a2d` | Soft green |
| Dark   | `#1a1a1a` | `#d4d4d4` | Neutral dark |
| Night  | `#0a0a0a` | `#c0c0c0` | Pure black |

Themes are applied via inline `style` attributes on the reader container so they override any CSS in the EPUB itself.

## Typography

Per-book reader typography, persisted to the server.

- **Font family**: Literata (serif default), Andika, Shantell Sans, Noto Sans, Libertinus Mono. All bundled via `@fontsource`: no external requests.
- **Font size**: 0.5× to 2.0× of the base size, controlled by a slider (default: 120%).
- **Line height**: adjustable, default 1.5.
- **Margins**: left/right page margin as a percentage (default 20%).
- **Alignment**: left, justify.

Stored via `GET`/`PUT /api/books/{id}/settings` as a JSON blob. Falls back to `defaultTypography` from `frontend/src/lib/reader/themes.ts`.

## Bookmarks

- **Create**: bookmark button in the reader toolbar stamps the current `(chapter_index, cfi)`.
- **List**: sidebar panel shows all bookmarks for the current book, with chapter and an excerpt.
- **Jump**: click a bookmark to navigate to its CFI.
- **Delete**: remove individual bookmarks; the book record and other bookmarks are unaffected.

## Highlights & Notes

- **Selection**: select text in the reader; a popover offers highlight colors and an optional note.
- **Colors**: yellow (default), green, blue, pink. Tinted background is rendered over the highlighted text.
- **Notes**: optional text attached to a highlight. Edits sync to the server.
- **List**: highlights panel shows all highlights with surrounding context and the note (if any).
- **Jump**: click a highlight to navigate to its CFI.
- **Export**: `GET /api/books/{id}/annotations/export` returns all annotations for a book as a single Markdown file (chapter heading + quote + note), suitable for import into other tools.

## Authentication

- **Single password**: no email, no username, no recovery.
- **argon2 hashing**: the password is hashed with a random salt; only the PHC string is stored.
- **First-run signup**: the first visit hits `POST /api/auth/signup`, which only succeeds when the `users` table is empty. Subsequent signups return 409 Conflict.
- **JWT cookie**: 30-day httpOnly, `SameSite=Lax` cookie issued on signup/signin.
- **Logout**: clears the cookie.

## Roadmap

Highlights from [ROADMAP.md](../ROADMAP.md):

- **v0.2: Core EPUB Reading**: full metadata extraction (title/author/ISBN from OPF), per-chapter content serving, cover image extraction (already shipped), paginated reader with themes, typography controls, library grid, drag-and-drop upload.
- **v0.3: Reading Features**: server-side bookmark/annotation CRUD (UI exists; some endpoints are already shipped), FTS5 search, tag management, preference sync across devices.
- **v0.4: Polish**: Apple Books-inspired UI, keyboard shortcuts, table of contents panel, offline reading via Service Worker, bulk import/export.
- **v1.0: MOBI**: Kindle format support via `mobi` crate.
- **v1.2: CBZ/CBR**: comic book support, page-by-page image rendering, right-to-left manga mode.
- **v2.0: Future**: audiobooks, reading stats, collections/series, OPDS feed, multi-user, TTS, dictionary lookup.

See the full checklist in [ROADMAP.md](../ROADMAP.md).

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `←` / `→` | Previous / next page |
| `Space` | Next page |
| `Esc` | Close reader, return to library |
| `F` | Toggle fullscreen (planned) |
| `Cmd+,` | Open settings (planned) |

> **Note:** `Cmd+W` / `Ctrl+W` is reserved by browsers for closing the tab.
