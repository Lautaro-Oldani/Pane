// commands.rs — Comandos Tauri que el frontend puede invocar.
// Cada función con #[tauri::command] se puede llamar desde React con:
//   invoke("nombre_del_comando", { param1: valor1, param2: valor2 })
//
// State<DbPath> es inyección de dependencias de Tauri:
// automáticamente recibe la ruta de la DB que registramos con app.manage().

use arboard::Clipboard;
use tauri::State;

use crate::clipboard::set_skip_next;
use crate::db::{self, Clip, Collection, DbPath};

/// Oculta la ventana y actualiza el flag de visibilidad.
/// El frontend llama esto en vez de window.hide() directamente
/// para mantener sincronizado el flag WINDOW_VISIBLE.
#[tauri::command]
pub fn hide_app_window(app: tauri::AppHandle) {
    crate::hide_window(&app);
}

/// Obtiene clips paginados.
/// Frontend: invoke("get_clips", { limit: 50, offset: 0 })
#[tauri::command]
pub fn get_clips(db_path: State<DbPath>, limit: i64, offset: i64) -> Result<Vec<Clip>, String> {
    db::get_clips_paginated(&db_path.0, limit, offset)
}

/// Elimina un clip por ID.
/// Frontend: invoke("delete_clip", { id: 42 })
#[tauri::command]
pub fn delete_clip(db_path: State<DbPath>, id: i64) -> Result<(), String> {
    db::delete_clip(&db_path.0, id)
}

/// Alterna el pin de un clip. Retorna el nuevo estado (true/false).
/// Frontend: invoke("toggle_pin", { id: 42 })
#[tauri::command]
pub fn toggle_pin(db_path: State<DbPath>, id: i64) -> Result<bool, String> {
    db::toggle_pin(&db_path.0, id)
}

/// Alterna el favorito de un clip. Retorna el nuevo estado.
/// Frontend: invoke("toggle_favorite", { id: 42 })
#[tauri::command]
pub fn toggle_favorite(db_path: State<DbPath>, id: i64) -> Result<bool, String> {
    db::toggle_favorite(&db_path.0, id)
}

/// Borra todo el historial excepto clips pinned y favoritos.
/// Retorna la cantidad de clips eliminados.
/// Frontend: invoke("clear_history")
#[tauri::command]
pub fn clear_history(db_path: State<DbPath>) -> Result<u64, String> {
    db::clear_history(&db_path.0)
}

/// Copia el contenido de un clip al clipboard del sistema.
/// Activa SKIP_NEXT para que el monitor no lo guarde como nuevo clip.
/// Frontend: invoke("copy_to_clipboard", { id: 42 })
#[tauri::command]
pub fn copy_to_clipboard(db_path: State<DbPath>, id: i64) -> Result<(), String> {
    // Leer el clip de la DB
    let clip = db::get_clip_by_id(&db_path.0, id)?;

    // Crear una nueva instancia del clipboard del sistema
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;

    // Activar flag ANTES de escribir, para que el monitor lo ignore
    set_skip_next();

    // Escribir al clipboard según el tipo de contenido
    match clip.content_type.as_str() {
        "image" => {
            // TODO: soporte de imágenes en fase posterior
            Err("Image copy not yet supported".to_string())
        }
        _ => {
            // Para texto, URL, código, color, HTML — todos se copian como texto
            clipboard
                .set_text(&clip.content)
                .map_err(|e| format!("Set text error: {e}"))?;
            Ok(())
        }
    }
}

// ── Colecciones ──────────────────────────────────────────────────────

#[tauri::command]
pub fn get_collections(db_path: State<DbPath>) -> Result<Vec<Collection>, String> {
    db::get_collections(&db_path.0)
}

#[tauri::command]
pub fn create_collection(db_path: State<DbPath>, name: String, icon: Option<String>) -> Result<Collection, String> {
    db::create_collection(&db_path.0, &name, icon.as_deref())
}

#[tauri::command]
pub fn delete_collection(db_path: State<DbPath>, id: i64) -> Result<(), String> {
    db::delete_collection(&db_path.0, id)
}

#[tauri::command]
pub fn rename_collection(db_path: State<DbPath>, id: i64, name: String) -> Result<(), String> {
    db::rename_collection(&db_path.0, id, &name)
}

#[tauri::command]
pub fn set_clip_collection(db_path: State<DbPath>, clip_id: i64, collection_id: Option<i64>) -> Result<(), String> {
    db::set_clip_collection(&db_path.0, clip_id, collection_id)
}
