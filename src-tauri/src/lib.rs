// lib.rs — Punto de entrada de la aplicación Tauri.
// Declara los módulos y configura todos los plugins, estado y commands.

mod clipboard;
mod commands;
mod db;

use db::DbPath;
use tauri::Manager; // Trait para app.path(), app.manage(), app.get_webview_window()
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_sql::{Migration, MigrationKind};

fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create clips, collections, and settings tables",
        sql: include_str!("../migrations/001.sql"),
        kind: MigrationKind::Up,
    }]
}

/// Muestra la ventana principal y le da foco.
/// Se usa desde el tray icon y el global hotkey.
fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Oculta la ventana principal.
fn hide_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
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
        // ── Setup ────────────────────────────────────────────
        .setup(|app| {
            // ── Base de datos ──
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("pane.db");
            db::run_migrations(&db_path)?;
            app.manage(DbPath(db_path.clone()));

            // ── Clipboard monitor ──
            clipboard::start_clipboard_monitor(app.handle().clone(), db_path);

            // ── System Tray ──
            // Crear items del menú contextual (click derecho en el ícono del tray)
            let show_item = MenuItemBuilder::with_id("show", "Show Pane")
                .build(app)?;
            let hide_item = MenuItemBuilder::with_id("hide", "Hide Pane")
                .build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit")
                .build(app)?;

            // Construir el menú con separador entre Hide y Quit
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&hide_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Crear el ícono del tray con el ícono de la app.
            // El ícono se carga desde el directorio icons/ del proyecto.
            // include_image! carga y decodifica el PNG en tiempo de compilación.
            let _tray = TrayIconBuilder::new()
                .icon(tauri::include_image!("icons/32x32.png"))
                .menu(&tray_menu)
                .tooltip("Pane — Clipboard Manager")
                // Manejar clicks en los items del menú
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => show_window(app),
                        "hide" => hide_window(app),
                        "quit" => {
                            app.exit(0); // Cierra la app completamente
                        }
                        _ => {}
                    }
                })
                // Click izquierdo en el ícono del tray: siempre mostrar y dar foco
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        show_window(app);
                    }
                })
                .build(app)?;

            // ── Global Hotkey: Ctrl+Shift+V ──
            // Registrar un atajo global que funciona incluso cuando la app no tiene foco.
            // Ctrl+Shift+V toggle la ventana: si está oculta la muestra, si está visible la oculta.
            let shortcut: Shortcut = "ctrl+shift+v".parse().expect("Invalid shortcut");
            let app_handle = app.handle().clone();

            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                // Solo reaccionar al evento "Pressed", ignorar "Released".
                // Sin esto, la ventana se muestra al presionar y se oculta al soltar.
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.center();
                            let _ = window.set_focus();
                        }
                    }
                }
            })?;

            Ok(())
        })
        // ── Interceptar el cierre de ventana: ocultar en vez de cerrar ──
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevenir que la ventana se cierre
                api.prevent_close();
                // En su lugar, ocultarla
                let _ = window.hide();
            }
        })
        // ── Commands ─────────────────────────────────────────
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
