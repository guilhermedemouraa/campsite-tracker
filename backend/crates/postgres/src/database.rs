use sqlx::{PgPool, Row};

/// Creates a connection pool to the PostgreSQL database.
pub async fn create_connection_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/campsite_tracker".to_string());

    PgPool::connect(&database_url).await
}

/// Tests the database connection by executing a simple query.
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT 1 as test").fetch_one(pool).await?;

    let test_value: i32 = row.get("test");
    println!(
        "✅ Database connection successful! Test value: {}",
        test_value
    );

    Ok(())
}
