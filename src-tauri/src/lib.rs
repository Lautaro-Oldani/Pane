// lib.rs — Punto de entrada de la aplicación Tauri.
// Declara los módulos y configura todos los plugins, estado y commands.

mod categories;
mod clipboard;
mod commands;
mod db;

use db::DbPath;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_sql::{Migration, MigrationKind};

/// Flag global para trackear si la ventana está visible.
/// No confiamos en window.is_visible() porque devuelve valores incorrectos
/// después de prevent_close() + hide().
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create clips, collections, and settings tables",
        sql: include_str!("../migrations/001.sql"),
        kind: MigrationKind::Up,
    }]
}

/// Muestra la ventana principal y le da foco.
/// Maneja todos los casos: oculta, minimizada, o sin foco.
fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize(); // Por si está minimizada
        let _ = window.show();       // Por si está oculta (hide)
        let _ = window.center();
        let _ = window.set_focus();
        WINDOW_VISIBLE.store(true, Ordering::Relaxed);
    }
}

/// Oculta la ventana principal.
fn hide_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        WINDOW_VISIBLE.store(false, Ordering::Relaxed);
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
            let show_item = MenuItemBuilder::with_id("show", "Show Pane").build(app)?;
            let hide_item = MenuItemBuilder::with_id("hide", "Hide Pane").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&hide_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(tauri::include_image!("icons/32x32.png"))
                .menu(&tray_menu)
                .tooltip("Pane — Clipboard Manager")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => show_window(app),
                        "hide" => hide_window(app),
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // ── Global Hotkey: Ctrl+Shift+V ──
            let shortcut: Shortcut = "ctrl+shift+v".parse().expect("Invalid shortcut");
            let app_handle = app.handle().clone();

            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    // Siempre mostrar — Escape oculta desde el frontend
                    show_window(&app_handle);
                }
            })?;

            Ok(())
        })
        // ── Interceptar eventos de ventana ──────────────────
        .on_window_event(|window, event| {
            match event {
                // Cerrar (X) → ocultar en vez de cerrar
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                    WINDOW_VISIBLE.store(false, Ordering::Relaxed);
                }
                // Perdió foco → marcar como no visible (para que toggle funcione)
                tauri::WindowEvent::Focused(false) => {
                    WINDOW_VISIBLE.store(false, Ordering::Relaxed);
                }
                // Ganó foco → marcar como visible
                tauri::WindowEvent::Focused(true) => {
                    WINDOW_VISIBLE.store(true, Ordering::Relaxed);
                }
                _ => {}
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
            commands::hide_app_window,
            commands::get_all_settings,
            commands::set_setting,
            commands::get_collections,
            commands::create_collection,
            commands::delete_collection,
            commands::rename_collection,
            commands::set_clip_collection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
