//! Main entry point for the Campsite Tracker backend server.
//! This crate provides REST API endpoints and serves the frontend application.

use actix_files::Files;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use campsite_tracker::*;
use std::path::Path;

async fn api_hello() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Hello from Rust backend on AWS!",
        "status": "running"
    })))
}

fn get_frontend_path() -> &'static str {
    // Check multiple possible locations for frontend files
    if Path::new("./frontend-build").exists() {
        log::info!("✅ Using Docker frontend path: ./frontend-build");
        "./frontend-build"
    } else if Path::new("../frontend/build").exists() {
        log::info!("✅ Using local frontend path: ../frontend/build");
        "../frontend/build"
    } else {
        log::info!("❌ Frontend files not found in either location");
        "./frontend-build" // fallback
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("🚀 Starting campsite tracker server...");

    // Create database connection pool
    let pool = match create_connection_pool().await {
        Ok(pool) => {
            log::info!("🗃️ Database pool created successfully");

            if let Err(e) = test_connection(&pool).await {
                log::error!("❌ Database connection test failed: {}", e);
            }
            pool
        }
        Err(e) => {
            log::error!("❌ Failed to create database pool: {}", e);
            log::error!("💡 Make sure PostgreSQL is running: brew services start postgresql@16");
            std::process::exit(1);
        }
    };

    let frontend_path = get_frontend_path();
    log::info!("📁 Frontend files location: {}", frontend_path);
    log::info!("🌐 Server will be available at: http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    // Public routes
                    .route("/hello", web::get().to(api_hello))
                    .route("/facilities/search", web::get().to(facilities_search))
                    .service(
                        web::scope("/auth")
                            .route("/health", web::get().to(auth_health))
                            .route("/signup", web::post().to(signup))
                            .route("/login", web::post().to(login))
                            .route("/users", web::get().to(list_users)),
                    )
                    // Protected routes (require authentication)
                    .service(
                        web::scope("/user")
                            .wrap(AuthMiddleware)
                            .route("/profile", web::get().to(get_profile))
                            .route("/profile/update", web::put().to(update_profile)),
                    ),
            )
            .route(
                "/health",
                web::get().to(|| async { HttpResponse::Ok().body("OK") }),
            )
            .service(Files::new("/", frontend_path).index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
