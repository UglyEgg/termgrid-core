use crate::{Cell, Grid, Style};

/// Render a grid to an ANSI string.
///
/// This is intentionally conservative:
/// - Continuation cells are skipped (the preceding wide glyph consumes them).
/// - Lines are terminated with a reset to avoid style bleed.
pub fn grid_to_ansi(grid: &Grid) -> String {
    let mut out = String::new();
    let mut current = Style::plain();

    for row in grid.rows() {
        emit_row(&mut out, row, &mut current);
        // Reset at end of line.
        out.push_str("\x1b[0m");
        current = Style::plain();
        out.push('\n');
    }

    out
}

fn emit_row(out: &mut String, row: &[Cell], current: &mut Style) {
    let mut cur = *current;
    for cell in row {
        match cell {
            Cell::Empty => {
                // Empty cells are rendered as plain spaces. If we are currently in a
                // non-plain SGR state, reset so the space doesn't inherit styling.
                if cur != Style::plain() {
                    out.push_str("\x1b[0m");
                    cur = Style::plain();
                }
                out.push(' ');
            }
            Cell::Continuation => {
                // Skip. The previous glyph consumed this cell.
            }
            Cell::Glyph { grapheme, style } => {
                if *style != cur {
                    emit_style(out, *style);
                    cur = *style;
                }
                out.push_str(grapheme);
            }
        }
    }
    *current = cur;
}

fn emit_style(out: &mut String, style: Style) {
    // Reset, then re-apply.
    out.push_str("\x1b[0m");

    let mut parts: Vec<String> = Vec::new();
    if style.dim {
        parts.push("2".to_string());
    }
    if style.bold {
        parts.push("1".to_string());
    }
    if style.italic {
        parts.push("3".to_string());
    }
    if style.underline {
        parts.push("4".to_string());
    }
    if style.blink {
        parts.push("5".to_string());
    }
    if style.inverse {
        parts.push("7".to_string());
    }
    if style.strike {
        parts.push("9".to_string());
    }
    if let Some(fg) = style.fg {
        parts.push(format!("38;5;{}", fg));
    }
    if let Some(bg) = style.bg {
        parts.push(format!("48;5;{}", bg));
    }

    if !parts.is_empty() {
        out.push_str("\x1b[");
        out.push_str(&parts.join(";"));
        out.push('m');
    }
}
