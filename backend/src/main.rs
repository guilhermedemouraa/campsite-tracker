use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use campsite_tracker::{create_connection_pool, facilities_search, test_connection};
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
        println!("✅ Using Docker frontend path: ./frontend-build");
        "./frontend-build"
    } else if Path::new("../frontend/build").exists() {
        println!("✅ Using local frontend path: ../frontend/build");
        "../frontend/build"
    } else {
        println!("❌ Frontend files not found in either location");
        "./frontend-build" // fallback
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting campsite tracker server...");
    match create_connection_pool().await {
        Ok(pool) => {
            println!("🗃️ Database pool created successfully");

            if let Err(e) = test_connection(&pool).await {
                println!("❌ Database connection test failed: {}", e);
            }
        }
        Err(e) => {
            println!("❌ Failed to create database pool: {}", e);
            println!("💡 Make sure PostgreSQL is running: brew services start postgresql@16");
        }
    }
    let frontend_path = get_frontend_path();
    println!("📁 Frontend files location: {}", frontend_path);
    println!("🌐 Server will be available at: http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .route("/hello", web::get().to(api_hello))
                    .route("/facilities/search", web::get().to(facilities_search)),
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
