CREATE TABLE notes_new (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL,
  tab_id INTEGER NOT NULL,
  order_id INTEGER NOT NULL,
  title TEXT,
  content TEXT,
  bg_color TEXT NOT NULL DEFAULT '#1c1c1c'
);

INSERT INTO notes_new (id, user_id, tab_id, order_id, title, content, bg_color)
SELECT id, user_id, tab_id, order_id, title, content, '#1c1c1c'
FROM notes;

DROP TABLE notes;
ALTER TABLE notes_new RENAME TO notes;