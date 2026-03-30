// clipboard.rs — Monitor de clipboard en tiempo real.
// Usa clipboard-master que internamente usa la API de Windows
// (AddClipboardFormatListener / WM_CLIPBOARDUPDATE) para detectar
// cuando CUALQUIER app copia algo al clipboard.

use arboard::Clipboard;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use image::ImageEncoder;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

use crate::categories;
use crate::db;

// ── Flag global para ignorar cambios propios ─────────────────────────

/// Cuando Pane copia algo al clipboard (porque el usuario hizo click en un clip),
/// este flag se pone en true para que el monitor ignore ese cambio
/// y no lo guarde como un clip nuevo (duplicado).
/// AtomicBool es thread-safe: se puede leer/escribir desde cualquier thread.
static SKIP_NEXT: AtomicBool = AtomicBool::new(false);

/// Activa el flag. Llamar ANTES de escribir al clipboard desde la app.
pub fn set_skip_next() {
    SKIP_NEXT.store(true, Ordering::Relaxed);
}

// ── Handler del clipboard ────────────────────────────────────────────

/// Struct que implementa ClipboardHandler (trait de clipboard-master).
/// Cada vez que el clipboard cambia, clipboard-master llama a on_clipboard_change().
struct Handler {
    app: AppHandle,    // Handle de Tauri para emitir eventos al frontend
    db_path: PathBuf,  // Ruta a pane.db
    last_hash: String, // Hash del último contenido procesado (evita duplicados consecutivos)
}

/// Implementación del trait ClipboardHandler.
/// Es como implementar una interfaz en TypeScript.
impl ClipboardHandler for Handler {
    /// Se llama cada vez que el clipboard del sistema cambia.
    fn on_clipboard_change(&mut self) -> CallbackResult {
        // Si nosotros mismos copiamos algo, ignorar este cambio.
        // swap() lee el valor actual y lo cambia a false en una operación atómica.
        if SKIP_NEXT.swap(false, Ordering::Relaxed) {
            return CallbackResult::Next;
        }

        // Intentar leer el clipboard.
        // Clipboard::new() puede fallar si otro proceso tiene el clipboard bloqueado.
        if let Ok(mut clipboard) = Clipboard::new() {
            // Intentar leer texto primero (es lo más común)
            let handled_text = if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() {
                    self.handle_text(&text);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Si no era texto, intentar leer imagen
            if !handled_text {
                if let Ok(img) = clipboard.get_image() {
                    self.handle_image(img);
                }
            }
        }

        // CallbackResult::Next = seguir escuchando cambios
        CallbackResult::Next
    }

    /// Se llama si hay un error al escuchar el clipboard.
    fn on_clipboard_error(&mut self, error: std::io::Error) -> CallbackResult {
        eprintln!("Clipboard monitor error: {error}");
        CallbackResult::Next // Seguir intentando
    }
}

impl Handler {
    /// Límite de contenido: 50KB. Textos más largos se truncan.
    const MAX_CONTENT_BYTES: usize = 50 * 1024;

    /// Procesa un texto copiado al clipboard.
    fn handle_text(&mut self, text: &str) {
        // Truncar textos muy largos (ej: copiar un archivo entero)
        let text = if text.len() > Self::MAX_CONTENT_BYTES {
            // Buscar un punto de corte seguro en UTF-8
            let mut end = Self::MAX_CONTENT_BYTES;
            while end > 0 && !text.is_char_boundary(end) {
                end -= 1;
            }
            &text[..end]
        } else {
            text
        };

        let hash = calculate_hash(text);

        // Si es el mismo contenido que el último, ignorar
        if hash == self.last_hash {
            return;
        }
        self.last_hash = hash.clone();

        // Detectar tipo de contenido automáticamente
        let content_type = categories::detect_content_type(text);

        // Preview: primeros 200 caracteres
        let preview = truncate_preview(text, 200);

        match db::insert_clip(&self.db_path, text, content_type, &preview, &hash, None) {
            Ok(clip) => {
                let _ = self.app.emit("new-clip", &clip);
                // Ejecutar limpieza después de cada nuevo clip
                let _ = db::run_cleanup(&self.db_path);
            }
            Err(e) => eprintln!("Error saving clip: {e}"),
        }
    }

