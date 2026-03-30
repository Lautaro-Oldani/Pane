// db.rs — Operaciones directas con SQLite usando rusqlite.
// Este módulo lo usan tanto el clipboard monitor (para guardar clips)
// como los commands (para leer/borrar/actualizar clips).
// Cada función abre su propia conexión y la cierra al terminar,
// lo cual es seguro con SQLite (maneja locks por archivo).

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Ruta a la base de datos SQLite.
/// Se comparte entre todos los commands via Tauri State (app.manage).
pub struct DbPath(pub PathBuf);

/// Estructura que representa un clip del clipboard.
/// derive(Serialize) permite enviarla al frontend como JSON.
/// derive(Clone) permite copiarla cuando emitimos eventos.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Clip {
    pub id: i64,
    pub content: String,
    pub content_type: String,
    pub preview: Option<String>,
    pub image_base64: Option<String>,
    pub source_app: Option<String>,
    pub hash: String,
    pub is_pinned: bool,
    pub is_favorite: bool,
    pub collection_id: Option<i64>,
    pub created_at: String,
}

/// Columnas que seleccionamos siempre (evita repetir el SELECT largo)
const SELECT_COLS: &str = "id, content, content_type, preview, image_base64, \
                           source_app, hash, is_pinned, is_favorite, collection_id, created_at";

/// Abre una conexión a SQLite con busy_timeout de 5 segundos.
/// El timeout evita errores SQLITE_BUSY si otra conexión tiene un lock.
fn open(path: &PathBuf) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| format!("DB open error: {e}"))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| format!("DB timeout error: {e}"))?;
    Ok(conn)
}

/// Convierte una fila de SQLite a nuestra struct Clip.
/// row.get(N) extrae la columna N (en el orden del SELECT).
/// Para is_pinned/is_favorite: SQLite guarda 0/1, lo convertimos a bool.
fn row_to_clip(row: &rusqlite::Row) -> Result<Clip, rusqlite::Error> {
    Ok(Clip {
        id: row.get(0)?,
        content: row.get(1)?,
        content_type: row.get(2)?,
        preview: row.get(3)?,
        image_base64: row.get(4)?,
        source_app: row.get(5)?,
        hash: row.get(6)?,
        is_pinned: row.get::<_, i32>(7)? != 0,  // 0 -> false, 1 -> true
        is_favorite: row.get::<_, i32>(8)? != 0,
        collection_id: row.get(9)?,
        created_at: row.get(10)?,
    })
}

/// Ejecuta las migraciones SQL directamente desde Rust.
/// Se llama en setup() ANTES de iniciar el clipboard monitor,
/// para garantizar que las tablas existan.
/// Las sentencias usan IF NOT EXISTS, así que es seguro llamar múltiples veces.
pub fn run_migrations(path: &PathBuf) -> Result<(), String> {
    let conn = open(path)?;
    // execute_batch ejecuta múltiples sentencias SQL separadas por ;
    conn.execute_batch(include_str!("../migrations/001.sql"))
        .map_err(|e| format!("Migration error: {e}"))?;
    Ok(())
}

/// Inserta un nuevo clip en la base de datos.
/// Si ya existe un clip con el mismo hash, lo mueve al tope (actualiza created_at)
/// en vez de crear un duplicado. Retorna el Clip completo.
pub fn insert_clip(
    path: &PathBuf,
    content: &str,
    content_type: &str,
    preview: &str,
    hash: &str,
    image_base64: Option<&str>,
) -> Result<Clip, String> {
    let conn = open(path)?;

    // Buscar si ya existe un clip con este hash (duplicado de una sesión anterior)
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM clips WHERE hash = ?1",
            params![hash],
            |row| row.get(0),
        )
        .ok();

    let id = if let Some(eid) = existing_id {
        // Ya existe: actualizar created_at para moverlo al tope de la lista
        conn.execute(
            "UPDATE clips SET created_at = datetime('now', 'localtime') WHERE id = ?1",
            params![eid],
        )
        .map_err(|e| format!("Update error: {e}"))?;
        eid
    } else {
        // No existe: insertar nuevo
        conn.execute(
            "INSERT INTO clips (content, content_type, preview, hash, image_base64) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![content, content_type, preview, hash, image_base64],
        )
        .map_err(|e| format!("Insert error: {e}"))?;
        conn.last_insert_rowid()
    };

    conn.query_row(
        &format!("SELECT {SELECT_COLS} FROM clips WHERE id = ?1"),
        params![id],
        row_to_clip,
    )
    .map_err(|e| format!("Query error after insert: {e}"))
}

/// Obtiene el hash del último clip guardado.
/// Se usa al iniciar el monitor para no re-guardar lo que ya estaba en el clipboard.
pub fn get_last_hash(path: &PathBuf) -> Result<Option<String>, String> {
    let conn = open(path)?;
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT hash FROM clips ORDER BY created_at DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(hash) => Ok(Some(hash)),
        // QueryReturnedNoRows = tabla vacía, no es un error
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Query error: {e}")),
    }
}

