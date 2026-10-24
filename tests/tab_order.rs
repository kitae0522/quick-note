//! Tab placement survives reopening without modifying text, cursor or note timestamps.
use quick_note::model::Selection;
use quick_note::storage::Result;
use quick_note::storage::Store;
use rusqlite::Connection;

fn ids(store: &Store) -> Result<Vec<String>> {
    Ok(store.summaries()?.into_iter().map(|note| note.id).collect())
}

#[test]
fn moves_preserve_note_data_and_survive_reopen_edits_and_new_notes() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.sqlite3");
    let mut store = Store::try_new(&path)?;
    let mut original = Vec::new();
    for text in ["First 한글", "Second 💡", "Third café"] {
        let mut note = store.create_note()?;
        note.body = text.into();
        note.selection = Selection {
            location: 2,
            length: 3,
        };
        store.save(&note)?;
        original.push(store.note(&note.id)?.expect("the saved note exists"));
    }
    let expected = vec![
        original[2].id.clone(),
        original[0].id.clone(),
        original[1].id.clone(),
    ];
    assert!(store.move_tab(&original[2].id, 0)?);
    assert_eq!(ids(&store)?, expected);
    for note in &original {
        let stored = store
            .note(&note.id)?
            .expect("moving a tab must retain its note");
        assert_eq!(stored.body, note.body);
        assert_eq!(stored.selection, note.selection);
        assert_eq!(stored.created_at, note.created_at);
        assert_eq!(stored.updated_at, note.updated_at);
    }
    drop(store);
    let mut store = Store::try_new(&path)?;
    assert_eq!(ids(&store)?, expected);
    assert_eq!(store.resume()?.id, original[2].id);
    original[0].body.push_str("\nEdited");
    store.save(&original[0])?;
    assert_eq!(ids(&store)?, expected);
    let new = store.create_note()?;
    assert_eq!(ids(&store)?, [expected, vec![new.id.clone()]].concat());
    assert!(store.move_tab(&original[2].id, usize::MAX)?);
    assert_eq!(
        ids(&store)?,
        vec![
            original[0].id.clone(),
            original[1].id.clone(),
            new.id,
            original[2].id.clone()
        ]
    );
    Ok(())
}

#[test]
fn deleting_uses_the_visual_neighbor_and_discards_stale_order_ids() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut store = Store::try_new(&directory.path().join("notes.sqlite3"))?;
    let first = store.create_note()?;
    let second = store.create_note()?;
    let third = store.create_note()?;
    store.move_tab(&third.id, 0)?;
    store.activate(&third.id)?;
    assert_eq!(store.delete_confirmed(&third.id)?.id, first.id);
    assert_eq!(ids(&store)?, vec![first.id.clone(), second.id.clone()]);
    assert_eq!(store.delete_confirmed(&second.id)?.id, first.id);
    let blank = store.delete_confirmed(&first.id)?;
    assert_eq!(ids(&store)?, vec![blank.id.clone()]);
    assert!(blank.body.is_empty());
    Ok(())
}

#[test]
fn rejected_order_write_changes_nothing_and_no_op_needs_no_write() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.sqlite3");
    let mut store = Store::try_new(&path)?;
    let first = store.create_note()?;
    let second = store.create_note()?;
    store.move_tab(&second.id, 0)?;
    Connection::open(&path)?.execute_batch(
        "CREATE TRIGGER reject_order BEFORE UPDATE ON app_state WHEN NEW.key='tabOrder' \
            BEGIN SELECT RAISE(ABORT,'order write rejected'); END;",
    )?;
    assert!(!store.move_tab(&second.id, 0)?);
    assert!(store.move_tab(&second.id, 1).is_err());
    assert_eq!(ids(&store)?, vec![second.id.clone(), first.id.clone()]);
    assert_eq!(store.resume()?.id, second.id);
    assert!(store.note(&first.id)?.is_some());
    assert!(store.move_tab("missing", 0).is_err());
    Ok(())
}
