PRAGMA foreign_keys=off;

CREATE TABLE users_new (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  requires_password_reset BOOLEAN NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  last_password_change TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO users_new (id, name, password, requires_password_reset, created_at, last_password_change)
SELECT id, name, password, requires_password_reset, datetime('now'), datetime('now')
FROM users;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;

PRAGMA foreign_keys=on;