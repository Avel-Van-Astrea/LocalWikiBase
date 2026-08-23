use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Row};
use chrono::Utc;
use comrak::{markdown_to_html, ComrakOptions};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub html: String,
    pub tags: Vec<String>,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    pub pinned: bool,
    pub lang: String,
}

#[derive(Debug, Deserialize)]
pub struct NewPage {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub author: String,
    pub lang: String,
}

#[derive(Debug, Deserialize)]
pub struct TogglePinned {
    pub pinned: bool,
}

pub struct AppState {
    pub db: SqlitePool,
}

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    std::fs::create_dir_all("./data")?;
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:./data/knowledge.db?mode=rwc")
        .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL,
            html TEXT NOT NULL,
            tags TEXT NOT NULL,
            author TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            pinned BOOLEAN NOT NULL DEFAULT 0,
            lang TEXT NOT NULL DEFAULT 'ru'
        );
        
        CREATE INDEX IF NOT EXISTS idx_pages_title ON pages(title);
        CREATE INDEX IF NOT EXISTS idx_pages_updated ON pages(updated_at);
        "#
    )
    .execute(&pool)
    .await?;
    
    // Проверяем колонку lang
    let columns: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM pragma_table_info('pages') WHERE name = 'lang'"
    )
    .fetch_all(&pool)
    .await?;
    
    if columns.is_empty() {
        sqlx::query("ALTER TABLE pages ADD COLUMN lang TEXT NOT NULL DEFAULT 'ru'")
            .execute(&pool)
            .await?;
    }
    
    Ok(pool)
}

fn detect_language(text: &str) -> String {
    let text_lower = text.to_lowercase();
    
    if text_lower.chars().any(|c| c >= 'а' && c <= 'я' || c == 'ё') {
        return "ru".to_string();
    }
    
    if text_lower.chars().any(|c| c >= 'a' && c <= 'z') {
        return "en".to_string();
    }
    
    "ru".to_string()
}

pub async fn list_pages(state: web::Data<Arc<AppState>>) -> HttpResponse {
    match sqlx::query(
        r#"
        SELECT 
            id, 
            title, 
            content, 
            html, 
            tags, 
            author, 
            created_at, 
            updated_at, 
            pinned,
            lang
        FROM pages 
        ORDER BY pinned DESC, updated_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await 
    {
        Ok(rows) => {
            let mut result = Vec::new();
            for row in rows {
                let tags_str: String = row.get("tags");
                let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
                result.push(Page {
                    id: row.get("id"),
                    title: row.get("title"),
                    content: row.get("content"),
                    html: row.get("html"),
                    tags,
                    author: row.get("author"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    pinned: row.get::<i64, _>("pinned") != 0,
                    lang: row.get("lang"),
                });
            }
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            eprintln!("Ошибка БД: {}", e);
            HttpResponse::InternalServerError().body("Ошибка загрузки страниц")
        }
    }
}

pub async fn get_page(
    state: web::Data<Arc<AppState>>, 
    id: web::Path<i64>
) -> HttpResponse {
    let page_id = id.into_inner();
    
    match sqlx::query(
        r#"
        SELECT 
            id, 
            title, 
            content, 
            html, 
            tags, 
            author, 
            created_at, 
            updated_at, 
            pinned,
            lang
        FROM pages 
        WHERE id = $1
        "#
    )
    .bind(page_id)
    .fetch_optional(&state.db)
    .await 
    {
        Ok(Some(row)) => {
            let tags_str: String = row.get("tags");
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            HttpResponse::Ok().json(Page {
                id: row.get("id"),
                title: row.get("title"),
                content: row.get("content"),
                html: row.get("html"),
                tags,
                author: row.get("author"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                pinned: row.get::<i64, _>("pinned") != 0,
                lang: row.get("lang"),
            })
        }
        Ok(None) => HttpResponse::NotFound().body("Страница не найдена"),
        Err(e) => {
            eprintln!("Ошибка БД: {}", e);
            HttpResponse::InternalServerError().body("Ошибка загрузки страницы")
        }
    }
}

pub async fn create_page(
    state: web::Data<Arc<AppState>>,
    new_page: web::Json<NewPage>,
) -> HttpResponse {
    let html = markdown_to_html(&new_page.content, &ComrakOptions::default());
    let tags_json = serde_json::to_string(&new_page.tags).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();
    
    let lang = if new_page.lang.is_empty() {
        detect_language(&new_page.title)
    } else {
        new_page.lang.clone()
    };
    
    match sqlx::query(
        "INSERT INTO pages (title, content, html, tags, author, created_at, updated_at, pinned, lang) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
    )
    .bind(&new_page.title)
    .bind(&new_page.content)
    .bind(&html)
    .bind(&tags_json)
    .bind(&new_page.author)
    .bind(&now)
    .bind(&now)
    .bind(false)
    .bind(&lang)
    .fetch_one(&state.db)
    .await 
    {
        Ok(row) => {
            let id: i64 = row.get("id");
            println!("Создана страница: {} (ID: {}, Язык: {})", new_page.title, id, lang);
            HttpResponse::Created().json(serde_json::json!({ "id": id }))
        }
        Err(e) => {
            eprintln!("Ошибка создания: {}", e);
            if e.to_string().contains("UNIQUE constraint failed") {
                HttpResponse::Conflict().body("Страница с таким названием уже существует")
            } else {
                HttpResponse::InternalServerError().body("Ошибка создания страницы")
            }
        }
    }
}

pub async fn update_page(
    state: web::Data<Arc<AppState>>,
    id: web::Path<i64>,
    page: web::Json<NewPage>,
) -> HttpResponse {
    let html = markdown_to_html(&page.content, &ComrakOptions::default());
    let tags_json = serde_json::to_string(&page.tags).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();
    let page_id = id.into_inner();
    
    let lang = if page.lang.is_empty() {
        detect_language(&page.title)
    } else {
        page.lang.clone()
    };
    
    match sqlx::query(
        "UPDATE pages SET title = $1, content = $2, html = $3, tags = $4, updated_at = $5, lang = $6
         WHERE id = $7"
    )
    .bind(&page.title)
    .bind(&page.content)
    .bind(&html)
    .bind(&tags_json)
    .bind(&now)
    .bind(&lang)
    .bind(page_id)
    .execute(&state.db)
    .await 
    {
        Ok(result) if result.rows_affected() > 0 => {
            println!("Обновлена страница ID: {}", page_id);
            HttpResponse::Ok().finish()
        }
        Ok(_) => HttpResponse::NotFound().body("Страница не найдена"),
        Err(e) => {
            eprintln!("Ошибка обновления: {}", e);
            HttpResponse::InternalServerError().body("Ошибка обновления страницы")
        }
    }
}

pub async fn toggle_pinned(
    state: web::Data<Arc<AppState>>,
    id: web::Path<i64>,
    data: web::Json<TogglePinned>,
) -> HttpResponse {
    let page_id = id.into_inner();
    
    match sqlx::query(
        "UPDATE pages SET pinned = $1 WHERE id = $2"
    )
    .bind(data.pinned)
    .bind(page_id)
    .execute(&state.db)
    .await 
    {
        Ok(result) if result.rows_affected() > 0 => {
            println!("Закрепление страницы ID: {} -> {}", page_id, data.pinned);
            HttpResponse::Ok().finish()
        }
        Ok(_) => HttpResponse::NotFound().body("Страница не найдена"),
        Err(e) => {
            eprintln!("Ошибка обновления: {}", e);
            HttpResponse::InternalServerError().body("Ошибка обновления статуса")
        }
    }
}