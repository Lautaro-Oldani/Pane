// Importamos el trait MigrationKind para definir migraciones SQL
use tauri_plugin_sql::{Migration, MigrationKind};

// Tauri command de ejemplo (lo vamos a reemplazar en la próxima fase)
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Define las migraciones de la base de datos.
/// Cada migración tiene una versión, descripción, el SQL a ejecutar, y el tipo (Up = crear/modificar).
/// Tauri ejecuta automáticamente las migraciones que no se hayan aplicado todavía.
fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create clips, collections, and settings tables",
            sql: include_str!("../migrations/001.sql"), // Carga el SQL desde el archivo
            kind: MigrationKind::Up,                     // Up = aplicar migración
        },
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Plugin de clipboard (leer/escribir clipboard desde el frontend)
        .plugin(tauri_plugin_clipboard_manager::init())
        // Plugin de autostart (iniciar con Windows)
        .plugin(tauri_plugin_autostart::Builder::new().build())
        // Plugin de global shortcut (hotkeys globales como Ctrl+Shift+V)
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Plugin de SQLite con migraciones
        // "sqlite:pane.db" = crea/abre la base de datos "pane.db" en el directorio de datos de la app
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:pane.db", get_migrations())
                .build(),
        )
        // Plugin para abrir links externos
        .plugin(tauri_plugin_opener::init())
        // Registra los comandos que el frontend puede llamar via invoke()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
