use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use rusqlite::params;
use serde::Serialize;
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

fn storage(state: &AppState) -> Result<&Storage, StatusCode> {
    state.storage.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)
}

pub async fn upload_book(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let storage = storage(&state)?;

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = String::new();
    let mut content_type = String::new();
    let mut title = String::new();
    let mut author: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                content_type = field
                    .content_type()
                    .unwrap_or("application/epub+zip")
                    .to_string();
                filename = field.file_name().unwrap_or("untitled.epub").to_string();
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .to_vec(),
                );
            }
            "title" => {
                title = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "author" => {
                author = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or(StatusCode::BAD_REQUEST)?;

    if bytes.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
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

    storage
        .put(&file_path, bytes, &content_type)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let db = state.db.lock().await;
    db.execute(
        "INSERT INTO books (id, title, author, file_path, format, file_size) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, title, author, file_path, format, file_size],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
    let storage = storage(&state)?;

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
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_path.split('/').last().unwrap_or("book")),
        )
        .body(bytes.into())
        .unwrap())
}

pub async fn delete_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let db = state.db.lock().await;

    let file_path: String = db
        .query_row(
            "SELECT file_path FROM books WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    db.execute("DELETE FROM books WHERE id = ?1", params![id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    drop(db);

    if let Ok(storage) = storage(&state) {
        storage.delete(&file_path).await.ok();
    }

    Ok(StatusCode::NO_CONTENT)
}
