use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::sync::Arc;
use sqlx::Row;
use crate::wiki::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: i64,
    pub title: String,
    pub tags: Vec<String>,
    pub snippet: String,
}

pub async fn search_pages(
    state: web::Data<Arc<AppState>>,
    query: web::Query<SearchQuery>,
) -> HttpResponse {
    if query.q.trim().is_empty() {
        return HttpResponse::BadRequest().body("Enter search query");
    }
    
    let matcher = SkimMatcherV2::default();
    let query_lower = query.q.to_lowercase();
    
    let rows = match sqlx::query(
        "SELECT id, title, content, html, tags, author, created_at, updated_at, pinned FROM pages"
    )
    .fetch_all(&state.db)
    .await 
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Search error: {}", e);
            return HttpResponse::InternalServerError().body("Search error");
        }
    };
    
    let mut results = Vec::new();
    
    for row in rows {
        let id: i64 = row.get("id");
        let title: String = row.get("title");
        let content: String = row.get("content");
        let tags_str: String = row.get("tags");
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        
        let mut score = 0;
        
        if let Some(s) = matcher.fuzzy_match(&title, &query_lower) {
            score += s * 3;
        }
        
        if let Some(s) = matcher.fuzzy_match(&content, &query_lower) {
            score += s;
        }
        
        for tag in &tags {
            if let Some(s) = matcher.fuzzy_match(tag, &query_lower) {
                score += s * 2;
            }
        }
        
        if score > 20 {
            let snippet = if let Some(pos) = content.to_lowercase().find(&query_lower) {
                let start = pos.saturating_sub(30);
                let end = (pos + query_lower.len() + 30).min(content.len());
                let mut snippet = content[start..end].to_string();
                if start > 0 { snippet = format!("...{}", snippet); }
                if end < content.len() { snippet = format!("{}...", snippet); }
                snippet
            } else {
                content.chars().take(100).collect::<String>() + "..."
            };
            
            results.push(SearchResult {
                id,
                title,
                tags,
                snippet,
            });
        }
    }
    
    results.sort_by(|a, b| b.snippet.len().cmp(&a.snippet.len()));
    results.truncate(20);
    
    HttpResponse::Ok().json(results)
}