//! Use the native routing policy with the real editor: one physical command must deliver one delete.
use crate::ime_commit;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::Harness;
use quick_note::app::QuickNoteApp;
use quick_note::model::Selection;
use quick_note::storage::Result;

#[test]
fn an_ime_commit_and_command_backspace_in_one_native_key_delete_the_line() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    app.note.body = "Keep this\nPrefix ".into();
    app.note.selection = Selection {
        location: 17,
        length: 0,
    };
    assert!(app.persist());
    let mut harness = Harness::builder()
        .with_size([640.0, 400.0])
        .build_eframe(move |creation| {
            app.configure(&creation.egui_ctx);
            app
        });
    harness.run();
    let mut commit = ime_commit::ImeCommit::default();
    commit.preedit("한");
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "한".into(),
        active_range_chars: None,
    }));
    harness.run();
    assert_eq!(harness.state().note.body, "Keep this\nPrefix 한");
    commit.begin_key();
    assert_eq!(commit.insert("한", true), ime_commit::InsertAction::Commit);
    harness.event(Event::Ime(ImeEvent::Commit("한".into())));
    if ime_commit::forward_key(true, false, true) {
        harness.key_press_modifiers(
            Modifiers {
                command: true,
                mac_cmd: true,
                ..Modifiers::NONE
            },
            Key::Backspace,
        );
    }
    harness.run();
    assert_eq!(harness.state().note.body, "Keep this\n");
    assert_eq!(
        harness.state().note.selection,
        Selection {
            location: 10,
            length: 0
        }
    );
    assert!(!harness.state().dirty);
    Ok(())
}

#[test]
fn plain_ime_keys_stay_consumed_and_explicit_appkit_commands_stay_forwarded() {
    assert!(!ime_commit::forward_key(true, false, false));
    assert!(ime_commit::forward_key(true, true, false));
    assert!(ime_commit::forward_key(false, false, false));
}
