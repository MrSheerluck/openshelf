use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) {
    let current_page_type: String = conn
        .query_row(
            "SELECT type FROM pragma_table_info('books') WHERE name = 'current_page'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

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
            current_page TEXT,
            reading_status TEXT,
            last_opened_at TEXT,
            progress REAL DEFAULT 0.0,
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
            name UNIQUE NOT NULL
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

    if current_page_type.eq_ignore_ascii_case("integer") {
        eprintln!("[migrate] converting books.current_page from INTEGER to TEXT");
        match conn.execute_batch(
            "ALTER TABLE books ADD COLUMN current_page_new TEXT;
             UPDATE books SET current_page_new = CAST(current_page AS TEXT);
             ALTER TABLE books DROP COLUMN current_page;
             ALTER TABLE books RENAME COLUMN current_page_new TO current_page;",
        ) {
            Ok(_) => eprintln!("[migrate] conversion complete"),
            Err(e) => eprintln!("[migrate] conversion failed: {e}"),
        }
    }

    // Add new columns if they don't exist (for existing databases)
    for (col, def) in [
        ("reading_status", "TEXT"),
        ("last_opened_at", "TEXT"),
        ("progress", "REAL DEFAULT 0.0"),
    ] {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('books') WHERE name = ?1",
                rusqlite::params![col],
                |row| row.get(0),
            )
            .unwrap_or(false);
        if !exists {
            eprintln!("[migrate] adding column books.{col}");
            conn.execute_batch(&format!("ALTER TABLE books ADD COLUMN {col} {def};"))
                .unwrap_or_else(|e| eprintln!("[migrate] failed to add {col}: {e}"));
        }
    }
}
