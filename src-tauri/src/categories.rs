// categories.rs — Detección automática del tipo de contenido copiado.
// Analiza el texto y lo clasifica como: url, code, color, html, o text.

/// Detecta el tipo de contenido de un texto.
/// Retorna uno de: "url", "code", "color", "html", "text"
pub fn detect_content_type(text: &str) -> &'static str {
    let trimmed = text.trim();

    // 1. Color — hex (#fff, #ffffff), rgb(), hsl()
    if is_color(trimmed) {
        return "color";
    }

    // 2. URL — empieza con http://, https://, o www.
    if is_url(trimmed) {
        return "url";
    }

    // 3. HTML — contiene tags HTML
    if is_html(trimmed) {
        return "html";
    }

    // 4. Código — heurísticas basadas en keywords y patrones
    if is_code(trimmed) {
        return "code";
    }

    // 5. Default: texto plano
    "text"
}

/// Detecta si el texto es un color CSS.
/// Soporta: #RGB, #RRGGBB, #RRGGBBAA, rgb(), rgba(), hsl(), hsla()
fn is_color(text: &str) -> bool {
    // Solo una línea (los colores no son multilínea)
    if text.contains('\n') {
        return false;
    }

    // Hex colors: #fff, #ffffff, #ffffffff
    if text.starts_with('#') {
        let hex = &text[1..];
        let len = hex.len();
        // 3, 4, 6, u 8 caracteres hex válidos
        if (len == 3 || len == 4 || len == 6 || len == 8)
            && hex.chars().all(|c| c.is_ascii_hexdigit())
        {
            return true;
        }
    }

    // Funciones CSS: rgb(), rgba(), hsl(), hsla()
    let lower = text.to_lowercase();
    if (lower.starts_with("rgb(")
        || lower.starts_with("rgba(")
        || lower.starts_with("hsl(")
        || lower.starts_with("hsla("))
        && lower.ends_with(')')
    {
        return true;
    }

    false
}

/// Detecta si el texto es una URL.
fn is_url(text: &str) -> bool {
    // Solo una línea (una URL no es multilínea)
    if text.contains('\n') || text.contains(' ') {
        return false;
    }

    text.starts_with("http://")
        || text.starts_with("https://")
        || text.starts_with("ftp://")
        || (text.starts_with("www.") && text.contains('.'))
}

/// Detecta si el texto contiene HTML.
fn is_html(text: &str) -> bool {
    // Buscar tags HTML comunes (apertura y cierre)
    let has_tags = text.contains("</") && text.contains('>');
    if has_tags {
        return true;
    }

    // Tags auto-cerrados comunes
    let html_patterns = ["<br", "<hr", "<img ", "<input ", "<!DOCTYPE", "<html"];
    html_patterns.iter().any(|p| text.contains(p))
}

/// Detecta si el texto parece ser código fuente.
/// Usa heurísticas: keywords de programación, patrones sintácticos, indentación.
fn is_code(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();

    // Texto de una sola línea muy corto probablemente no es código
    // (excepto si tiene patrones muy claros)
    if lines.len() == 1 && text.len() < 20 {
        // Excepción: patrones cortos pero claramente código
        let short_patterns = ["=>", "->", "===", "!==", "&&", "||", "::"];
        if !short_patterns.iter().any(|p| text.contains(p)) {
            return false;
        }
    }

    // Contar cuántos indicadores de código encontramos
    let mut score = 0;

    // Keywords de programación comunes
    let keywords = [
        "function ",
        "const ",
        "let ",
        "var ",
        "import ",
        "export ",
        "class ",
        "interface ",
        "type ",
        "enum ",
        "struct ",
        "def ",
        "fn ",
        "pub ",
        "async ",
        "await ",
        "return ",
        "if ",
        "else ",
        "for ",
        "while ",
        "try ",
        "catch ",
        "throw ",
        "new ",
        "console.log",
        "println!",
        "print(",
        "require(",
        "from '",
        "from \"",
    ];

    for kw in &keywords {
        if text.contains(kw) {
            score += 1;
        }
    }

    // Patrones sintácticos
    let syntax_patterns = [
        "=> {", "=> ", "->", "===", "!==", "#{", "${", " = {", "();", ");", "};", "];", "/**",
        "///", "//", "/*",
    ];

    for pat in &syntax_patterns {
        if text.contains(pat) {
            score += 1;
        }
    }

    // Indentación consistente (tabs o espacios al inicio)
    let indented_lines = lines
        .iter()
        .filter(|l| l.starts_with("  ") || l.starts_with('\t'))
        .count();
    if lines.len() > 2 && indented_lines as f32 / lines.len() as f32 > 0.3 {
        score += 2;
    }

    // Llaves o corchetes balanceados (común en código)
    let open_braces = text.matches('{').count();
    let close_braces = text.matches('}').count();
    if open_braces > 0 && open_braces == close_braces {
        score += 1;
    }

    // Un score de 2+ indica código con alta probabilidad
    score >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors() {
        assert_eq!(detect_content_type("#fff"), "color");
        assert_eq!(detect_content_type("#ff5733"), "color");
        assert_eq!(detect_content_type("#ff573380"), "color");
        assert_eq!(detect_content_type("rgb(255, 87, 51)"), "color");
        assert_eq!(detect_content_type("hsl(9, 100%, 60%)"), "color");
    }

    #[test]
    fn test_urls() {
        assert_eq!(detect_content_type("https://google.com"), "url");
        assert_eq!(detect_content_type("http://localhost:3000"), "url");
        assert_eq!(detect_content_type("www.example.com"), "url");
    }

    #[test]
    fn test_code() {
        assert_eq!(
            detect_content_type("function hello() {\n  return 'hi';\n}"),
            "code"
        );
        assert_eq!(
            detect_content_type("const x = () => {\n  console.log('test');\n};"),
            "code"
        );
        assert_eq!(detect_content_type("import React from 'react';"), "code");
    }

    #[test]
    fn test_html() {
        assert_eq!(detect_content_type("<div>hello</div>"), "html");
        assert_eq!(detect_content_type("<!DOCTYPE html>"), "html");
    }

    #[test]
    fn test_plain_text() {
        assert_eq!(detect_content_type("hello world"), "text");
        assert_eq!(detect_content_type("just some notes"), "text");
    }
}
