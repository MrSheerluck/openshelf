use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use std::sync::Arc;
use uuid::Uuid;

use crate::storage::Storage;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub language: Option<String>,
    pub cover_path: Option<String>,
    pub file_path: String,
    pub format: String,
    pub file_size: Option<i64>,
    pub page_count: Option<i32>,
    pub current_page: Option<String>,
    pub reading_status: Option<String>,
    pub last_opened_at: Option<String>,
    pub progress: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct BookQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub format: Option<String>,
    pub tag: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBookPayload {
    pub title: Option<String>,
    pub author: Option<String>,
    pub reading_status: Option<String>,
}

fn storage(state: &AppState) -> Result<&Storage, (StatusCode, Json<serde_json::Value>)> {
    state.storage.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Storage not configured"})),
    ))
}

fn extract_epub_cover(epub_bytes: &[u8]) -> Option<(Vec<u8>, String)> {
    let cursor = Cursor::new(epub_bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;

    let container_bytes = {
        let mut f = archive.by_name("META-INF/container.xml").ok()?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).ok()?;
        buf
    };

    let mut reader = Reader::from_reader(Cursor::new(&container_bytes));
    reader.config_mut().trim_text(true);
    let mut opf_path = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let elem_name = e.name();
                if elem_name.as_ref() == b"rootfile" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"full-path" {
                            opf_path = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    eprintln!("[cover] OPF path: {opf_path:?}");
    if opf_path.is_empty() {
        return None;
    }

    let opf_bytes = {
        let mut f = archive.by_name(&opf_path).ok()?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).ok()?;
        buf
    };

    let opf_dir = std::path::Path::new(&opf_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut items: Vec<(String, String, String)> = Vec::new();
    let mut cover_id: Option<String> = None;
    let mut cover_property_href: Option<String> = None;
    let mut buf = Vec::new();

    let mut reader = Reader::from_reader(Cursor::new(&opf_bytes));
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag_name = e.name();
                let tag_name = tag_name.as_ref();

                if tag_name == b"meta" {
                    let mut name = String::new();
                    let mut prop = String::new();
                    let mut content = String::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value);
                        match key.as_ref() {
                            "name" => name = val.to_string(),
                            "property" => prop = val.to_string(),
                            "content" => content = val.to_string(),
                            _ => {}
                        }
                    }
                    if (name == "cover" || prop == "cover-image") && !content.is_empty() {
                        cover_id = Some(content.clone());
                        eprintln!("[cover] found cover meta: {name}/{prop} -> {content}");
                    }
                }

                if tag_name == b"item" {
                    let mut id = String::new();
                    let mut href = String::new();
                    let mut properties = String::new();
                    let mut media_type = String::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value);
                        match key.as_ref() {
                            "id" => id = val.to_string(),
                            "href" => href = val.to_string(),
                            "properties" => properties = val.to_string(),
                            "media-type" => media_type = val.to_string(),
                            _ => {}
                        }
                    }
                    if !href.is_empty() {
                        if properties.split_whitespace().any(|p| p == "cover-image") {
                            cover_property_href = Some(href.clone());
                            eprintln!("[cover] found cover item by property: {href}");
                        }
                        items.push((id, href, media_type));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    eprintln!("[cover] total items in manifest: {}", items.len());
    eprintln!(
        "[cover] image items: {:?}",
        items
            .iter()
            .filter(|(_, _, mt)| mt.starts_with("image/"))
            .collect::<Vec<_>>()
    );

    let cover_href = if let Some(href) = cover_property_href {
        Some(href)
    } else if let Some(ref id) = cover_id {
        items
            .iter()
            .find(|(item_id, _, _)| item_id == id)
            .map(|(_, href, _)| href.clone())
    } else {
        None
    }
    .or_else(|| {
        items
            .iter()
            .find(|(id, href, _)| {
                let lower_id = id.to_lowercase();
                let lower_href = href.to_lowercase();
                lower_id.contains("cover")
                    || lower_href.contains("cover")
                    || lower_href.starts_with("cover.")
            })
            .map(|(_, href, _)| href.clone())
    })?;

    eprintln!("[cover] resolved cover_href: {cover_href}");

    let cover_path = if opf_dir.is_empty() {
        cover_href.clone()
    } else {
        format!("{}/{}", opf_dir, cover_href)
    };

    eprintln!("[cover] archive path: {cover_path}");

    let cover_bytes = {
        let mut f = archive.by_name(&cover_path).ok()?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).ok()?;
        buf
    };

    let media_type = mime_guess::from_path(&cover_href)
        .first_or_octet_stream()
        .to_string();

    Some((cover_bytes, media_type))
}

