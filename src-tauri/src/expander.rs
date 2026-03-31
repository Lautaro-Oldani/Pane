// expander.rs — Text Expansion engine.
// Escucha todas las teclas globalmente usando rdev.
// Cuando detecta un trigger (ej: "/greeting" + espacio), lo reemplaza
// por el contenido del shortcut usando backspaces + clipboard paste.

use arboard::Clipboard;
use rdev::{listen, simulate, EventType, Key};
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
        // Cargar shortcuts de la DB al iniciar
        let shortcuts = Self::load_shortcuts(&db_path);
        Self {
            buffer: String::new(),
            db_path,
            shortcuts,
        }
    }

    /// Carga los shortcuts de la DB como pares (trigger, content)
    fn load_shortcuts(db_path: &PathBuf) -> Vec<(String, String)> {
        db::get_shortcuts(db_path)
            .unwrap_or_default()
            .into_iter()
            .map(|s| (s.trigger, s.content))
            .collect()
    }

    /// Recarga los shortcuts desde la DB (se llama cuando se agregan/borran)
    pub fn reload(&mut self) {
        self.shortcuts = Self::load_shortcuts(&self.db_path);
    }

    /// Procesa una tecla presionada. Retorna Some((trigger_len, content)) si hay match.
    fn on_key(&mut self, key: &Key) -> Option<(usize, String)> {
        match key {
            // Letras, números, símbolos — agregar al buffer
            Key::Unknown(code) => {
                // En rdev, las teclas se reportan como KeyPress con el código
                // pero los caracteres reales vienen como Key::Unknown en Windows
                // Ignoramos estos
                let _ = code;
                None
            }
            // Espacio o Enter — momento de chequear triggers
            Key::Space | Key::Return => {
                let result = self.check_triggers();
                self.buffer.clear();
                result
            }
            // Backspace — borrar último carácter del buffer
            Key::Backspace => {
                self.buffer.pop();
                None
            }
            // Tab, Escape, flechas, etc. — limpiar buffer
            Key::Tab | Key::Escape | Key::UpArrow | Key::DownArrow
            | Key::LeftArrow | Key::RightArrow => {
                self.buffer.clear();
                None
            }
            // Cualquier otra tecla — intentar convertir a char
            _ => {
                if let Some(c) = key_to_char(key) {
                    self.buffer.push(c);
                    // Limitar buffer a 100 chars para no crecer infinitamente
                    if self.buffer.len() > 100 {
                        self.buffer.drain(..self.buffer.len() - 100);
                    }
                }
                None
            }
        }
    }

    /// Chequea si el buffer termina con algún trigger.
    /// Case-insensitive. Si hay triggers superpuestos, gana el más largo.
    fn check_triggers(&self) -> Option<(usize, String)> {
        let buffer_lower = self.buffer.to_lowercase();
        let mut best: Option<(usize, String)> = None;

        for (trigger, content) in &self.shortcuts {
            let trigger_lower = trigger.to_lowercase();
            if buffer_lower.ends_with(&trigger_lower) {
                // Si este trigger es más largo que el mejor encontrado, reemplazar
                if best.as_ref().map_or(true, |(len, _)| trigger.len() + 1 > *len) {
                    best = Some((trigger.len() + 1, content.clone()));
                }
            }
        }
        best
    }
}

