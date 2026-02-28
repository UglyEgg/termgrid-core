use pretty_assertions::assert_eq;
use unicode_segmentation::UnicodeSegmentation;

use termgrid_core::{
    fuzzy_match_positions_graphemes, fuzzy_match_positions_graphemes_latest,
    fuzzy_match_positions_graphemes_v1, match_positions_graphemes,
};

#[test]
fn match_positions_graphemes_matches_in_order_and_skips_non_matches() {
    let cand = "a🙂b";
    let pos = match_positions_graphemes("ab", cand).expect("match");
    assert_eq!(pos, vec![0, 2]);
}

#[test]
fn match_positions_graphemes_is_case_insensitive() {
    let cand = "aB";
    let pos = match_positions_graphemes("Ab", cand).expect("match");
    assert_eq!(pos, vec![0, 1]);
}

#[test]
fn match_positions_graphemes_returns_none_on_failure() {
    assert_eq!(match_positions_graphemes("zzz", "abc"), None);
}

#[test]
fn match_positions_graphemes_empty_query_matches_everything() {
    let pos = match_positions_graphemes("", "anything").expect("match");
    assert_eq!(pos, Vec::<usize>::new());
}

#[test]
fn fuzzy_match_positions_graphemes_returns_positions_and_score() {
    let (pos, score) = fuzzy_match_positions_graphemes_v1("ab", "a🙂b").expect("match");
    assert_eq!(pos, vec![0, 2]);
    // Score is deterministic and should be positive for a short successful match.
    assert!(score > 0);
}

#[test]
fn fuzzy_match_positions_graphemes_prefers_consecutive_matches() {
    let (_pos1, s1) = fuzzy_match_positions_graphemes_v1("abc", "a b c").expect("match");
    let (_pos2, s2) = fuzzy_match_positions_graphemes_v1("abc", "abc").expect("match");
    assert!(s2 > s1);
}

#[test]
fn fuzzy_match_positions_graphemes_prefers_word_boundaries() {
    let (_pos1, s1) = fuzzy_match_positions_graphemes_v1("cat", "concatenate").expect("match");
    let (_pos2, s2) = fuzzy_match_positions_graphemes_v1("cat", "catapult").expect("match");
    assert!(s2 > s1);
}

#[test]
fn fuzzy_match_positions_graphemes_is_stable_alias_of_v1() {
    let a = fuzzy_match_positions_graphemes_v1("abc", "a b c").expect("match");
    let b = fuzzy_match_positions_graphemes("abc", "a b c").expect("match");
    assert_eq!(a, b);
}

#[test]
fn fuzzy_match_positions_graphemes_latest_defaults_to_v1_today() {
    let a = fuzzy_match_positions_graphemes_v1("abc", "a b c").expect("match");
    let b = fuzzy_match_positions_graphemes_latest("abc", "a b c").expect("match");
    assert_eq!(a, b);
}

#[test]
fn match_and_fuzzy_positions_are_grapheme_indices_for_combining_sequences() {
    // "e" + combining acute accent is a single grapheme cluster.
    let cand = "e\u{0301}x";

    let pos = match_positions_graphemes("e\u{0301}", cand).expect("match");
    assert_eq!(pos, vec![0]);

    let (pos2, _score) = fuzzy_match_positions_graphemes_v1("e\u{0301}", cand).expect("match");
    assert_eq!(pos2, vec![0]);
}

#[test]
fn match_and_fuzzy_positions_are_grapheme_indices_for_emoji() {
    let cand = "a🙂b";
    let g: Vec<&str> = cand.graphemes(true).collect();
    assert_eq!(g, vec!["a", "🙂", "b"]);

    let pos = match_positions_graphemes("🙂", cand).expect("match");
    assert_eq!(pos, vec![1]);
    assert_eq!(g[pos[0]], "🙂");

    let (pos2, _score) = fuzzy_match_positions_graphemes_v1("🙂", cand).expect("match");
    assert_eq!(pos2, vec![1]);
    assert_eq!(g[pos2[0]], "🙂");
}

#[test]
fn match_and_fuzzy_positions_are_grapheme_indices_for_emoji_with_variation_selector() {
    // U+2665 HEART SUIT + U+FE0F VARIATION SELECTOR-16 should be a single grapheme cluster.
    let cand = "a♥️b";
    let g: Vec<&str> = cand.graphemes(true).collect();
    assert_eq!(g, vec!["a", "♥️", "b"]);

    let pos = match_positions_graphemes("♥️", cand).expect("match");
    assert_eq!(pos, vec![1]);
    assert_eq!(g[pos[0]], "♥️");

    let (pos2, _score) = fuzzy_match_positions_graphemes_v1("♥️", cand).expect("match");
    assert_eq!(pos2, vec![1]);
    assert_eq!(g[pos2[0]], "♥️");
}

#[test]
fn casefold_allows_multi_unit_folds_while_preserving_grapheme_positions() {
    // German sharp s folds to "ss".
    let cand = "ß";
    let pos = match_positions_graphemes("ss", cand).expect("match");
    assert_eq!(pos, vec![0]);

    let (pos2, _score) = fuzzy_match_positions_graphemes_v1("ss", cand).expect("match");
    assert_eq!(pos2, vec![0]);
}

#[test]
fn casefold_is_nfkc_and_matches_ligatures() {
    // The "ﬀ" ligature should fold/normalize to "ff" under NFKC.
    let cand = "ﬀ";
    let pos = match_positions_graphemes("ff", cand).expect("match");
    assert_eq!(pos, vec![0]);

    let (pos2, _score) = fuzzy_match_positions_graphemes_v1("ff", cand).expect("match");
    assert_eq!(pos2, vec![0]);
}
