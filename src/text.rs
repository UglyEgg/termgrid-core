//! Styled text utilities.
//!
//! This module provides a small, generic representation of "styled spans" and
//! helper functions for measuring and manipulating them in terms of terminal
//! cell width.

use crate::registry::GlyphRegistry;
use crate::{style::is_plain_style, Style};
use serde::{Deserialize, Serialize};

/// A segment of text with a single style.
///
/// Invariants expected by helpers in this module:
/// - `text` may be empty, but most helpers will drop empty spans.
/// - Adjacent spans with identical `style` can be coalesced.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    #[serde(default, skip_serializing_if = "is_plain_style")]
    pub style: Style,
}

impl Span {
    /// Construct a new span from text and style.
    ///
    /// Note: many helpers treat empty spans as non-semantic and will drop
    /// them during normalization.
    pub fn new<T: Into<String>>(text: T, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

/// A convenience alias for a sequence of styled spans.
pub type Spans = Vec<Span>;

/// Returns the concatenated plain-text content of spans.
pub fn spans_plain_text(spans: &[Span]) -> String {
    let mut out = String::new();
    for s in spans {
        out.push_str(&s.text);
    }
    out
}

/// Normalize spans by:
/// - removing empty spans
/// - coalescing adjacent spans with identical styles
pub fn normalize_spans(spans: &[Span]) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    for s in spans {
        if s.text.is_empty() {
            continue;
        }
        if let Some(last) = out.last_mut() {
            if last.style == s.style {
                last.text.push_str(&s.text);
                continue;
            }
        }
        out.push(s.clone());
    }
    out
}

/// Measure a plain string in terminal cells using the provided glyph registry.
///
/// This measures display width by iterating grapheme clusters and consulting
/// the glyph registry for width policy.
pub fn measure_cells_text(registry: &GlyphRegistry, text: &str) -> usize {
    use unicode_segmentation::UnicodeSegmentation;
    let mut w: usize = 0;
    for g in text.graphemes(true) {
        w = w.saturating_add(registry.width(g) as usize);
    }
    w
}

/// Measure spans in terminal cells using the provided glyph registry.
pub fn measure_cells_spans(registry: &GlyphRegistry, spans: &[Span]) -> usize {
    spans
        .iter()
        .map(|s| measure_cells_text(registry, &s.text))
        .sum()
}

fn clip_text_to_cells_internal(
    registry: &GlyphRegistry,
    text: &str,
    max_cells: usize,
) -> (String, usize, bool) {
    use unicode_segmentation::UnicodeSegmentation;

    if max_cells == 0 || text.is_empty() {
        return (String::new(), 0, !text.is_empty());
    }

    let mut out = String::new();
    let mut used: usize = 0;
    let mut clipped = false;

    for g in text.graphemes(true) {
        let gw = registry.width(g) as usize;
        if used.saturating_add(gw) > max_cells {
            clipped = true;
            break;
        }
        used = used.saturating_add(gw);
        out.push_str(g);
    }

    // If we did not iterate all graphemes, we clipped.
    if !clipped {
        // Detect remaining content without re-walking too much.
        // If the output differs from the input, we clipped.
        if out.len() != text.len() {
            clipped = true;
        }
    }

    (out, used, clipped)
}

/// Clip a plain string to at most `w` cells.
///
/// Returns the clipped string and whether clipping occurred.
pub fn clip_to_cells_text(registry: &GlyphRegistry, text: &str, w: usize) -> (String, bool) {
    let (out, _used, clipped) = clip_text_to_cells_internal(registry, text, w);
    (out, clipped)
}

/// Clip spans to at most `w` cells.
///
/// Returns clipped spans and whether clipping occurred.
pub fn clip_to_cells_spans(
    registry: &GlyphRegistry,
    spans: &[Span],
    w: usize,
) -> (Vec<Span>, bool) {
    if w == 0 {
        return (Vec::new(), !spans.is_empty());
    }

    let mut out: Vec<Span> = Vec::new();
    let mut used: usize = 0;
    let mut clipped = false;

    for s in spans {
        if s.text.is_empty() {
            continue;
        }
        let remaining = w.saturating_sub(used);
        if remaining == 0 {
            clipped = true;
            break;
        }
        let (clipped_text, text_used, did_clip) =
            clip_text_to_cells_internal(registry, &s.text, remaining);
        if !clipped_text.is_empty() {
            out.push(Span::new(clipped_text, s.style));
        }
        used = used.saturating_add(text_used);
        if did_clip {
            clipped = true;
            break;
        }
    }

    (normalize_spans(&out), clipped)
}

/// Ellipsize a plain string to at most `w` cells using the provided `ellipsis`.
///
/// If the `ellipsis` itself does not fit, it will be clipped to `w`.
pub fn ellipsis_to_cells_text(
    registry: &GlyphRegistry,
    text: &str,
    w: usize,
    ellipsis: &str,
) -> String {
    if w == 0 {
        return String::new();
    }
    if measure_cells_text(registry, text) <= w {
        return text.to_string();
    }

    let ell_w = measure_cells_text(registry, ellipsis);
    if ell_w >= w {
        let (e, _clipped) = clip_to_cells_text(registry, ellipsis, w);
        return e;
    }

    let avail = w.saturating_sub(ell_w);
    let (prefix, _clipped) = clip_to_cells_text(registry, text, avail);
    let mut out = prefix;
    out.push_str(ellipsis);
    out
}

/// Ellipsize spans to at most `w` cells using the provided `ellipsis_span`.
///
/// If the ellipsis span does not fit, it will be clipped to `w`.
pub fn ellipsis_to_cells_spans(
    registry: &GlyphRegistry,
    spans: &[Span],
    w: usize,
    ellipsis_span: &Span,
) -> Vec<Span> {
    if w == 0 {
        return Vec::new();
    }

    if measure_cells_spans(registry, spans) <= w {
        return normalize_spans(spans);
    }

    let ell_w = measure_cells_text(registry, &ellipsis_span.text);
    if ell_w >= w {
        let (t, _clipped) = clip_to_cells_text(registry, &ellipsis_span.text, w);
        return normalize_spans(&[Span::new(t, ellipsis_span.style)]);
    }

    let avail = w.saturating_sub(ell_w);
    let (mut prefix, _clipped) = clip_to_cells_spans(registry, spans, avail);
    prefix.push(Span::new(ellipsis_span.text.clone(), ellipsis_span.style));
    normalize_spans(&prefix)
}

/// Options controlling span-aware word wrapping.
///
/// These settings are intentionally generic and suitable for help viewers,
/// log viewers, search UIs, and other terminal applications.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WrapOpts {
    /// Preserve whitespace runs exactly as provided.
    ///
    /// When false, whitespace runs are normalized to a single ASCII space, and
    /// leading whitespace at the beginning of a visual line is dropped.
    pub preserve_spaces: bool,