    /// Procesa una imagen copiada al clipboard.
    /// arboard nos da los bytes RGBA crudos. Los convertimos a PNG y luego a base64.
    fn handle_image(&mut self, img: arboard::ImageData) {
        // Calcular hash de los bytes crudos de la imagen
        let hash = calculate_hash_bytes(&img.bytes);

        if hash == self.last_hash {
            return;
        }
        self.last_hash = hash.clone();

        // Convertir RGBA bytes a PNG en memoria
        let png_base64 = match encode_image_to_base64(&img) {
            Ok(b64) => b64,
            Err(e) => {
                eprintln!("Error encoding image: {e}");
                return;
            }
        };

        // Preview descriptivo con las dimensiones
        let preview = format!("Image ({}×{})", img.width, img.height);
        // content guarda una descripción, image_base64 guarda la imagen real
        let content = format!("Image {}×{}", img.width, img.height);

        match db::insert_clip(
            &self.db_path,
            &content,
            "image",
            &preview,
            &hash,
            Some(&png_base64),
        ) {
            Ok(clip) => {
                let _ = self.app.emit("new-clip", &clip);
                let _ = db::run_cleanup(&self.db_path);
            }
            Err(e) => eprintln!("Error saving image clip: {e}"),
        }
    }
}

// ── Funciones auxiliares ─────────────────────────────────────────────

/// Calcula un hash de un string.
fn calculate_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Calcula un hash de bytes (para imágenes).
/// Usa solo los primeros 64KB para no ser lento con imágenes grandes.
fn calculate_hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    // Hashear solo los primeros 64KB — suficiente para detectar duplicados
    // sin procesar megabytes de datos en cada cambio
    let sample = if bytes.len() > 65536 {
        &bytes[..65536]
    } else {
        bytes
    };
    sample.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Convierte una imagen RGBA (de arboard) a PNG codificado en base64.
/// Proceso: RGBA bytes -> PNG bytes -> base64 string
fn encode_image_to_base64(img: &arboard::ImageData) -> Result<String, String> {
    let mut png_bytes: Vec<u8> = Vec::new();

    // PngEncoder escribe los bytes PNG en el Vec
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);

    // write_image convierte los bytes RGBA crudos a formato PNG.
    // Parámetros: bytes, ancho, alto, formato de color (RGBA con 8 bits por canal)
    encoder
        .write_image(
            &img.bytes,
            img.width as u32,
            img.height as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("PNG encode error: {e}"))?;

    // Codificar los bytes PNG como base64 para almacenar como texto
    Ok(BASE64.encode(&png_bytes))
}

/// Trunca un texto a max_chars caracteres, agregando "..." si se truncó.
fn truncate_preview(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let end_idx = text
            .char_indices()
            .nth(max_chars)
            .map(|(idx, _)| idx)
            .unwrap_or(text.len());
        format!("{}...", &text[..end_idx])
    }
}

// ── Inicio del monitor ──────────────────────────────────────────────

/// Inicia el monitor de clipboard en un thread separado.
pub fn start_clipboard_monitor(app: AppHandle, db_path: PathBuf) {
    let last_hash = db::get_last_hash(&db_path)
        .unwrap_or(None)
        .unwrap_or_default();

    let handler = Handler {
        app,
        db_path,
        last_hash,
    };

    std::thread::spawn(move || {
        match Master::new(handler) {
            Ok(mut master) => {
                if let Err(e) = master.run() {
                    eprintln!("Clipboard monitor crashed: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to create clipboard monitor: {e}");
            }
        }
    });
}