/// Obtiene clips paginados, ordenados por fecha descendente (más reciente primero).
/// limit = cuántos clips traer, offset = desde qué posición (para paginación).
pub fn get_clips_paginated(path: &PathBuf, limit: i64, offset: i64) -> Result<Vec<Clip>, String> {
    let conn = open(path)?;

    // prepare() compila la query SQL una vez (más eficiente si se reutiliza)
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLS} FROM clips ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        ))
        .map_err(|e| format!("Prepare error: {e}"))?;

    // query_map ejecuta la query y aplica row_to_clip a cada fila.
    // Guardamos el resultado en una variable local para que los borrows
    // de conn y stmt se resuelvan correctamente (Rust lifetime rules).
    let clips: Vec<Clip> = stmt
        .query_map(params![limit, offset], row_to_clip)
        .map_err(|e| format!("Query error: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))?;

    Ok(clips)
}

/// Elimina un clip por ID.
pub fn delete_clip(path: &PathBuf, id: i64) -> Result<(), String> {
    let conn = open(path)?;
    conn.execute("DELETE FROM clips WHERE id = ?1", params![id])
        .map_err(|e| format!("Delete error: {e}"))?;
    Ok(())
}

/// Alterna el estado "pinned" de un clip (0 -> 1, 1 -> 0).
/// Retorna el nuevo valor de is_pinned como bool.
pub fn toggle_pin(path: &PathBuf, id: i64) -> Result<bool, String> {
    let conn = open(path)?;
    // CASE WHEN hace el toggle en una sola query
    conn.execute(
        "UPDATE clips SET is_pinned = CASE WHEN is_pinned = 0 THEN 1 ELSE 0 END WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("Update error: {e}"))?;

    // Leer el nuevo valor
    let new_val: i32 = conn
        .query_row(
            "SELECT is_pinned FROM clips WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| format!("Query error: {e}"))?;
    Ok(new_val != 0)
}

/// Alterna el estado "favorite" de un clip.
/// Retorna el nuevo valor de is_favorite como bool.
pub fn toggle_favorite(path: &PathBuf, id: i64) -> Result<bool, String> {
    let conn = open(path)?;
    conn.execute(
        "UPDATE clips SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("Update error: {e}"))?;

    let new_val: i32 = conn
        .query_row(
            "SELECT is_favorite FROM clips WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| format!("Query error: {e}"))?;
    Ok(new_val != 0)
}

/// Borra todo el historial EXCEPTO clips pinned y favoritos.
/// Retorna la cantidad de clips eliminados.
pub fn clear_history(path: &PathBuf) -> Result<u64, String> {
    let conn = open(path)?;
    let deleted = conn
        .execute(
            "DELETE FROM clips WHERE is_pinned = 0 AND is_favorite = 0 AND collection_id IS NULL",
            [],
        )
        .map_err(|e| format!("Delete error: {e}"))?;
    // "as u64" convierte usize a u64 (Rust es estricto con tipos numéricos)
    Ok(deleted as u64)
}

// ── Colecciones ──────────────────────────────────────────────────────

/// Estructura que representa una colección de clips.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub created_at: String,
}

/// Crea una nueva colección. Retorna la colección creada.
pub fn create_collection(path: &PathBuf, name: &str, icon: Option<&str>) -> Result<Collection, String> {
    let conn = open(path)?;
    conn.execute(
        "INSERT INTO collections (name, icon) VALUES (?1, ?2)",
        params![name, icon],
    )
    .map_err(|e| format!("Insert error: {e}"))?;

    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, name, icon, created_at FROM collections WHERE id = ?1",
        params![id],
        |row| Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            created_at: row.get(3)?,
        }),
    )
    .map_err(|e| format!("Query error: {e}"))
}

/// Obtiene todas las colecciones.
pub fn get_collections(path: &PathBuf) -> Result<Vec<Collection>, String> {
    let conn = open(path)?;
    let mut stmt = conn
        .prepare("SELECT id, name, icon, created_at FROM collections ORDER BY name")
        .map_err(|e| format!("Prepare error: {e}"))?;

    let collections: Vec<Collection> = stmt
        .query_map([], |row| Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            created_at: row.get(3)?,
        }))
        .map_err(|e| format!("Query error: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))?;

    Ok(collections)
}

/// Elimina una colección. Los clips que pertenecían quedan con collection_id = NULL.
pub fn delete_collection(path: &PathBuf, id: i64) -> Result<(), String> {
    let conn = open(path)?;
    conn.execute("DELETE FROM collections WHERE id = ?1", params![id])
        .map_err(|e| format!("Delete error: {e}"))?;
    Ok(())
}

/// Renombra una colección.
pub fn rename_collection(path: &PathBuf, id: i64, name: &str) -> Result<(), String> {
    let conn = open(path)?;
    conn.execute(
        "UPDATE collections SET name = ?1 WHERE id = ?2",
        params![name, id],
    )
    .map_err(|e| format!("Update error: {e}"))?;
    Ok(())
}

/// Asigna un clip a una colección (o lo saca si collection_id es None).
pub fn set_clip_collection(path: &PathBuf, clip_id: i64, collection_id: Option<i64>) -> Result<(), String> {
    let conn = open(path)?;
    conn.execute(
        "UPDATE clips SET collection_id = ?1 WHERE id = ?2",
        params![collection_id, clip_id],
    )
    .map_err(|e| format!("Update error: {e}"))?;
    Ok(())
}

/// Cuenta clips por colección.
pub fn count_clips_in_collection(path: &PathBuf, collection_id: i64) -> Result<i64, String> {
    let conn = open(path)?;
    conn.query_row(
        "SELECT COUNT(*) FROM clips WHERE collection_id = ?1",
        params![collection_id],
        |r| r.get(0),
    )
    .map_err(|e| format!("Count error: {e}"))
}

// ── Settings ─────────────────────────────────────────────────────────

/// Obtiene todos los settings como pares key-value.
pub fn get_all_settings(path: &PathBuf) -> Result<Vec<(String, String)>, String> {
    let conn = open(path)?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| format!("Prepare error: {e}"))?;
    let settings: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| format!("Query error: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))?;
    Ok(settings)
}

/// Obtiene un setting por key.
pub fn get_setting(path: &PathBuf, key: &str) -> Result<Option<String>, String> {
    let conn = open(path)?;
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    );
    match result {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Query error: {e}")),
    }
}