    /// If a single non-whitespace token exceeds the available width, hard-break
    /// it by grapheme cluster.
    pub hard_break_long_tokens: bool,

    /// Trim trailing whitespace at the end of each visual line.
    pub trim_end: bool,

    /// Optional prefix to apply to continuation lines produced by wrapping.
    ///
    /// A continuation line is a visual line created because a token would
    /// exceed the available `width` and the line is wrapped. Explicit newline
    /// characters do not produce continuation lines.
    ///
    /// The prefix is only applied if its measured cell width is strictly less
    /// than `width`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation_prefix: Option<Vec<Span>>,
}

impl Default for WrapOpts {
    fn default() -> Self {
        Self {
            preserve_spaces: false,
            hard_break_long_tokens: true,
            trim_end: true,
            continuation_prefix: None,
        }
    }
}

pub(crate) fn is_default_wrap_opts(o: &WrapOpts) -> bool {
    *o == WrapOpts::default()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TokenKind {
    Text,
    Space,
    Newline,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    text: String,
    style: Style,
    kind: TokenKind,
}

fn tokenize_spans(spans: &[Span], preserve_spaces: bool) -> Vec<Token> {
    let spans = normalize_spans(spans);
    let mut out: Vec<Token> = Vec::new();

    for s in spans {
        // Split on explicit newlines. Newlines are always hard line breaks.
        let mut first = true;
        for part in s.text.split('\n') {
            if !first {
                out.push(Token {
                    text: "\n".to_string(),
                    style: s.style,
                    kind: TokenKind::Newline,
                });
            }
            first = false;

            if part.is_empty() {
                continue;
            }

            // Split into runs of whitespace vs non-whitespace.
            let mut buf = String::new();
            let mut in_space: Option<bool> = None;
            for ch in part.chars() {
                let is_space = ch.is_whitespace();
                match in_space {
                    None => {
                        in_space = Some(is_space);
                        buf.push(ch);
                    }
                    Some(prev) if prev == is_space => {
                        buf.push(ch);
                    }
                    Some(prev) => {
                        // flush
                        if prev {
                            let txt = if preserve_spaces {
                                buf.clone()
                            } else {
                                " ".to_string()
                            };
                            out.push(Token {
                                text: txt,
                                style: s.style,
                                kind: TokenKind::Space,
                            });
                        } else {
                            out.push(Token {
                                text: buf.clone(),
                                style: s.style,
                                kind: TokenKind::Text,
                            });
                        }
                        buf.clear();
                        in_space = Some(is_space);
                        buf.push(ch);
                    }
                }
            }

            if !buf.is_empty() {
                if in_space.unwrap_or(false) {
                    let txt = if preserve_spaces {
                        buf
                    } else {
                        " ".to_string()
                    };
                    out.push(Token {
                        text: txt,
                        style: s.style,
                        kind: TokenKind::Space,
                    });
                } else {
                    out.push(Token {
                        text: buf,
                        style: s.style,
                        kind: TokenKind::Text,
                    });
                }
            }
        }
    }

    out
}

fn trim_trailing_spaces(registry: &GlyphRegistry, spans: &mut Vec<Span>) {
    // Remove trailing whitespace spans, but do not split graphemes in the middle.
    // This function is conservative: it trims ASCII whitespace at the end.
    // If a caller wants more aggressive trimming, do it before constructing spans.
    while let Some(last) = spans.last_mut() {
        if last.text.is_empty() {
            spans.pop();
            continue;
        }
        // Trim only ASCII spaces/tabs at the end.
        let trimmed = last.text.trim_end_matches(&[' ', '\t', '\r'] as &[char]);
        if trimmed.len() == last.text.len() {
            break;
        }
        last.text = trimmed.to_string();
        if last.text.is_empty() {
            spans.pop();
        }
    }

    // Re-normalize to coalesce styles after trimming.
    let norm = normalize_spans(spans);
    spans.clear();
    spans.extend(norm);

    // Keep registry referenced to avoid unused warnings under some feature sets.
    let _ = registry;
}

fn push_token(line: &mut Vec<Span>, tok: &Token) {
    if tok.text.is_empty() {
        return;
    }
    line.push(Span::new(tok.text.clone(), tok.style));
}

fn push_span_text(line: &mut Vec<Span>, text: &str, style: Style) {
    if text.is_empty() {
        return;
    }
    line.push(Span::new(text.to_string(), style));
}

fn split_token_by_width<'a>(
    registry: &GlyphRegistry,
    text: &'a str,
    max_cells: usize,
) -> (String, &'a str) {
    let (chunk, used, _clipped) = clip_text_to_cells_internal(registry, text, max_cells);
    if chunk.is_empty() || used == 0 {
        return (String::new(), text);
    }
    let rest = &text[chunk.len()..];
    (chunk, rest)
}

