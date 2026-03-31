// expander.rs — Text Expansion engine.
// Escucha todas las teclas globalmente usando rdev.
// Cuando detecta un trigger (ej: "/greeting" + espacio), lo reemplaza
// por el contenido del shortcut usando backspaces + clipboard paste.

use arboard::Clipboard;
use rdev::{listen, simulate, Event, EventType, Key};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::clipboard::set_skip_next;
use crate::db;

/// Buffer que acumula las teclas que el usuario escribe.
/// Cuando detecta un trigger al final del buffer, ejecuta el reemplazo.
struct TypeBuffer {
    buffer: String,
    db_path: PathBuf,
    // Cache de shortcuts para no leer la DB en cada tecla
    shortcuts: Vec<(String, String)>, // (trigger, content)
}

impl TypeBuffer {
    fn new(db_path: PathBuf) -> Self {
        let shortcuts = Self::load_shortcuts(&db_path);
        Self {
            buffer: String::new(),
            db_path,
            shortcuts,
        }
    }

    fn load_shortcuts(db_path: &PathBuf) -> Vec<(String, String)> {
        db::get_shortcuts(db_path)
            .unwrap_or_default()
            .into_iter()
            .map(|s| (s.trigger, s.content))
            .collect()
    }

    pub fn reload(&mut self) {
        self.shortcuts = Self::load_shortcuts(&self.db_path);
    }

    /// Procesa un evento de teclado. Usa event.name para obtener el caracter real
    /// según el layout del teclado del usuario (funciona con español, inglés, etc.)
    fn on_event(&mut self, event: &Event) -> Option<(usize, String)> {
        if let EventType::KeyPress(key) = &event.event_type {
            match key {
                // Espacio o Enter — chequear triggers
                Key::Space | Key::Return => {
                    let result = self.check_triggers();
                    self.buffer.clear();
                    return result;
                }
                // Backspace — borrar último carácter del buffer
                Key::Backspace => {
                    self.buffer.pop();
                    return None;
                }
                // Teclas de navegación — limpiar buffer
                Key::Tab | Key::Escape | Key::UpArrow | Key::DownArrow
                | Key::LeftArrow | Key::RightArrow | Key::Home | Key::End
                | Key::PageUp | Key::PageDown => {
                    self.buffer.clear();
                    return None;
                }
                // Modificadores solos — ignorar
                Key::ShiftLeft | Key::ShiftRight | Key::ControlLeft | Key::ControlRight
                | Key::Alt | Key::AltGr | Key::MetaLeft | Key::MetaRight | Key::CapsLock => {
                    return None;
                }
                _ => {}
            }

            // Usar event.name para obtener el caracter real del OS.
            // Esto resuelve correctamente Shift+7 = "/" en teclado español,
            // AltGr+combinaciones, etc.
            if let Some(name) = &event.name {
                if name.len() == 1 {
                    // Un solo caracter — es una tecla normal
                    let c = name.chars().next().unwrap();
                    self.buffer.push(c);
                    // Limitar buffer
                    if self.buffer.len() > 100 {
                        self.buffer.drain(..self.buffer.len() - 100);
                    }
                } else {
                    // Tecla especial con nombre largo (ej: "Delete", "F1") — limpiar buffer
                    self.buffer.clear();
                }
            }
        }
        None
    }

    /// Chequea si el buffer termina con algún trigger.
    /// Case-insensitive. Si hay triggers superpuestos, gana el más largo.
    fn check_triggers(&self) -> Option<(usize, String)> {
        let buffer_lower = self.buffer.to_lowercase();
        let mut best: Option<(usize, String)> = None;

        for (trigger, content) in &self.shortcuts {
            let trigger_lower = trigger.to_lowercase();
            if buffer_lower.ends_with(&trigger_lower) {
                if best.as_ref().map_or(true, |(len, _)| trigger.len() + 1 > *len) {
                    best = Some((trigger.len() + 1, content.clone()));
                }
            }
        }
        best
    }
}

/// Ejecuta el reemplazo: borra el trigger con backspaces y pega el contenido.
fn perform_replacement(trigger_len: usize, content: &str) {
    for _ in 0..trigger_len {
        simulate_key(Key::Backspace);
        thread::sleep(Duration::from_millis(5));
    }

    thread::sleep(Duration::from_millis(30));

    if let Ok(mut clipboard) = Clipboard::new() {
        let old_text = clipboard.get_text().ok();

        set_skip_next();
        let _ = clipboard.set_text(content);

        thread::sleep(Duration::from_millis(30));

        simulate_key_combo(Key::ControlLeft, Key::KeyV);

        thread::sleep(Duration::from_millis(100));
        if let Some(old) = old_text {
            set_skip_next();
            let _ = clipboard.set_text(&old);
        }
    }
}

fn simulate_key(key: Key) {
    let _ = simulate(&EventType::KeyPress(key));
    thread::sleep(Duration::from_millis(2));
    let _ = simulate(&EventType::KeyRelease(key));
}

fn simulate_key_combo(modifier: Key, key: Key) {
    let _ = simulate(&EventType::KeyPress(modifier));
    thread::sleep(Duration::from_millis(5));
    let _ = simulate(&EventType::KeyPress(key));
    thread::sleep(Duration::from_millis(5));
    let _ = simulate(&EventType::KeyRelease(key));
    thread::sleep(Duration::from_millis(5));
    let _ = simulate(&EventType::KeyRelease(modifier));
}

// ── Estado global para recargar shortcuts ────────────────────────────

static EXPANDER_HANDLE: Mutex<Option<Arc<Mutex<TypeBuffer>>>> = Mutex::new(None);

pub fn reload_shortcuts() {
    if let Ok(guard) = EXPANDER_HANDLE.lock() {
        if let Some(buffer) = guard.as_ref() {
            if let Ok(mut buf) = buffer.lock() {
                buf.reload();
            }
        }
    }
}

pub fn start_expander(db_path: PathBuf) {
    let buffer = Arc::new(Mutex::new(TypeBuffer::new(db_path)));

    if let Ok(mut guard) = EXPANDER_HANDLE.lock() {
        *guard = Some(buffer.clone());
    }

    thread::spawn(move || {
        // Pasamos el evento completo (no solo la key) para acceder a event.name
        if let Err(e) = listen(move |event| {
            if let Ok(mut buf) = buffer.lock() {
                if let Some((trigger_len, content)) = buf.on_event(&event) {
                    drop(buf);
                    perform_replacement(trigger_len, &content);
                }
            }
        }) {
            eprintln!("Text expander error: {:?}", e);
        }
    });
}