/// Guarda un setting (INSERT OR REPLACE).
pub fn set_setting(path: &PathBuf, key: &str, value: &str) -> Result<(), String> {
    let conn = open(path)?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| format!("Insert error: {e}"))?;
    Ok(())
}

// ── Limpieza automática ──────────────────────────────────────────────

/// Borra clips que exceden el límite de historial.
/// Respeta pinned y favorites (no los borra).
/// Borra los más viejos primero.
pub fn enforce_history_limit(path: &PathBuf, limit: i64) -> Result<u64, String> {
    let conn = open(path)?;
    // Contar clips no-protegidos (no pinned, no favorito, no en colección)
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM clips WHERE is_pinned = 0 AND is_favorite = 0 AND collection_id IS NULL",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("Count error: {e}"))?;

    if count <= limit {
        return Ok(0);
    }

    let to_delete = count - limit;
    let deleted = conn
        .execute(
            "DELETE FROM clips WHERE id IN (
                SELECT id FROM clips
                WHERE is_pinned = 0 AND is_favorite = 0 AND collection_id IS NULL
                ORDER BY created_at ASC
                LIMIT ?1
            )",
            params![to_delete],
        )
        .map_err(|e| format!("Delete error: {e}"))?;

    Ok(deleted as u64)
}

/// Borra clips más viejos que X días.
/// Respeta pinned y favorites.
pub fn clear_old_clips(path: &PathBuf, days: i64) -> Result<u64, String> {
    if days <= 0 {
        return Ok(0);
    }
    let conn = open(path)?;
    let deleted = conn
        .execute(
            "DELETE FROM clips WHERE is_pinned = 0 AND is_favorite = 0 AND collection_id IS NULL
             AND created_at < datetime('now', 'localtime', ?1)",
            params![format!("-{days} days")],
        )
        .map_err(|e| format!("Delete error: {e}"))?;

    Ok(deleted as u64)
}

/// Ejecuta ambas limpiezas usando los valores de la tabla settings.
pub fn run_cleanup(path: &PathBuf) -> Result<(), String> {
    // Leer settings
    let limit_str = get_setting(path, "history_limit")?.unwrap_or_else(|| "500".to_string());
    let days_str = get_setting(path, "auto_clear_days")?.unwrap_or_else(|| "30".to_string());

    let limit: i64 = limit_str.parse().unwrap_or(500);
    let days: i64 = days_str.parse().unwrap_or(30);

    // Primero borrar por antigüedad, después por límite
    let old = clear_old_clips(path, days)?;
    let over = enforce_history_limit(path, limit)?;

    if old > 0 || over > 0 {
        eprintln!("Cleanup: removed {old} old + {over} over limit");
    }

    Ok(())
}

// ── Clips (continuación) ────────────────────────────────────────────

/// Obtiene un clip por ID. Se usa en copy_to_clipboard para leer el contenido.
pub fn get_clip_by_id(path: &PathBuf, id: i64) -> Result<Clip, String> {
    let conn = open(path)?;
    conn.query_row(
        &format!("SELECT {SELECT_COLS} FROM clips WHERE id = ?1"),
        params![id],
        row_to_clip,
    )
    .map_err(|e| format!("Clip not found: {e}"))
}