fn hard_break_token(registry: &GlyphRegistry, tok: &Token, width: usize) -> Vec<Vec<Span>> {
    // Break a single token into multiple visual lines by grapheme cluster.
    // Each produced line contains a single span with the token's style.
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut remaining = tok.text.as_str();

    loop {
        if remaining.is_empty() {
            break;
        }
        let (chunk, used, _clipped) = clip_text_to_cells_internal(registry, remaining, width);
        if chunk.is_empty() || used == 0 {
            // Defensive: avoid infinite loop if width is too small for any grapheme.
            break;
        }
        lines.push(vec![Span::new(chunk.clone(), tok.style)]);
        // Advance remaining by chunk length in bytes.
        remaining = &remaining[chunk.len()..];
    }

    lines
}

/// Wrap spans word-wise into a sequence of visual lines.
///
/// - Word boundaries are whitespace runs.
/// - Newlines always force a hard line break.
/// - When `opts.preserve_spaces` is false, whitespace is normalized to a single
///   ASCII space and leading spaces at the start of a line are dropped.
/// - When a single token exceeds `width` and `opts.hard_break_long_tokens` is
///   true, the token is hard-broken by grapheme cluster.
pub fn wrap_spans_wordwise(
    registry: &GlyphRegistry,
    spans: &[Span],
    width: usize,
    opts: &WrapOpts,
) -> Vec<Vec<Span>> {
    if width == 0 {
        return Vec::new();
    }

    let tokens = tokenize_spans(spans, opts.preserve_spaces);
    let mut q: std::collections::VecDeque<Token> = tokens.into();

    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut line: Vec<Span> = Vec::new();
    let mut line_w: usize = 0;

    let begin_line = |line: &mut Vec<Span>, line_w: &mut usize, continuation: bool| {
        if !continuation {
            return;
        }
        let Some(prefix) = &opts.continuation_prefix else {
            return;
        };
        let mut p = normalize_spans(prefix);
        if p.is_empty() {
            return;
        }
        let pw = measure_cells_spans(registry, &p);
        // Only apply when the prefix fits and leaves at least 1 cell.
        if pw >= width {
            return;
        }
        line.append(&mut p);
        *line_w = line_w.saturating_add(pw);
    };

    let flush_line = |lines: &mut Vec<Vec<Span>>, line: &mut Vec<Span>, line_w: &mut usize| {
        let mut out = normalize_spans(line);
        if opts.trim_end {
            trim_trailing_spaces(registry, &mut out);
        }
        lines.push(out);
        line.clear();
        *line_w = 0;
    };

    while let Some(tok) = q.pop_front() {
        match tok.kind {
            TokenKind::Newline => {
                flush_line(&mut lines, &mut line, &mut line_w);
                // Explicit newlines reset continuation.
            }
            TokenKind::Space => {
                if !opts.preserve_spaces {
                    // drop leading spaces
                    if line.is_empty() {
                        continue;
                    }
                }
                let tok_w = measure_cells_text(registry, &tok.text);
                if line_w.saturating_add(tok_w) > width {
                    flush_line(&mut lines, &mut line, &mut line_w);
                    begin_line(&mut line, &mut line_w, true);
                    // Discard the wrapping whitespace token. This matches
                    // typical word-wrap semantics and avoids leading separators
                    // on continuation lines.
                } else {
                    push_token(&mut line, &tok);
                    line_w = line_w.saturating_add(tok_w);
                }
            }
            TokenKind::Text => {
                let tok_w = measure_cells_text(registry, &tok.text);

                if tok_w > width {
                    if opts.hard_break_long_tokens {
                        // If there is remaining space on the current line,
                        // fill it with the first chunk of this token.
                        if !line.is_empty() && line_w < width {
                            let avail = width - line_w;
                            let (first, rest) = split_token_by_width(registry, &tok.text, avail);
                            if !first.is_empty() {
                                push_span_text(&mut line, &first, tok.style);
                                line_w =
                                    line_w.saturating_add(measure_cells_text(registry, &first));
                            }
                            flush_line(&mut lines, &mut line, &mut line_w);
                            if !rest.is_empty() {
                                q.push_front(Token {
                                    text: rest.to_string(),
                                    style: tok.style,
                                    kind: TokenKind::Text,
                                });
                            }
                            continue;
                        }

                        if !line.is_empty() {
                            flush_line(&mut lines, &mut line, &mut line_w);
                        }

                        let broken = hard_break_token(registry, &tok, width);
                        for (i, b) in broken.into_iter().enumerate() {
                            if i == 0 {
                                lines.push(normalize_spans(&b));
                                continue;
                            }
                            // Continuation lines from hard-break may be prefixed.
                            if opts.continuation_prefix.is_some() {
                                let mut out: Vec<Span> = Vec::new();
                                let mut out_w: usize = 0;
                                begin_line(&mut out, &mut out_w, true);
                                out.extend(normalize_spans(&b));
                                lines.push(normalize_spans(&out));
                            } else {
                                lines.push(normalize_spans(&b));
                            }
                        }
                        continue;
                    } else {
                        if !line.is_empty() {
                            flush_line(&mut lines, &mut line, &mut line_w);
                        }
                        let (clipped, _did) = clip_to_cells_text(registry, &tok.text, width);
                        if !clipped.is_empty() {
                            lines.push(vec![Span::new(clipped, tok.style)]);
                        }
                        continue;
                    }
                }

                if line_w.saturating_add(tok_w) > width {
                    flush_line(&mut lines, &mut line, &mut line_w);
                    begin_line(&mut line, &mut line_w, true);
                }
                push_token(&mut line, &tok);
                line_w = line_w.saturating_add(tok_w);
            }
        }
    }

    if !line.is_empty() || lines.is_empty() {
        flush_line(&mut lines, &mut line, &mut line_w);
    }

    lines
}

