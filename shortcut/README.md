# Keyboard shortcuts

Open **Settings → Keyboard shortcuts → Open shortcuts folder**. Edit `bindings.json`
there as UTF-8 JSON, then choose **Reload shortcuts** or return to the app from another
window. The app creates this file on first launch and never overwrites an existing file.

- macOS: `~/Library/Application Support/Quick Note/shortcut/bindings.json`
- Windows: `%LOCALAPPDATA%\Quick Note\shortcut\bindings.json`

The file in this source directory is the bundled default. A data-folder file can contain
only the actions you want to change; omitted actions keep the defaults. An empty array
disables a binding. Each supplied array replaces all aliases for that action.

```json
{
  "schemaVersion": 1,
  "bindings": {
    "new_note": ["Primary+Shift+N"],
    "search_notes": ["Primary+K"],
    "toggle_preview": []
  }
}
```

`Primary` (also `CmdOrCtrl`) means Cmd on macOS and Ctrl on Windows. Other modifiers
are `Ctrl`, `Shift`, `Alt` (also `Option`), and macOS `Cmd` (also `Command`). Separate
modifiers and the final key with `+`. Key names include letters, digits, `Comma`,
`Slash`, `OpenBracket`, `CloseBracket`, `Space`, `Enter`, `Tab`, `PageUp`, `PageDown`, and `F1`–`F35`.
Modifier names are case-insensitive; use the displayed spelling for named keys.

Use Primary, Ctrl, or Alt for ordinary keys. Unmodified and Shift-only combinations
are allowed for F keys. Plain Tab, Shift+Tab, letters, arrows and other editing keys
stay with text editing and control navigation. Standard copy, paste, undo, redo and
indentation use the editor's native bindings. These files configure application actions.
OS-reserved shortcuts may be handled before the app receives them.

All action IDs and their defaults are listed in [bindings.json](bindings.json).
`find_in_note`, `replace_in_note`, `find_next` and `find_previous` control the active
note's find panel. `search_notes` keeps searching across notes. The former default
Cmd/Ctrl+F alias for `search_notes` moves to `find_in_note` in memory; existing JSON
is never rewritten. Custom assignments retain priority over newly added default keys.
`move_tab_left` and `move_tab_right` reorder the current tab; `previous_note` and
`next_note` switch notes in that order. The default reorder keys are Ctrl+Shift+PageUp
and Ctrl+Shift+PageDown on both platforms.
`close_tab` now uses Primary+W and retains the note; `delete_note` uses
Primary+Shift+Backspace. Exact legacy defaults for deletion and window closing
migrate in memory, without rewriting the existing JSON. `quit_app` is separate from
`close_window`. Explicit custom assignments keep priority over new defaults.

`quick_capture` registers a system-wide shortcut while the app runs, defaulting to
Primary+Alt+N. `new_capture` opens an empty capture with Primary+Alt+Shift+N;
`capture_clipboard` saves copied text with Primary+Alt+V. An empty array disables
that action's global binding. Registration conflicts show an error;
a failed registration during reload retains the previous shortcuts.
`focus_mode` defaults to Primary+Shift+F. `recently_deleted` and `pin_window` have
no default keys and can be assigned here.

There can be at most eight combinations per action and 64 KB per file. Conflicts are
checked after resolving platform aliases, so Ctrl and Primary are the same on Windows.
Unknown actions, keys, duplicate assignments and malformed files are rejected together.
Help and preview tooltips display the current bindings, including disabled actions.

A failed reload retains the entire last valid binding set and shows the reason.
On startup, an invalid file leaves defaults active and shows a notice. Your file and
notes stay intact. Fix the JSON and reload, or remove the file to recreate defaults.
