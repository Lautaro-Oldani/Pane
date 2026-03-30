// lib.rs — Punto de entrada de la aplicación Tauri.
// Declara los módulos y configura todos los plugins, estado y commands.

// "mod" declara un módulo (como un import de un archivo).
// Rust busca el archivo clipboard.rs, commands.rs, db.rs en el mismo directorio.
mod clipboard;
mod commands;
mod db;

use db::DbPath;
use tauri::Manager; // Trait necesario para app.path() y app.manage()
use tauri_plugin_sql::{Migration, MigrationKind};

/// Define las migraciones SQL para el plugin tauri-plugin-sql.
/// Estas se ejecutan cuando el frontend llama Database.load().
fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create clips, collections, and settings tables",
        sql: include_str!("../migrations/001.sql"),
        kind: MigrationKind::Up,
    }]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ── Plugins ──────────────────────────────────────────
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:pane.db", get_migrations())
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        // ── Setup: se ejecuta una vez al arrancar la app ────
        .setup(|app| {
            // Obtener el directorio de datos de la app.
            // En Windows: C:\Users\{user}\AppData\Roaming\com.lauta.pane\
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            // Crear el directorio si no existe (por si es la primera vez)
            std::fs::create_dir_all(&app_data_dir).ok();

            let db_path = app_data_dir.join("pane.db");

            // Ejecutar migraciones desde Rust ANTES de iniciar el monitor.
            // Esto garantiza que las tablas existan cuando el monitor
            // intente guardar un clip (sin esperar al frontend).
            db::run_migrations(&db_path)?;

            // Registrar la ruta de la DB como estado compartido de Tauri.
            // Los commands la reciben con State<DbPath>.
            app.manage(DbPath(db_path.clone()));

            // Iniciar el monitor de clipboard en un thread separado.
            // A partir de aquí, todo lo que se copie en Windows se guarda en la DB.
            clipboard::start_clipboard_monitor(app.handle().clone(), db_path);

            Ok(())
        })
        // ── Commands: funciones que el frontend puede llamar ─
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::delete_clip,
            commands::toggle_pin,
            commands::toggle_favorite,
            commands::clear_history,
            commands::copy_to_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
