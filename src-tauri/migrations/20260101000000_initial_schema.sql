CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_password_change TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_users_name ON users (name);

CREATE TABLE IF NOT EXISTS user_crypto (
    id INTEGER PRIMARY KEY,
    version BLOB NOT NULL,
    user_id INTEGER NOT NULL UNIQUE,
    wrapped_dek BLOB NOT NULL,
    wrapped_dek_recovery BLOB NOT NULL,
    recovery_kek_salt BLOB NOT NULL,
    kek_salt BLOB NOT NULL,
    kdf_params BLOB NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    payload BLOB,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions (user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_user_date ON transactions (user_id, date);

CREATE TABLE IF NOT EXISTS recovery_keys (
    user_id INTEGER PRIMARY KEY,
    key_hash TEXT NOT NULL,
    is_used BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tabs (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    order_id INTEGER NOT NULL,
    title TEXT,
    color TEXT DEFAULT 'transparent',
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_tabs_user_id ON tabs (user_id);
CREATE INDEX IF NOT EXISTS idx_tabs_user_order ON tabs (user_id, order_id);

CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    tab_id INTEGER NOT NULL,
    order_id INTEGER NOT NULL,
    title TEXT,
    content TEXT,
    bg_color TEXT NOT NULL DEFAULT '#1c1c1c',
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (tab_id) REFERENCES tabs(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_notes_user_id ON notes (user_id);
CREATE INDEX IF NOT EXISTS idx_notes_user_tab ON notes (user_id, tab_id);
CREATE INDEX IF NOT EXISTS idx_notes_tab_order ON notes (tab_id, order_id);

CREATE TABLE IF NOT EXISTS timers (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    order_id INTEGER NOT NULL,
    duration INTEGER NOT NULL,
    title TEXT NOT NULL,
    message TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_timers_user_id ON timers (user_id);

CREATE TABLE IF NOT EXISTS calendar_events (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    isodate TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    start_time INTEGER,
    end_time INTEGER,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS calendar_tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS calendar_events_tags (
    event_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (event_id, tag_id),
    FOREIGN KEY (event_id) REFERENCES calendar_events(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES calendar_tags(id) ON DELETE CASCADE
);