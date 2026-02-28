//! Generic search helpers.
//!
//! This module intentionally provides small, deterministic building blocks that
//! higher-level UIs can compose into experiences such as an fzf-style picker.

use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// Deterministic folding for matching.
///
/// We apply NFKC normalization and then a Unicode-aware lowercase transform.
///
/// Notes:
/// - This is not a full Unicode case fold in the strict sense, but it is
///   stable and deterministic without requiring additional case-fold tables.
/// - Lowercasing can expand in some scripts, which is fine for our unit-based
///   matching.
fn nfkc_lower(s: &str) -> String {
    // `nfkc()` yields an iterator of chars. We then apply a small, deterministic
    // “casefold-ish” transform suitable for search keys:
    // - Unicode NFKC normalization (handles compatibility forms like ligatures)
    // - Unicode lowercase
    // - Special-case German sharp s (ß/ẞ) → "ss" to allow multi-unit folds
    //
    // Note: This is not a complete implementation of Unicode CaseFolding. We
    // implement the multi-unit fold we rely on for search UX while keeping the
    // mapping back to original grapheme indices stable.
    let mut out = String::new();
    for c in s.nfkc() {
        match c {
            'ß' | 'ẞ' => out.push_str("ss"),
            _ => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// Fold a string into comparable units.
///
/// We fold using NFKC + Unicode-aware lowercase, then split the folded output into grapheme
/// clusters. This permits multi-codepoint folds (e.g. ß → ss) while retaining a
/// stable mapping back to original grapheme indices (for candidate strings).
fn fold_query_units(query: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for g in query.graphemes(true) {
        // Fast path for single-byte ASCII.
        let b = g.as_bytes();
        if b.len() == 1 && b[0].is_ascii() {
            out.push(((b[0] as char).to_ascii_lowercase()).to_string());
            continue;
        }

        let folded: String = nfkc_lower(g);
        for fg in folded.graphemes(true) {
            if !fg.is_empty() {
                out.push(fg.to_string());
            }
        }
    }
    out
}

fn fold_candidate_units(candidate: &str) -> (Vec<&str>, Vec<(String, usize)>) {
    let cgs: Vec<&str> = candidate.graphemes(true).collect();
    let mut units: Vec<(String, usize)> = Vec::new();
    for (i, cg) in cgs.iter().enumerate() {
        let b = cg.as_bytes();
        if b.len() == 1 && b[0].is_ascii() {
            units.push((((b[0] as char).to_ascii_lowercase()).to_string(), i));
            continue;
        }

        let folded: String = nfkc_lower(cg);
        for fg in folded.graphemes(true) {
            if !fg.is_empty() {
                units.push((fg.to_string(), i));
            }
        }
    }
    (cgs, units)
}

/// Returns grapheme indices in `candidate` that match `query` in order.
///
/// Matching is case-insensitive by Unicode NFKC + lowercase folding.
///
/// Case folding may expand a single grapheme into multiple folded units (for
/// example, ß → ss). Returned positions are still grapheme indices in the
/// original `candidate` and collapse multi-unit folds to a single position,
/// which is suitable for highlighting.
///
/// - If `query` is empty, returns `Some(vec![])`.
/// - If all graphemes in `query` can be matched in order, returns their
///   positions as grapheme indices in `candidate`.
/// - Otherwise returns `None`.
///
/// This function performs no scoring. For a scored matcher that also returns
/// match positions suitable for highlighting, use
/// [`fuzzy_match_positions_graphemes`].
pub fn match_positions_graphemes(query: &str, candidate: &str) -> Option<Vec<usize>> {
    let qu: Vec<String> = fold_query_units(query);
    if qu.is_empty() {
        return Some(Vec::new());
    }

    let (_cgs, cu) = fold_candidate_units(candidate);
    let mut q_i: usize = 0;
    let mut raw_positions: Vec<usize> = Vec::new();

    for (unit, orig_i) in cu {
        if q_i >= qu.len() {
            break;
        }
        if unit == qu[q_i] {
            raw_positions.push(orig_i);
            q_i += 1;
            if q_i == qu.len() {
                break;
            }
        }
    }

    if q_i != qu.len() {
        return None;
    }

    // Collapse multi-unit matches that map to the same original candidate
    // grapheme (e.g. ß -> ss) into a single highlight position.
    let mut positions: Vec<usize> = Vec::new();
    for p in raw_positions {
        if positions.last().copied() != Some(p) {
            positions.push(p);
        }
    }
    Some(positions)
}

/// Returns `(positions, score)` for a case-insensitive fuzzy match.
///
/// "Fuzzy" here means ordered subsequence matching over grapheme clusters,
/// plus a deterministic score to help sort candidates.
///
/// - `positions` are grapheme indices in `candidate` where query graphemes were
///   matched.
/// - `score` is higher for:
///   - consecutive matches
///   - matches at the start of the candidate
///   - matches at word boundaries
///   - shorter candidates (mild bonus)
///
/// Returns `None` if `query` cannot be matched in order.
///
/// Scoring model (intentionally simple and stable):
/// - Base: +10 per matched grapheme
/// - Consecutive bonus: +20 for each match that immediately follows the prior
///   match (by grapheme index)
/// - Start-of-string bonus: +30 if the first match is at index 0
/// - Word-boundary bonus: +15 for each match whose position is a word boundary
///   (start of string, or preceded by a non-word grapheme)
/// - Gap penalty: -1 per non-matching grapheme between successive matches
/// - Length penalty: -1 per grapheme in `candidate` (mild; favors shorter)
///
/// Word characters are ASCII letters/digits and underscore. Boundary is detected
/// between a non-word grapheme and a word grapheme.
///
/// ## Versioning
///
/// This function is an alias for the stable scorer
/// [`fuzzy_match_positions_graphemes_v1`].
pub fn fuzzy_match_positions_graphemes(query: &str, candidate: &str) -> Option<(Vec<usize>, i64)> {
    fuzzy_match_positions_graphemes_v1(query, candidate)
}

/// Latest version of the scored fuzzy matcher.
///
/// This function may change its scoring model in a future release. If you need
/// a pinned scoring model, call a versioned function such as
/// [`fuzzy_match_positions_graphemes_v1`].
pub fn fuzzy_match_positions_graphemes_latest(
    query: &str,
    candidate: &str,
) -> Option<(Vec<usize>, i64)> {
    fuzzy_match_positions_graphemes_v1(query, candidate)
}

/// Version 1 of the scored fuzzy matcher.
///
/// This function's scoring model is intended to be stable across patch/minor
/// releases. If a future scoring model is introduced, it will be exposed as a
/// new versioned function (for example, `..._v2`).
pub fn fuzzy_match_positions_graphemes_v1(
    query: &str,
    candidate: &str,
) -> Option<(Vec<usize>, i64)> {
    let qu: Vec<String> = fold_query_units(query);
    if qu.is_empty() {
        return Some((Vec::new(), 0));
    }

    let (cgs, cu) = fold_candidate_units(candidate);
    let mut q_i: usize = 0;
    let mut raw_positions: Vec<usize> = Vec::new();

    for (unit, orig_i) in cu {
        if q_i >= qu.len() {
            break;
        }
        if unit == qu[q_i] {
            raw_positions.push(orig_i);
            q_i += 1;
            if q_i == qu.len() {
                break;
            }
        }
    }

    if q_i != qu.len() {
        return None;
    }

    let mut positions: Vec<usize> = Vec::new();
    for p in raw_positions {
        if positions.last().copied() != Some(p) {
            positions.push(p);
        }
    }

    let score = score_match_positions(&cgs, &positions);
    Some((positions, score))
}

fn is_ascii_word_grapheme(g: &str) -> bool {
    // Treat a single ASCII byte as "word" if it is [A-Za-z0-9_].
    // Multi-grapheme clusters (including emoji) are treated as non-word.
    let b = g.as_bytes();
    if b.len() != 1 {
        return false;
    }
    matches!(b[0], b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
}

fn is_word_boundary(cgs: &[&str], pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    let cur_word = is_ascii_word_grapheme(cgs[pos]);
    if !cur_word {
        return false;
    }
    let prev_word = is_ascii_word_grapheme(cgs[pos - 1]);
    !prev_word
}

fn score_match_positions(cgs: &[&str], positions: &[usize]) -> i64 {
    let mut score: i64 = 0;

    // Mild length penalty favors shorter candidates.
    score -= cgs.len() as i64;

    // Base per-match scoring.
    score += 10 * positions.len() as i64;

    // Start-of-string bonus.
    if positions.first().copied() == Some(0) {
        score += 30;
    }

    // Boundary and consecutive bonuses, plus gap penalty.
    let mut prev: Option<usize> = None;
    for &p in positions {
        if is_word_boundary(cgs, p) {
            score += 15;
        }
        if let Some(pp) = prev {
            if p == pp + 1 {
                score += 20;
            } else if p > pp + 1 {
                score -= (p - (pp + 1)) as i64;
            }
        }
        prev = Some(p);
    }

    score
}
