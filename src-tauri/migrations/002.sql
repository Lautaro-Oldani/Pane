-- Tabla de shortcuts para text expansion
-- trigger: el texto que activa el shortcut (ej: "/greeting")
-- content: el texto que lo reemplaza
CREATE TABLE IF NOT EXISTS shortcuts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trigger TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