// -----------------------------------------------------------------------------
// Highlighting helpers
// -----------------------------------------------------------------------------

/// Apply highlighting to spans using grapheme-index ranges.
///
/// Ranges are expressed as half-open intervals `(start, end)` in **grapheme
/// indices** over the concatenated plain text (`spans_plain_text`).
///
/// Highlighting is applied by *overlaying* the provided `highlight_style` onto
/// the base span style:
/// - `fg` / `bg` fields from `highlight_style` replace the base fields when
///   they are `Some`.
/// - `dim`, `bold`, `italic`, `underline`, `blink`, `inverse`, and `strike` are ORed with the base flags.
///
/// Invalid ranges (where `start >= end`) are ignored.
pub fn apply_highlight(
    spans: &[Span],
    ranges: &[(usize, usize)],
    highlight_style: Style,
) -> Vec<Span> {
    use unicode_segmentation::UnicodeSegmentation;

    if spans.is_empty() || ranges.is_empty() {
        return normalize_spans(spans);
    }

    // Normalize and merge ranges (sorted, non-overlapping).
    let mut rs: Vec<(usize, usize)> = ranges.iter().copied().filter(|(s, e)| s < e).collect();
    if rs.is_empty() {
        return normalize_spans(spans);
    }
    rs.sort_by_key(|(s, _e)| *s);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in rs {
        match merged.last_mut() {
            None => merged.push((s, e)),
            Some((_ls, le)) => {
                if s <= *le {
                    *le = (*le).max(e);
                } else {
                    merged.push((s, e));
                }
            }
        }
    }

    let spans = normalize_spans(spans);
    let mut out: Vec<Span> = Vec::new();

    let mut global_g: usize = 0;
    let mut r_idx: usize = 0;

    for s in spans {
        if s.text.is_empty() {
            continue;
        }

        for g in s.text.graphemes(true) {
            // Advance current range if we've passed it.
            while r_idx < merged.len() && global_g >= merged[r_idx].1 {
                r_idx += 1;
            }

            let in_range = if r_idx < merged.len() {
                let (rs, re) = merged[r_idx];
                global_g >= rs && global_g < re
            } else {
                false
            };

            let style = if in_range {
                s.style.overlay(highlight_style)
            } else {
                s.style
            };

            // Append grapheme with appropriate style, coalescing where possible.
            if let Some(last) = out.last_mut() {
                if last.style == style {
                    last.text.push_str(g);
                } else {
                    out.push(Span::new(g.to_string(), style));
                }
            } else {
                out.push(Span::new(g.to_string(), style));
            }

            global_g = global_g.saturating_add(1);
        }
    }

    normalize_spans(&out)
}
