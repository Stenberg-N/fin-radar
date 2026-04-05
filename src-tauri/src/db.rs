use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Executor};

pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let db = SqlitePoolOptions::new()
    .after_connect(|conn, _| {
        Box::pin(async move {
            conn.execute(sqlx::query(
                "PRAGMA journal_mode = WAL;\
                PRAGMA foreign_keys = ON;\
                PRAGMA auto_vacuum = INCREMENTAL;\
                PRAGMA optimize;",
            ))
            .await?;

            Ok(())
        })
    })
    .connect(db_path)
    .await?;

    let mut conn = db.acquire().await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL
        )"
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY,
            user_id INTEGER,
            category TEXT NOT NULL,
            description TEXT NOT NULL,
            amount REAL NOT NULL,
            _type TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )"
    )
    .execute(&mut *conn)
    .await?;

    Ok(db)
}