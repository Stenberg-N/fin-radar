use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Executor};

pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let db = SqlitePoolOptions::new()
    .after_connect(|conn, _| {
        Box::pin(async move {
            conn.execute(sqlx::query(
                "PRAGMA journal_mode = WAL;\
                PRAGMA foreign_keys = ON;\
                PRAGMA auto_vacuum = INCREMENTAL;\
                PRAGMA optimize;\
                PRAGMA incremental_vacuum(0);\
                PRAGMA wal_checkpoint(TRUNCATE);",
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
            password TEXT NOT NULL,
            requires_password_reset BOOLEAN NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_users_name ON users (name);"
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            category TEXT NOT NULL,
            date TEXT NOT NULL,
            description TEXT NOT NULL,
            amount REAL NOT NULL,
            _type TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions (user_id);
        CREATE INDEX IF NOT EXISTS idx_transactions_user_date ON transactions (user_id, date);"
    )    
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS recovery_keys (
            user_id INTEGER PRIMARY KEY,
            key_hash TEXT NOT NULL,
            is_used BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );"
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tabs (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            order_id INTEGER NOT NULL,
            title TEXT,
            color TEXT DEFAULT 'transparent',
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_tabs_user_id ON tabs (user_id);
        CREATE INDEX IF NOT EXISTS idx_tabs_user_order ON tabs (user_id, order_id);"
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            tab_id INTEGER NOT NULL,
            order_id INTEGER NOT NULL,
            title TEXT,
            content TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (tab_id) REFERENCES tabs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_notes_user_id ON notes (user_id);
        CREATE INDEX IF NOT EXISTS idx_notes_user_tab ON notes (user_id, tab_id);
        CREATE INDEX IF NOT EXISTS idx_notes_tab_order ON notes (tab_id, order_id);"
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS timers (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            order_id INTEGER NOT NULL,
            duration INTEGER NOT NULL,
            title TEXT NOT NULL,
            message TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_timers_user_id ON timers (user_id);"
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS calendar_events (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            isodate TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            start_time INTEGER,
            end_time INTEGER,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );"
    )
    .execute(&mut *conn)
    .await?;

    Ok(db)
}