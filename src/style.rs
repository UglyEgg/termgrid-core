use serde::{Deserialize, Serialize};

fn is_false(v: &bool) -> bool {
    !*v
}

/// Helper for serde `skip_serializing_if` on op `style` fields.
pub fn is_plain_style(s: &Style) -> bool {
    *s == Style::plain()
}

/// A minimal style model compatible with classic ANSI SGR.
///
/// This intentionally stays small; higher layers can extend or map their own
/// style semantics onto this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Style {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<u8>,
    /// ANSI "faint" (SGR 2). Often rendered as dim/low intensity.
    #[serde(default, skip_serializing_if = "is_false")]
    pub dim: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub bold: bool,
    /// ANSI italic (SGR 3). Not universally supported, but common in modern terminals.
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub underline: bool,
    /// ANSI blink (SGR 5). Often disabled by terminals; modeled for completeness.
    #[serde(default, skip_serializing_if = "is_false")]
    pub blink: bool,
    /// ANSI reverse video (SGR 7).
    #[serde(
        default,
        skip_serializing_if = "is_false",
        rename = "reverse",
        alias = "inverse"
    )]
    pub inverse: bool,
    /// ANSI strikethrough (SGR 9).
    #[serde(
        default,
        skip_serializing_if = "is_false",
        rename = "strikethrough",
        alias = "strike"
    )]
    pub strike: bool,
}

impl Style {
    pub const fn plain() -> Self {
        Self {
            fg: None,
            bg: None,
            dim: false,
            bold: false,
            italic: false,
            underline: false,
            blink: false,
            inverse: false,
            strike: false,
        }
    }

    /// Overlay `top` style on this style.
    ///
    /// Semantics (kept intentionally small and deterministic):
    /// - `fg` / `bg`: `top` wins when present; otherwise the base value remains.
    /// - boolean flags (`dim`, `bold`, `italic`, `underline`, `blink`, `inverse`, `strike`): logical OR.
    #[must_use]
    pub const fn overlay(self, top: Style) -> Style {
        Style {
            fg: match top.fg {
                Some(v) => Some(v),
                None => self.fg,
            },
            bg: match top.bg {
                Some(v) => Some(v),
                None => self.bg,
            },
            dim: self.dim || top.dim,
            bold: self.bold || top.bold,
            italic: self.italic || top.italic,
            underline: self.underline || top.underline,
            blink: self.blink || top.blink,
            inverse: self.inverse || top.inverse,
            strike: self.strike || top.strike,
        }
    }
}