pub async fn upload_book(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let storage = storage(&state)?;

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = String::new();
    let mut content_type = String::new();
    let mut title = String::new();
    let mut author: Option<String> = None;
    let mut cover_upload: Option<(Vec<u8>, String)> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        eprintln!("[upload] processing field: {name}");
        match name.as_str() {
            "file" => {
                content_type = field
                    .content_type()
                    .unwrap_or("application/epub+zip")
                    .to_string();
                filename = field.file_name().unwrap_or("untitled.epub").to_string();
                match field.bytes().await {
                    Ok(data) => {
                        let len = data.len();
                        file_bytes = Some(data.to_vec());
                        eprintln!(
                            "[upload] received file: {filename} ({len} bytes, type={content_type})"
                        );
                    }
                    Err(e) => {
                        eprintln!("[upload] error reading file bytes: {e}");
                        file_bytes = Some(Vec::new());
                    }
                }
            }
            "cover" => {
                let ct = field.content_type().unwrap_or("image/jpeg").to_string();
                match field.bytes().await {
                    Ok(data) => {
                        let len = data.len();
                        eprintln!("[upload] received cover image: {len} bytes, type={ct}");
                        cover_upload = Some((data.to_vec(), ct));
                    }
                    Err(e) => {
                        eprintln!("[upload] error reading cover bytes: {e}");
                    }
                }
            }
            "title" => {
                title = field.text().await.unwrap_or_default();
                eprintln!("[upload] title: {title}");
            }
            "author" => {
                let text = field.text().await.unwrap_or_default();
                eprintln!("[upload] author: {text}");
                if !text.is_empty() {
                    author = Some(text);
                }
            }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        eprintln!("[upload] ERROR: no file bytes");
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No file provided"})),
        )
    })?;

    if bytes.is_empty() {
        eprintln!("[upload] ERROR: file is empty");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "File is empty"})),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let format = if filename.ends_with(".epub") {
        "epub"
    } else if filename.ends_with(".mobi") {
        "mobi"
    } else {
        "epub"
    };

    if title.is_empty() {
        title = filename
            .strip_suffix(".epub")
            .or_else(|| filename.strip_suffix(".mobi"))
            .unwrap_or(&filename)
            .to_string();
    }

    let file_path = format!("books/{}/{}", id, filename);
    let file_size = bytes.len() as i64;

    let cover_result = if format == "epub" {
        eprintln!("[upload] attempting cover extraction");
        let result = extract_epub_cover(&bytes);
        if result.is_some() {
            eprintln!("[upload] cover extracted");
        } else {
            eprintln!("[upload] no cover found in EPUB");
        }
        result
    } else {
        None
    };

    eprintln!("[upload] uploading file to S3: {file_path} ({file_size} bytes)");
    storage
        .put(&file_path, bytes, &content_type)
        .await
        .map_err(|e| {
            eprintln!("[upload] ERROR: S3 upload failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "S3 upload failed"})),
            )
        })?;
    eprintln!("[upload] file uploaded successfully");

    let cover_path = if let Some((cover_data, cover_mime)) = cover_upload {
        let cover_key = format!("books/{}/cover.jpg", id);
        eprintln!("[upload] uploading provided cover to S3: {cover_key}");
        if storage
            .put(&cover_key, cover_data, &cover_mime)
            .await
            .is_ok()
        {
            eprintln!("[upload] cover uploaded successfully");
            Some(cover_key)
        } else {
            eprintln!("[upload] cover upload failed (non-fatal)");
            None
        }
    } else if let Some((cover_data, cover_mime)) = cover_result {
        let cover_key = format!("books/{}/cover.jpg", id);
        eprintln!("[upload] uploading extracted cover to S3: {cover_key}");
        if storage
            .put(&cover_key, cover_data, &cover_mime)
            .await
            .is_ok()
        {
            eprintln!("[upload] cover uploaded successfully");
            Some(cover_key)
        } else {
            eprintln!("[upload] cover upload failed (non-fatal)");
            None
        }
    } else {
        None
    };

    eprintln!("[upload] inserting into DB");
    let db = state.db.lock().await;
    db.execute(
        "INSERT INTO books (id, title, author, cover_path, file_path, format, file_size) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, title, author, cover_path, file_path, format, file_size],
    )
    .map_err(|e| {
        eprintln!("[upload] ERROR: DB insert failed: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"})))
    })?;
    eprintln!("[upload] book {id} inserted successfully");

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

