use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use rusqlite::params;
use serde::Serialize;
use std::io::{Cursor, Read};
use std::sync::Arc;
use uuid::Uuid;

use crate::storage::Storage;
use crate::AppState;

#[derive(Serialize)]
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
    pub current_page: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
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
            Ok(Event::Start(ref e)) => {
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

    let mut cover_id = String::new();
    let mut cover_href = String::new();
    let mut seen_cover_id = false;

    let mut reader = Reader::from_reader(Cursor::new(&opf_bytes));
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag_name = e.name();
                let tag_name = tag_name.as_ref();

                if tag_name == b"meta" && !seen_cover_id {
                    let mut name = String::new();
                    let mut content = String::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value);
                        if key == "name" && val == "cover" {
                            name = val.to_string();
                        }
                        if key == "content" {
                            content = val.to_string();
                        }
                    }
                    if name == "cover" && !content.is_empty() {
                        cover_id = content;
                        seen_cover_id = true;
                    }
                }

                if !cover_id.is_empty()
                    && (tag_name == b"item" || tag_name == b"itemref")
                {
                    let mut id = String::new();
                    let mut href = String::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value);
                        if key == "id" {
                            id = val.to_string();
                        }
                        if key == "href" {
                            href = val.to_string();
                        }
                    }
                    if id == cover_id && !href.is_empty() {
                        cover_href = href;
                        break;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if cover_href.is_empty() {
        return None;
    }

    let cover_path = if opf_dir.is_empty() {
        cover_href.clone()
    } else {
        format!("{}/{}", opf_dir, cover_href)
    };

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
                        eprintln!("[upload] received file: {filename} ({len} bytes, type={content_type})");
                    }
                    Err(e) => {
                        eprintln!("[upload] error reading file bytes: {e}");
                        file_bytes = Some(Vec::new());
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
    } else if filename.ends_with(".pdf") {
        "pdf"
    } else if filename.ends_with(".mobi") {
        "mobi"
    } else {
        "epub"
    };

    if title.is_empty() {
        title = filename
            .strip_suffix(".epub")
            .or_else(|| filename.strip_suffix(".pdf"))
            .or_else(|| filename.strip_suffix(".mobi"))
            .unwrap_or(&filename)
            .to_string();
    }

    let file_path = format!("books/{}/{}", id, filename);
    let file_size = bytes.len() as i64;

    let cover_result = if format == "epub" {
        extract_epub_cover(&bytes)
    } else {
        None
    };

    eprintln!("[upload] uploading file to S3: {file_path} ({file_size} bytes)");
    storage
        .put(&file_path, bytes, &content_type)
        .await
        .map_err(|e| {
            eprintln!("[upload] ERROR: S3 upload failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "S3 upload failed"})))
        })?;
    eprintln!("[upload] file uploaded successfully");

    let cover_path = if let Some((cover_data, cover_mime)) = cover_result {
        let cover_key = format!("books/{}/cover.jpg", id);
        eprintln!("[upload] uploading cover to S3: {cover_key}");
        if storage.put(&cover_key, cover_data, &cover_mime).await.is_ok() {
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

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id })),
    ))
}

pub async fn list_books(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Book>>, StatusCode> {
    let db = state.db.lock().await;

    let mut stmt = db
        .prepare(
            "SELECT id, title, author, description, publisher, isbn, language, cover_path, \
             file_path, format, file_size, page_count, current_page, created_at, updated_at \
             FROM books ORDER BY created_at DESC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let books = stmt
        .query_map([], |row| {
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
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(books))
}

pub async fn get_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Book>, StatusCode> {
    let db = state.db.lock().await;

    let book = db
        .query_row(
            "SELECT id, title, author, description, publisher, isbn, language, cover_path, \
             file_path, format, file_size, page_count, current_page, created_at, updated_at \
             FROM books WHERE id = ?1",
            params![id],
            |row| {
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
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                })
            },
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(book))
}

pub async fn serve_book_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let storage = storage(&state).map_err(|e| e.0)?;

    let db = state.db.lock().await;
    let (file_path, format): (String, String) = db
        .query_row(
            "SELECT file_path, format FROM books WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;
    drop(db);

    let bytes = storage
        .get(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mime = match format.as_str() {
        "pdf" => "application/pdf",
        _ => "application/epub+zip",
    };

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
