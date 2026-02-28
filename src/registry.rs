use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use unicode_width::UnicodeWidthStr;

/// The Unicode baseline assumed by BBSstalgia's default profiles.
///
/// This crate is designed so that *width policy is explicit* via [`RenderProfile`].
/// The baseline here documents the project's target when shipping built-in
/// example profiles.
pub const UNICODE_BASELINE: (u8, u8, u8) = (11, 0, 0);

/// Per-glyph metadata captured in a render profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlyphInfo {
    /// The *policy* display width for this glyph or grapheme cluster.
    ///
    /// Typical values are 1 or 2.
    pub width: u8,
}

/// A serializable render profile.
///
/// This is the data you ship and version-control. The engine (or any consumer)
/// loads it to create a `GlyphRegistry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderProfile {
    pub id: String,
    pub version: u32,
    pub glyphs: BTreeMap<String, GlyphInfo>,
}

/// Runtime registry used for width lookups.
#[derive(Debug, Clone)]
pub struct GlyphRegistry {
    profile: RenderProfile,
}

impl GlyphRegistry {
    /// Create a runtime registry from a serialized profile.
    pub fn new(profile: RenderProfile) -> Self {
        Self { profile }
    }

    /// Returns the underlying profile (useful for debugging and version checks).
    pub fn profile(&self) -> &RenderProfile {
        &self.profile
    }

    /// Returns the policy width for the given grapheme cluster.
    ///
    /// - If the grapheme appears in the profile, that width wins.
    /// - Otherwise, we fall back to a conservative Unicode-width estimate.
    pub fn width(&self, grapheme: &str) -> u8 {
        if let Some(info) = self.profile.glyphs.get(grapheme) {
            return info.width.clamp(1, 2);
        }
        // Fallback: do the best we can. This is *not* authoritative.
        let w = UnicodeWidthStr::width(grapheme);
        let w = if w == 0 { 1 } else { w };
        (w.min(2)) as u8
    }
}

impl RenderProfile {
    pub fn empty(id: impl Into<String>, version: u32) -> Self {
        Self {
            id: id.into(),
            version,
            glyphs: BTreeMap::new(),
        }
    }

    /// Built-in example profile tuned for BBSstalgia + xterm.js.
    ///
    /// This is intentionally small and exists to provide a safe starting point.
    /// Real deployments should version-control their own profile JSON.
    ///
    /// The returned profile matches `testdata/profile_example.json`.
    pub fn bbsstalgia_xtermjs_unicode11_example() -> Self {
        let mut p = RenderProfile::empty("bbsstalgia-xtermjs-unicode11", 1);
        p.set_width("🙂", 2);
        p.set_width("⚙️", 2);
        p.set_width("🧠", 2);
        // HEART SUIT is commonly rendered as narrow unless paired with VS-16.
        // We include it as an example of an explicit override.
        p.set_width("❤", 1);
        p
    }
}

impl RenderProfile {
    /// Set or replace the width policy for a glyph/grapheme.
    pub fn set_width(&mut self, glyph: impl Into<String>, width: u8) {
        let w = width.clamp(1, 2);
        self.glyphs.insert(glyph.into(), GlyphInfo { width: w });
    }

    /// Merge another profile's glyph table into this one (other wins on conflicts).
    ///
    /// This is intentionally a simple data operation. Profile identity/versioning
    /// is left to the caller.
    pub fn merge_glyphs_from(&mut self, other: &RenderProfile) {
        for (g, info) in &other.glyphs {
            self.glyphs.insert(g.clone(), info.clone());
        }
    }
}