fn book_from_row(row: &rusqlite::Row) -> rusqlite::Result<Book> {
    let tags_str: String = row.get(18)?;
    let tags: Vec<String> = if tags_str.is_empty() {
        Vec::new()
    } else {
        tags_str.split("||").map(|s| s.to_string()).collect()
    };
    Ok(Book {
        id: row.get(0)?,
        title: row.get(1)?,
        author: row.get(2)?,
        description: row.get(3)?,
        publisher: row.get(4)?,
        isbn: row.get(5)?,
        language: row.get(6)?,
        cover_path: row.get(7)?,
        file_path: row.get(8)?,
        format: row.get(9)?,
        file_size: row.get(10)?,
        page_count: row.get(11)?,
        current_page: row.get(12)?,
        reading_status: row.get(13)?,
        last_opened_at: row.get(14)?,
        progress: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        tags,
    })
}

const BOOK_SELECT_COLS: &str =
    "b.id, b.title, b.author, b.description, b.publisher, b.isbn, b.language, \
     b.cover_path, b.file_path, b.format, b.file_size, b.page_count, b.current_page, \
     b.reading_status, b.last_opened_at, b.progress, b.created_at, b.updated_at, \
     COALESCE(GROUP_CONCAT(t.name, '||'), '')";

const BOOK_FROM: &str = "FROM books b \
     LEFT JOIN book_tags bt ON bt.book_id = b.id \
     LEFT JOIN tags t ON t.id = bt.tag_id";

pub async fn list_books(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BookQuery>,
) -> Result<Json<Vec<Book>>, StatusCode> {
    let db = state.db.lock().await;

    let sort_field = match query.sort.as_deref() {
        Some("title") => "b.title",
        Some("author") => "b.author",
        Some("created_at") => "b.created_at",
        Some("updated_at") => "b.updated_at",
        Some("progress") => "b.progress",
        _ => "b.created_at",
    };
    let order_dir = match query.order.as_deref() {
        Some("asc") | Some("ASC") => "ASC",
        _ => "DESC",
    };

    let mut sql = format!("SELECT {BOOK_SELECT_COLS} {BOOK_FROM}");
    let mut conditions: Vec<String> = Vec::new();
    let mut param_values: Vec<String> = Vec::new();
    let mut param_idx = 1;

    if let Some(ref search) = query.search {
        let s = format!("%{}%", search);
        conditions.push(format!(
            "(b.title LIKE ?{p} OR b.author LIKE ?{p})",
            p = param_idx
        ));
        param_values.push(s);
        param_idx += 1;
    }

    if let Some(ref status) = query.status {
        if !status.is_empty() {
            conditions.push(format!("b.reading_status = ?{p}", p = param_idx));
            param_values.push(status.clone());
            param_idx += 1;
        }
    }

    if let Some(ref fmt) = query.format {
        if !fmt.is_empty() {
            conditions.push(format!("b.format = ?{p}", p = param_idx));
            param_values.push(fmt.clone());
            param_idx += 1;
        }
    }

    if let Some(ref tag) = query.tag {
        if !tag.is_empty() {
            conditions.push(format!(
                "b.id IN (SELECT bt2.book_id FROM book_tags bt2 JOIN tags t2 ON t2.id = bt2.tag_id WHERE t2.name = ?{p})",
                p = param_idx
            ));
            param_values.push(tag.clone());
        }
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" GROUP BY b.id");
    sql.push_str(&format!(" ORDER BY {sort_field} {order_dir}"));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
        .iter()
        .map(|v| v as &dyn rusqlite::types::ToSql)
        .collect();

    let mut stmt = match db.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[list_books] prepare error: {e}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let rows = match stmt.query_map(param_refs.as_slice(), book_from_row) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[list_books] query error: {e}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut books: Vec<Book> = Vec::new();
    for row in rows {
        match row {
            Ok(b) => books.push(b),
            Err(e) => eprintln!("[list_books] skipping bad row: {e}"),
        }
    }

    Ok(Json(books))
}

