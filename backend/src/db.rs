use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS books (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT,
            description TEXT,
            publisher TEXT,
            isbn TEXT,
            language TEXT,
            cover_path TEXT,
            file_path TEXT NOT NULL,
            format TEXT NOT NULL DEFAULT 'epub',
            file_size INTEGER,
            page_count INTEGER,
            current_page INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS bookmarks (
            id TEXT PRIMARY KEY,
            book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            chapter_index INTEGER NOT NULL,
            cfi TEXT NOT NULL,
            label TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS annotations (
            id TEXT PRIMARY KEY,
            book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            chapter_index INTEGER NOT NULL,
            cfi TEXT NOT NULL,
            text TEXT NOT NULL,
            note TEXT,
            color TEXT DEFAULT 'yellow',
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );

        CREATE TABLE IF NOT EXISTS book_tags (
            book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (book_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS preferences (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );",
    )
    .expect("failed to run migrations");
}
