//! Route AppKit callbacks once per native key batch, retaining pending unmarked commits.
#[derive(Debug, Default)]
pub(super) struct ImeCommit {
    pending: bool,
    committed: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum InsertAction {
    Commit,
    ForwardKey,
    Ignore,
}

pub(super) fn forward_key(
    ime_handled: bool,
    input_method_command: bool,
    application_command: bool,
) -> bool {
    // Completing a composition must not consume the editor's line-delete command.
    !ime_handled || input_method_command || application_command
}

impl ImeCommit {
    pub fn begin_key(&mut self) {
        self.committed = None;
    }

    pub fn preedit(&mut self, text: &str) {
        // Clearing marked text can precede insertText; retain the pending commit.
        self.pending |= !text.is_empty();
    }

    pub fn pending(&self) -> bool {
        self.pending
    }

    pub fn insert(&mut self, text: &str, enabled: bool) -> InsertAction {
        if !enabled || text.is_empty() || text.chars().any(char::is_control) {
            return InsertAction::Ignore;
        }
        if self.pending {
            self.pending = false;
            self.committed = Some(text.to_owned());
            InsertAction::Commit
        } else if self
            .committed
            .as_deref()
            .is_some_and(|previous| previous != text)
        {
            // A trailing space/punctuation is a new key payload, not another composition.
            InsertAction::ForwardKey
        } else {
            InsertAction::Ignore
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::{ImeCommit, InsertAction};

    #[test]
    fn a_single_korean_space_callback_batch_cannot_commit_twice() {
        let mut state = ImeCommit::default();
        state.preedit(" ");
        assert_eq!(state.insert(" ", true), InsertAction::Commit);
        assert_eq!(state.insert(" ", true), InsertAction::Ignore);
    }

    #[test]
    fn a_triggering_space_after_a_syllable_still_reaches_keyboard_input() {
        let mut state = ImeCommit::default();
        state.preedit("한");
        assert_eq!(state.insert("한", true), InsertAction::Commit);
        assert_eq!(state.insert(" ", true), InsertAction::ForwardKey);
    }

    #[test]
    fn an_unmark_before_commit_does_not_drop_the_syllable() {
        let mut state = ImeCommit::default();
        state.preedit("한");
        state.preedit("");
        assert!(state.pending());
        assert_eq!(state.insert("한", true), InsertAction::Commit);
    }

    #[test]
    fn repeated_syllables_and_spaces_in_different_keys_are_preserved() {
        let mut state = ImeCommit::default();
        for text in ["한", "한", " ", " "] {
            state.begin_key();
            state.preedit(text);
            assert_eq!(state.insert(text, true), InsertAction::Commit);
        }
    }

    #[test]
    fn a_new_preedit_in_the_same_batch_is_a_distinct_commit() {
        let mut state = ImeCommit::default();
        for _ in 0..2 {
            state.preedit("가");
            assert_eq!(state.insert("가", true), InsertAction::Commit);
        }
    }

    #[test]
    fn ordinary_typing_and_control_characters_keep_the_existing_key_path() {
        let mut state = ImeCommit::default();
        assert_eq!(state.insert("a", true), InsertAction::Ignore);
        state.preedit("한");
        assert_eq!(state.insert("\n", true), InsertAction::Ignore);
        assert!(state.pending());
        assert_eq!(state.insert("한", true), InsertAction::Commit);
    }

    #[test]
    fn resetting_focus_or_input_source_drops_stale_pending_composition() {
        let mut state = ImeCommit::default();
        state.preedit("한");
        state.reset();
        assert!(!state.pending());
        assert_eq!(state.insert("한", true), InsertAction::Ignore);
    }
}
