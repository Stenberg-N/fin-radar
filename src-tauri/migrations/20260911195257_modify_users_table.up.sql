CREATE TABLE users_new (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  last_password_change TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO users_new (id, name, password, created_at, last_password_change)
SELECT id, name, password, datetime('now'), datetime('now')
FROM users;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;