-- Tabla principal: cada entrada del clipboard
CREATE TABLE IF NOT EXISTS clips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,                          -- El contenido copiado (texto, URL, código, etc.)
    content_type TEXT NOT NULL DEFAULT 'text',       -- Tipo: text, image, url, code, color, html
    preview TEXT,                                    -- Preview corto para mostrar en la lista
    image_base64 TEXT,                              -- Imagen en base64 (si es tipo image)
    source_app TEXT,                                -- App de origen (ej: "Chrome", "VS Code")
    hash TEXT NOT NULL,                             -- Hash del contenido para detectar duplicados
    is_pinned INTEGER NOT NULL DEFAULT 0,           -- 1 = pinned (no se borra con auto-clean)
    is_favorite INTEGER NOT NULL DEFAULT 0,         -- 1 = favorito
    collection_id INTEGER,                          -- FK a collections (nullable)
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE SET NULL
);

-- Tabla de colecciones / pinboards
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    icon TEXT,                                      -- Emoji o nombre de ícono
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- Tabla de configuración key-value
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Índices para performance
CREATE INDEX IF NOT EXISTS idx_clips_created_at ON clips(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clips_content_type ON clips(content_type);
CREATE INDEX IF NOT EXISTS idx_clips_hash ON clips(hash);
CREATE INDEX IF NOT EXISTS idx_clips_collection ON clips(collection_id);

-- Settings iniciales
INSERT OR IGNORE INTO settings (key, value) VALUES ('history_limit', '500');
INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_clear_days', '30');
INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'dark');
INSERT OR IGNORE INTO settings (key, value) VALUES ('hotkey', 'Ctrl+Shift+V');
INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', 'false');