pub async fn get_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Book>, StatusCode> {
    let db = state.db.lock().await;

    let sql = format!("SELECT {BOOK_SELECT_COLS} {BOOK_FROM} WHERE b.id = ?1 GROUP BY b.id");

    let book = db
        .query_row(&sql, params![id], book_from_row)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(book))
}

pub async fn update_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateBookPayload>,
) -> Result<Json<Book>, StatusCode> {
    let db = state.db.lock().await;
    // When reading_status is Some("") it means "clear the status".
    // When None it means "don't change".
    // CASE: if param IS NULL → keep existing; if param = '' → set to NULL; else set to param.
    db.execute(
        "UPDATE books SET
            title = COALESCE(?1, title),
            author = COALESCE(?2, author),
            reading_status = CASE WHEN ?3 IS NULL THEN reading_status WHEN ?3 = '' THEN NULL ELSE ?3 END,
            updated_at = datetime('now')
         WHERE id = ?4",
        params![body.title, body.author, body.reading_status, id],
    )
    .map_err(|e| {
        eprintln!("[update_book] error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let sql = format!("SELECT {BOOK_SELECT_COLS} {BOOK_FROM} WHERE b.id = ?1 GROUP BY b.id");
    let book = db
        .query_row(&sql, params![id], book_from_row)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(book))
}

pub async fn touch_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let db = state.db.lock().await;
    db.execute(
        "UPDATE books SET last_opened_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn serve_book_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let storage = storage(&state).map_err(|e| e.0)?;

    let db = state.db.lock().await;
    let file_path: String = db
        .query_row(
            "SELECT file_path FROM books WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;
    drop(db);

    let bytes = storage
        .get(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mime = "application/epub+zip";

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .body(bytes.into())
        .unwrap())
}

pub async fn serve_book_cover(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let storage = storage(&state).map_err(|e| e.0)?;

    let db = state.db.lock().await;
    let cover_path: Option<String> = db
        .query_row(
            "SELECT cover_path FROM books WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;
    drop(db);

    let cover_path = cover_path.ok_or(StatusCode::NOT_FOUND)?;

    let bytes = storage
        .get(&cover_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mime = mime_guess::from_path(&cover_path)
        .first_or_octet_stream()
        .to_string();

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(bytes.into())
        .unwrap())
}

pub async fn serve_book_resource(
    State(state): State<Arc<AppState>>,
    Path((id, raw_path)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    let storage = storage(&state).map_err(|e| e.0)?;

    // Reject any traversal attempts and normalize the requested path.
    let normalized: String = raw_path
        .split('/')
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect::<Vec<_>>()
        .join("/");
    if normalized.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Locate the EPUB in storage.
    let db = state.db.lock().await;
    let file_path: String = db
        .query_row(
            "SELECT file_path FROM books WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;
    drop(db);

    // Pull the EPUB bytes, caching them per-book so a 30-image book costs
    // exactly one S3 fetch rather than 30.
    let zip_bytes = {
        let cache = state.epub_cache.lock().await;
        if let Some(hit) = cache.get(&id) {
            hit.clone()
        } else {
            drop(cache);
            let fetched = storage
                .get(&file_path)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let arc = Arc::new(fetched);
            state
                .epub_cache
                .lock()
                .await
                .insert(id.clone(), arc.clone());
            arc
        }
    };

    let cursor = Cursor::new(zip_bytes.as_ref());
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Try an exact-name match first, then fall back to a case-insensitive
    // lookup because some authoring tools emit inconsistent case between the
    // OPF manifest and the XHTML references.
    let mut chosen: Option<String> = None;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let name = entry.name().to_string();
        if name == normalized {
            chosen = Some(name);
            break;
        }
    }
    if chosen.is_none() {
        let lower = normalized.to_lowercase();
        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if entry.name().to_lowercase() == lower {
                chosen = Some(entry.name().to_string());
                break;
            }
        }
    }
    let entry_name = chosen.ok_or(StatusCode::NOT_FOUND)?;
    let mut entry = archive
        .by_name(&entry_name)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut data = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mime = mime_guess::from_path(&entry_name)
        .first_or_octet_stream()
        .to_string();

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(data.into())
        .unwrap())
}

#[derive(Deserialize)]
pub struct ProgressUpdate {
    pub cfi: String,
    pub progress: Option<f64>,
}

pub async fn save_progress(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<ProgressUpdate>,
) -> Result<StatusCode, StatusCode> {
    let db = state.db.lock().await;
    db.execute(
        "UPDATE books SET current_page = ?1, progress = COALESCE(?2, progress), updated_at = datetime('now') WHERE id = ?3",
        params![body.cfi, body.progress, id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- Annotations (highlights + notes) ----

#[derive(Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub chapter_index: i64,
    pub cfi: String,
    pub text: String,
    pub note: Option<String>,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateAnnotation {
    pub chapter_index: Option<i64>,
    pub cfi: String,
    pub text: String,
    pub note: Option<String>,
    pub color: String,
}

#[derive(Deserialize)]
pub struct UpdateAnnotation {
    pub note: Option<String>,
    pub color: Option<String>,
}

pub async fn list_annotations(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Annotation>>, StatusCode> {
    let db = state.db.lock().await;
    let mut stmt = db
        .prepare(
            "SELECT id, chapter_index, cfi, text, note, color, created_at, updated_at FROM annotations WHERE book_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map(params![id], |row| {
            Ok(Annotation {
                id: row.get(0)?,
                chapter_index: row.get(1)?,
                cfi: row.get(2)?,
                text: row.get(3)?,
                note: row.get(4)?,
                color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let annotations: Vec<Annotation> = rows.filter_map(|r| r.ok()).collect();
    Ok(Json(annotations))
}

pub async fn create_annotation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<CreateAnnotation>,
) -> Result<Json<Annotation>, StatusCode> {
    let db = state.db.lock().await;
    let ann_id = Uuid::new_v4().to_string();
    let chapter_index = body.chapter_index.unwrap_or(0);
    db.execute(
        "INSERT INTO annotations (id, book_id, chapter_index, cfi, text, note, color) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![ann_id, id, chapter_index, body.cfi, body.text, body.note, body.color],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db
        .query_row(
            "SELECT id, chapter_index, cfi, text, note, color, created_at, updated_at FROM annotations WHERE id = ?1",
            params![ann_id],
            |row| {
                Ok(Annotation {
                    id: row.get(0)?,
                    chapter_index: row.get(1)?,
                    cfi: row.get(2)?,
                    text: row.get(3)?,
                    note: row.get(4)?,
                    color: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(row))
}

pub async fn update_annotation(
    State(state): State<Arc<AppState>>,
    Path((id, ann_id)): Path<(String, String)>,
    Json(body): Json<UpdateAnnotation>,
) -> Result<StatusCode, StatusCode> {
    let db = state.db.lock().await;
    db.execute(
        "UPDATE annotations SET note = COALESCE(?1, note), color = COALESCE(?2, color), updated_at = datetime('now') WHERE id = ?3 AND book_id = ?4",
        params![body.note, body.color, ann_id, id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_annotation(
    State(state): State<Arc<AppState>>,
    Path((id, ann_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let db = state.db.lock().await;
    db.execute(
        "DELETE FROM annotations WHERE id = ?1 AND book_id = ?2",
        params![ann_id, id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- Export annotations ----

pub async fn export_annotations(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<ExportParams>,
) -> Result<Response, StatusCode> {
    let format = params.format.as_deref().unwrap_or("json");
    let db = state.db.lock().await;

    let book_title: String = db
        .query_row(
            "SELECT title FROM books WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let annotations: Vec<Annotation> = {
        let mut stmt = db
            .prepare("SELECT id, chapter_index, cfi, text, note, color, created_at FROM annotations WHERE book_id = ?1 ORDER BY created_at ASC")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let rows = stmt
            .query_map(params![id], |row| {
                Ok(Annotation {
                    id: row.get(0)?,
                    chapter_index: row.get(1)?,
                    cfi: row.get(2)?,
                    text: row.get(3)?,
                    note: row.get(4)?,
                    color: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        rows.filter_map(|r| r.ok()).collect()
    };
    drop(db);

    if format == "markdown" || format == "md" {
        let mut md = format!("# Highlights: {}\n\n", book_title);
        for a in &annotations {
            md.push_str(&format!("## Chapter {}\n\n", a.chapter_index + 1));
            md.push_str(&format!("> {}\n\n", a.text.replace('\n', "\n> ")));
            if let Some(note) = &a.note {
                if !note.is_empty() {
                    md.push_str(&format!("**Note:** {}\n\n", note));
                }
            }
            md.push_str(&format!(
                "*Color: {} · Created: {}*\n\n---\n\n",
                a.color, a.created_at
            ));
        }
        if annotations.is_empty() {
            md.push_str("No highlights yet.\n");
        }
        Ok(Response::builder()
            .header(header::CONTENT_TYPE, "text/markdown; charset=utf-8")
            .header(
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=\"{}-highlights.md\"",
                    sanitize_filename(&book_title)
                ),
            )
            .body(md.into())
            .unwrap())
    } else {
        let json = serde_json::json!({
            "book": book_title,
            "exported_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            "highlights": annotations.iter().map(|a| serde_json::json!({
                "id": a.id,
                "chapter_index": a.chapter_index,
                "text": a.text,
                "note": a.note,
                "color": a.color,
                "cfi": a.cfi,
                "created_at": a.created_at,
            })).collect::<Vec<_>>(),
        });
        let body =
            serde_json::to_string_pretty(&json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Response::builder()
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=\"{}-highlights.json\"",
                    sanitize_filename(&book_title)
                ),
            )
            .body(body.into())
            .unwrap())
    }
}

#[derive(Deserialize)]
pub struct ExportParams {
    pub format: Option<String>,
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub chapter_index: i64,
    pub cfi: String,
    pub label: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateBookmark {
    pub chapter_index: i64,
    pub cfi: String,
    pub label: Option<String>,
}

pub async fn list_bookmarks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Bookmark>>, StatusCode> {
    let db = state.db.lock().await;
    let mut stmt = db
        .prepare(
            "SELECT id, chapter_index, cfi, label, created_at FROM bookmarks WHERE book_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map(params![id], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                chapter_index: row.get(1)?,
                cfi: row.get(2)?,
                label: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.filter_map(|r| r.ok()).collect()))
}

pub async fn create_bookmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<CreateBookmark>,
) -> Result<Json<Bookmark>, StatusCode> {
    let db = state.db.lock().await;
    let existing = db
        .query_row(
            "SELECT id, chapter_index, cfi, label, created_at FROM bookmarks WHERE book_id = ?1 AND cfi = ?2",
            params![id, body.cfi],
            |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    chapter_index: row.get(1)?,
                    cfi: row.get(2)?,
                    label: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .ok();
    if let Some(bookmark) = existing {
        return Ok(Json(bookmark));
    }

    let bookmark_id = Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO bookmarks (id, book_id, chapter_index, cfi, label) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![bookmark_id, id, body.chapter_index, body.cfi, body.label],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bookmark = db
        .query_row(
            "SELECT id, chapter_index, cfi, label, created_at FROM bookmarks WHERE id = ?1",
            params![bookmark_id],
            |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    chapter_index: row.get(1)?,
                    cfi: row.get(2)?,
                    label: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(bookmark))
}

pub async fn delete_bookmark(
    State(state): State<Arc<AppState>>,
    Path((id, bookmark_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let db = state.db.lock().await;
    db.execute(
        "DELETE FROM bookmarks WHERE id = ?1 AND book_id = ?2",
        params![bookmark_id, id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- Reader settings sync ----

#[derive(Serialize, Deserialize)]
pub struct ReaderSettingsPayload {
    pub theme: String,
    pub font_family: String,
    pub font_size: i64,
}

pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Option<ReaderSettingsPayload>>, StatusCode> {
    let db = state.db.lock().await;
    let result: Result<Option<String>, _> = db.query_row(
        "SELECT value FROM preferences WHERE key = ?1",
        params![format!("reader_settings:{}", id)],
        |row| row.get(0),
    );
    match result {
        Ok(Some(json_str)) => {
            let payload: ReaderSettingsPayload =
                serde_json::from_str(&json_str).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(Some(payload)))
        }
        Ok(None) => Ok(Json(None)),
        Err(_) => Ok(Json(None)),
    }
}

pub async fn save_settings(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<ReaderSettingsPayload>,
) -> Result<StatusCode, StatusCode> {
    let db = state.db.lock().await;
    let json_str = serde_json::to_string(&body).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    db.execute(
        "INSERT INTO preferences (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![format!("reader_settings:{}", id), json_str],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct SpineItem {
    pub href: String,
    pub id: String,
    pub media_type: String,
}

/// Lightweight endpoint that returns the EPUB spine (ordered table of contents
/// manifest entries) by parsing only the OPF — no full-file download needed.
pub async fn spine_info(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<SpineItem>>, StatusCode> {
    let storage = storage(&state).map_err(|e| e.0)?;

    let db = state.db.lock().await;
    let file_path: String = db
        .query_row(
            "SELECT file_path FROM books WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;
    drop(db);

    let zip_bytes = {
        let cache = state.epub_cache.lock().await;
        if let Some(hit) = cache.get(&id) {
            hit.clone()
        } else {
            drop(cache);
            let fetched = storage
                .get(&file_path)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let arc = Arc::new(fetched);
            state
                .epub_cache
                .lock()
                .await
                .insert(id.clone(), arc.clone());
            arc
        }
    };

    let cursor = Cursor::new(zip_bytes.as_ref());
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Read container.xml → locate OPF
    let container_bytes = {
        let mut f = archive
            .by_name("META-INF/container.xml")
            .map_err(|_| StatusCode::NOT_FOUND)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).ok();
        buf
    };

    let mut reader = Reader::from_reader(Cursor::new(&container_bytes));
    reader.config_mut().trim_text(true);
    let mut opf_path = String::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let ename = e.name();
                if ename.as_ref() == b"rootfile" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"full-path" {
                            opf_path = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    if opf_path.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let opf_bytes = {
        let mut f = archive
            .by_name(&opf_path)
            .map_err(|_| StatusCode::NOT_FOUND)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).ok();
        buf
    };

    // Parse manifest items
    let mut manifest: Vec<(String, String, String)> = Vec::new(); // (id, href, media_type)
    let mut spine_refs: Vec<String> = Vec::new(); // ordered idrefs

    let mut reader = Reader::from_reader(Cursor::new(&opf_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_manifest = false;
    let mut in_spine = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag_name = e.name();
                let tag = tag_name.as_ref();
                if tag == b"manifest" {
                    in_manifest = true;
                }
                if tag == b"spine" {
                    in_spine = true;
                }
                if in_manifest && tag == b"item" {
                    let mut id = String::new();
                    let mut href = String::new();
                    let mut media_type = String::new();
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let val = std::str::from_utf8(&attr.value).unwrap_or("");
                        match key {
                            "id" => id = val.to_string(),
                            "href" => href = val.to_string(),
                            "media-type" => media_type = val.to_string(),
                            _ => {}
                        }
                    }
                    if !id.is_empty() && !href.is_empty() {
                        manifest.push((id, href, media_type));
                    }
                }
                if in_spine && tag == b"itemref" {
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let val = std::str::from_utf8(&attr.value).unwrap_or("");
                        if key == "idref" {
                            spine_refs.push(val.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = e.name();
                let tag = tag_name.as_ref();
                if tag == b"manifest" {
                    in_manifest = false;
                }
                if tag == b"spine" {
                    in_spine = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // Join spine idrefs with manifest entries in spine order
    let lookup: std::collections::HashMap<&str, (&str, &str)> = manifest
        .iter()
        .map(|(id, href, mt)| (id.as_str(), (href.as_str(), mt.as_str())))
        .collect();

    let result: Vec<SpineItem> = spine_refs
        .iter()
        .filter_map(|idref| {
            lookup.get(idref.as_str()).map(|(href, mt)| SpineItem {
                id: idref.clone(),
                href: (*href).to_string(),
                media_type: (*mt).to_string(),
            })
        })
        .collect();

    if result.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(result))
}

pub async fn delete_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let db = state.db.lock().await;

    let (file_path, cover_path): (String, Option<String>) = db
        .query_row(
            "SELECT file_path, cover_path FROM books WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    db.execute("DELETE FROM books WHERE id = ?1", params![id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    drop(db);

    if let Ok(storage) = storage(&state) {
        storage.delete(&file_path).await.ok();
        if let Some(cover) = cover_path {
            storage.delete(&cover).await.ok();
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── Tags ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct TagCount {
    pub name: String,
    pub count: i64,
}

pub async fn list_all_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TagCount>>, StatusCode> {
    let db = state.db.lock().await;
    let mut stmt = db
        .prepare(
            "SELECT t.name, COUNT(bt.book_id) as cnt \
             FROM tags t \
             LEFT JOIN book_tags bt ON bt.tag_id = t.id \
             GROUP BY t.id \
             ORDER BY t.name ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TagCount {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.filter_map(|r| r.ok()).collect()))
}

pub async fn list_book_tags(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let db = state.db.lock().await;
    let mut stmt = db
        .prepare(
            "SELECT t.name FROM tags t \
             JOIN book_tags bt ON bt.tag_id = t.id \
             WHERE bt.book_id = ?1 \
             ORDER BY t.name ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map(params![id], |row| row.get::<_, String>(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.filter_map(|r| r.ok()).collect()))
}

pub async fn set_book_tags(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(tag_names): Json<Vec<String>>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let db = state.db.lock().await;

    // Remove all existing tags for this book
    db.execute("DELETE FROM book_tags WHERE book_id = ?1", params![id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for name in &tag_names {
        let trimmed = name.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        // Insert tag if it doesn't exist
        db.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
            params![Uuid::new_v4().to_string(), trimmed],
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Get tag id
        let tag_id: String = db
            .query_row(
                "SELECT id FROM tags WHERE name = ?1",
                params![trimmed],
                |row| row.get(0),
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Link book to tag
        db.execute(
            "INSERT OR IGNORE INTO book_tags (book_id, tag_id) VALUES (?1, ?2)",
            params![id, tag_id],
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    drop(db);
    list_book_tags(State(state), Path(id)).await
}

// ── Stats ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Stats {
    pub total_books: i64,
    pub finished_books: i64,
    pub reading_books: i64,
    pub want_to_read: i64,
    pub total_highlights: i64,
}

pub async fn get_stats(State(state): State<Arc<AppState>>) -> Result<Json<Stats>, StatusCode> {
    let db = state.db.lock().await;

    let total_books: i64 = db
        .query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))
        .unwrap_or(0);

    let finished_books: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM books WHERE reading_status = 'finished'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let reading_books: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM books WHERE reading_status = 'reading'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let want_to_read: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM books WHERE reading_status = 'want_to_read'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_highlights: i64 = db
        .query_row("SELECT COUNT(*) FROM annotations", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(Json(Stats {
        total_books,
        finished_books,
        reading_books,
        want_to_read,
        total_highlights,
    }))
}