/// Convierte una Key de rdev a un char.
/// rdev reporta teclas como Key::KeyA, Key::Num1, etc.
fn key_to_char(key: &Key) -> Option<char> {
    match key {
        Key::KeyA => Some('a'), Key::KeyB => Some('b'), Key::KeyC => Some('c'),
        Key::KeyD => Some('d'), Key::KeyE => Some('e'), Key::KeyF => Some('f'),
        Key::KeyG => Some('g'), Key::KeyH => Some('h'), Key::KeyI => Some('i'),
        Key::KeyJ => Some('j'), Key::KeyK => Some('k'), Key::KeyL => Some('l'),
        Key::KeyM => Some('m'), Key::KeyN => Some('n'), Key::KeyO => Some('o'),
        Key::KeyP => Some('p'), Key::KeyQ => Some('q'), Key::KeyR => Some('r'),
        Key::KeyS => Some('s'), Key::KeyT => Some('t'), Key::KeyU => Some('u'),
        Key::KeyV => Some('v'), Key::KeyW => Some('w'), Key::KeyX => Some('x'),
        Key::KeyY => Some('y'), Key::KeyZ => Some('z'),
        Key::Num0 => Some('0'), Key::Num1 => Some('1'), Key::Num2 => Some('2'),
        Key::Num3 => Some('3'), Key::Num4 => Some('4'), Key::Num5 => Some('5'),
        Key::Num6 => Some('6'), Key::Num7 => Some('7'), Key::Num8 => Some('8'),
        Key::Num9 => Some('9'),
        Key::Dot => Some('.'), Key::Comma => Some(','),
        Key::SemiColon => Some(';'), Key::Quote => Some('\''),
        Key::BackQuote => Some('`'),
        Key::Slash => Some('/'), Key::BackSlash => Some('\\'),
        Key::Minus => Some('-'), Key::Equal => Some('='),
        Key::LeftBracket => Some('['), Key::RightBracket => Some(']'),
        _ => None,
    }
}

/// Ejecuta el reemplazo: borra el trigger con backspaces y pega el contenido.
fn perform_replacement(trigger_len: usize, content: &str) {
    // Simular backspaces para borrar el trigger + el espacio
    for _ in 0..trigger_len {
        simulate_key(Key::Backspace);
        thread::sleep(Duration::from_millis(5));
    }

    // Pequeña pausa para que los backspaces se procesen
    thread::sleep(Duration::from_millis(30));

    // Guardar el clipboard actual, pegar el contenido, restaurar
    if let Ok(mut clipboard) = Clipboard::new() {
        // Guardar lo que había en el clipboard
        let old_text = clipboard.get_text().ok();

        // Poner el contenido del shortcut en el clipboard
        set_skip_next(); // Evitar que el monitor lo guarde
        let _ = clipboard.set_text(content);

        // Pequeña pausa para que el clipboard se actualice
        thread::sleep(Duration::from_millis(30));

        // Simular Ctrl+V para pegar
        simulate_key_combo(Key::ControlLeft, Key::KeyV);

        // Esperar a que se pegue y restaurar el clipboard original
        thread::sleep(Duration::from_millis(100));
        if let Some(old) = old_text {
            set_skip_next();
            let _ = clipboard.set_text(&old);
        }
    }
}

/// Simula presionar y soltar una tecla.
fn simulate_key(key: Key) {
    let _ = simulate(&EventType::KeyPress(key));
    thread::sleep(Duration::from_millis(2));
    let _ = simulate(&EventType::KeyRelease(key));
}

/// Simula una combinación de teclas (ej: Ctrl+V).
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

/// Handle global para poder recargar los shortcuts desde los commands.
static EXPANDER_HANDLE: Mutex<Option<Arc<Mutex<TypeBuffer>>>> = Mutex::new(None);

/// Recarga los shortcuts desde la DB. Se llama cuando se crean/borran/editan shortcuts.
pub fn reload_shortcuts() {
    if let Ok(guard) = EXPANDER_HANDLE.lock() {
        if let Some(buffer) = guard.as_ref() {
            if let Ok(mut buf) = buffer.lock() {
                buf.reload();
            }
        }
    }
}

/// Inicia el text expander en un thread separado.
pub fn start_expander(db_path: PathBuf) {
    let buffer = Arc::new(Mutex::new(TypeBuffer::new(db_path)));

    // Guardar referencia global para poder recargar
    if let Ok(mut guard) = EXPANDER_HANDLE.lock() {
        *guard = Some(buffer.clone());
    }

    thread::spawn(move || {
        // rdev::listen bloquea el thread y llama al callback por cada evento de teclado
        if let Err(e) = listen(move |event| {
            if let EventType::KeyPress(key) = event.event_type {
                if let Ok(mut buf) = buffer.lock() {
                    if let Some((trigger_len, content)) = buf.on_key(&key) {
                        // Soltar el lock antes de hacer el reemplazo
                        drop(buf);
                        perform_replacement(trigger_len, &content);
                    }
                }
            }
        }) {
            eprintln!("Text expander error: {:?}", e);
        }
    });
}
