use quick_note::model::Selection;
use quick_note::model::find_matches;

#[test]
fn unicode_matches_keep_byte_character_and_utf16_boundaries() {
    let body = "💡 한글 CAFÉ café";
    let found = find_matches(body, "café", false);
    assert_eq!(found.len(), 2);
    for (found, expected) in found.iter().zip(["CAFÉ", "café"]) {
        assert_eq!(&body[found.bytes.clone()], expected);
        let selection =
            Selection::from_char_range(body, found.characters.start, found.characters.end);
        assert_eq!(
            selection.char_range(body),
            (found.characters.start, found.characters.end)
        );
    }
    assert_eq!(find_matches(body, "café", true).len(), 1);
    assert_eq!(find_matches(body, "한글", false)[0].characters, 2..4);
}

#[test]
fn literal_queries_are_nonoverlapping_and_never_split_case_expansions() {
    assert!(find_matches("abc", "", false).is_empty());
    assert!(find_matches("abc", ".*", false).is_empty());
    assert_eq!(find_matches("a.$a.$", "a.$", false).len(), 2);
    assert_eq!(find_matches("aaaaa", "aa", false).len(), 2);
    assert!(find_matches("İ", "i", false).is_empty());
    assert_eq!(find_matches("İ", "i\u{307}", false)[0].bytes, 0..2);
}

#[test]
fn greek_sigma_matches_the_same_word_in_every_case() {
    let body = "💡 ΟΣ ος οσ";
    for query in ["ΟΣ", "ος", "οσ"] {
        let found = find_matches(body, query, false);
        assert_eq!(found.len(), 3, "{query}");
        for (found, expected) in found.iter().zip(["ΟΣ", "ος", "οσ"]) {
            assert_eq!(&body[found.bytes.clone()], expected);
        }
    }
    assert_eq!(find_matches(body, "ΟΣ", true).len(), 1);
}

#[test]
fn case_folding_preserves_original_ranges_and_skips_partial_expansions() {
    let body = "💡 Straße STRASSE";
    let found = find_matches(body, "strasse", false);
    assert_eq!(found.len(), 2);
    for (found, expected) in found.iter().zip(["Straße", "STRASSE"]) {
        assert_eq!(&body[found.bytes.clone()], expected);
        let selection =
            Selection::from_char_range(body, found.characters.start, found.characters.end);
        assert_eq!(
            selection.char_range(body),
            (found.characters.start, found.characters.end)
        );
    }
    assert!(find_matches("ß", "s", false).is_empty());
    assert!(find_matches("ﬃ", "fi", false).is_empty());
    assert_eq!(find_matches("ﬃfif", "fif", false)[0].bytes, 3..6);
}

#[test]
fn global_search_uses_the_same_case_rules_without_rewriting_notes()
-> quick_note::storage::Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = quick_note::app::QuickNoteApp::try_new(directory.path().to_owned())?;
    let body = "ΟΣ Straße 한글 💡";
    app.note.body = body.into();
    assert!(app.persist());
    for query in ["ΟΣ", "ος", "οσ", "STRASSE"] {
        app.header.query = query.into();
        app.refresh_search()?;
        assert_eq!(app.search_results().len(), 1, "{query}");
        assert_eq!(app.search_results()[0].id, app.note.id);
    }
    assert_eq!(app.store.resume()?.body, body);
    Ok(())
}
