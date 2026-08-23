mod wiki;
mod search;
mod i18n;

use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = wiki::init_db().await.unwrap();
    let state = Arc::new(wiki::AppState { db: pool });
    let i18n = Arc::new(i18n::I18n::new());
    
    println!("📚 Корпоративная база знаний");
    println!("🌐 Локальный доступ: http://localhost:8080");
    
    // Получаем IP адрес компьютера в сети
    if let Ok(ip) = get_local_ip() {
        println!("📡 Доступ в сети: http://{}:8080", ip);
    }
    
    println!("Нажмите Ctrl+C для остановки");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(i18n.clone()))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .route("/pages", web::get().to(wiki::list_pages))
                    .route("/page/{id}", web::get().to(wiki::get_page))
                    .route("/page", web::post().to(wiki::create_page))
                    .route("/page/{id}", web::put().to(wiki::update_page))
                    .route("/page/{id}/pinned", web::patch().to(wiki::toggle_pinned))
                    .route("/search", web::get().to(search::search_pages))
                    .route("/i18n", web::get().to(get_translations))
            )
            .route("/", web::get().to(|| async {
                HttpResponse::Ok().body(include_str!("../static/index.html"))
            }))
            .route("/style.css", web::get().to(|| async {
                HttpResponse::Ok()
                    .content_type("text/css")
                    .body(include_str!("../static/style.css"))
            }))
            .route("/script.js", web::get().to(|| async {
                HttpResponse::Ok()
                    .content_type("application/javascript")
                    .body(include_str!("../static/script.js"))
            }))
            .route("/health", web::get().to(|| async {
                HttpResponse::Ok().body("OK")
            }))
    })
    .bind("0.0.0.0:8080")?  // <- Меняем на 0.0.0.0
    .run()
    .await
}

async fn get_translations(
    i18n: web::Data<Arc<i18n::I18n>>,
) -> HttpResponse {
    let mut result = std::collections::HashMap::new();
    
    let keys = vec![
        "app_title", "search_placeholder", "new_article", "edit", "delete",
        "save", "cancel", "title", "content", "tags", "author", "pinned",
        "created", "updated", "no_pages", "loading", "theme",
        "dark_theme", "light_theme", "welcome", "welcome_desc", "recent_articles",
        "all_articles", "by", "search_results", "not_found", "back_to_list",
        "pin", "unpin"
    ];
    
    for key in keys {
        result.insert(key.to_string(), i18n.get(key));
    }
    
    HttpResponse::Ok().json(result)
}

// Функция для получения IP адреса в локальной сети
fn get_local_ip() -> Result<String, String> {
    use std::net::UdpSocket;
    
    // Подключаемся к публичному DNS для получения локального IP
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket.connect("8.8.8.8:80").map_err(|e| e.to_string())?;
    let ip = socket.local_addr().map_err(|e| e.to_string())?.ip().to_string();
    Ok(ip)
}