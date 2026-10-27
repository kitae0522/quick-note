# User guide

[Back to Quick Note](../README.md)

## Writing and navigation

| Action | macOS | Windows |
| --- | --- | --- |
| New note | Cmd+N | Ctrl+N |
| Search all notes | Cmd+P | Ctrl+P |
| Find in current note | Cmd+F | Ctrl+F |
| Replace in current note | Cmd+Option+F | Ctrl+Alt+F |
| Next / previous match | Cmd+G / Cmd+Shift+G | Ctrl+G / Ctrl+Shift+G |
| Edit / Markdown preview | Cmd+Shift+M | Ctrl+Shift+M |
| Delete current note · confirms when nonempty | Cmd+W | Ctrl+W |
| Close the window, keeping notes | Cmd+Shift+W | Ctrl+Shift+W |
| Next / previous note | Ctrl+Tab / Ctrl+Shift+Tab | Ctrl+Tab / Ctrl+Shift+Tab |
| Move current tab left / right | Ctrl+Shift+PageUp / PageDown | Ctrl+Shift+PageUp / PageDown |
| Settings | Cmd+, | Ctrl+, |
| Settings (alternate) | Cmd+Shift+P | Ctrl+Shift+P |
| Keyboard shortcuts | Cmd+/ | Ctrl+/ |
| Open note 1–8 / last note | Cmd+1–8 / Cmd+9 | Ctrl+1–8 / Ctrl+9 |
| Focus note / tabs | F6 or Shift+F6 | F6 or Shift+F6 |
| Indent / unindent while editing | Tab / Shift+Tab | Tab / Shift+Tab |
| Close a panel / return to writing | Esc | Esc |
| Retry saving | Cmd+S | Ctrl+S |
| Export note | Cmd+Shift+E | Ctrl+Shift+E |
| Reload theme | Cmd+Shift+R | Ctrl+Shift+R |
| Switch Light / Dark | Cmd+Shift+L | Ctrl+Shift+L |
| Import / export theme | Cmd+Option+I / E | Ctrl+Alt+I / E |
| Open themes folder | Cmd+Option+O | Ctrl+Alt+O |
| Minimize / maximize window | Cmd+M / F11 | Ctrl+M / F11 |

These are the defaults. Use **Settings → Keyboard shortcuts → Open shortcuts folder**
to edit UTF-8 `shortcut/bindings.json`. Reload there or return to the app to apply it.
See [shortcut configuration](../shortcut/README.md) for action IDs, aliases and examples.

Find uses literal text, with an optional **Match case** toggle. Enter and Shift+Enter
navigate matches from the find field; F3 and Shift+F3 work too. **Replace** and
**Replace all** affect only the current note. Every replacement saves immediately;
replacing all is one undo step. Esc closes the panel and returns to writing.
Case-insensitive search uses Unicode case folding in both search panels. Greek
sigma forms and `Straße` / `STRASSE` match. Replacements retain original UTF-8
ranges and never replace only part of an expanded character. Notes are not normalized.

On macOS, the system menu bar provides **Quick Note, File, Edit, View, Window and Help**.
It follows the current language and shortcut configuration. Edit actions target the
focused text field; Quit uses the same failed-save protection as closing the window.
Menu commands wait for pending IME commits before changing notes. The native menu is
installed only in native Mac windows; Windows keeps its existing window controls.

In the source editor, typing `(`, `[` or `{` adds its closing bracket and places
the cursor inside. With text selected, it wraps the selection instead. Typing an
existing closing bracket moves past it; Backspace inside an empty pair removes
both brackets. Undo and redo treat each pair or wrap as one edit. Pasted text,
Korean composition and search queries remain literal.

On macOS, Cmd+Backspace commits any active composition and deletes back to the
beginning of the current line in one press. Undo restores the deleted text,
including its last composed syllable. The same binding works in search fields.

In the editor, Tab inserts four spaces and Shift+Tab removes one indentation level.
Use F6 or Shift+F6 to focus the selected note tab, then Left, Right, Home or End.
At small widths, search collapses to an icon and overflow arrows yield space to the active tab.
Use horizontal scrolling or the note shortcuts to reach other tabs.
Drag a tab title to rearrange notes. Hold it at either edge to scroll to hidden tabs;
release inside the strip to save the order. Escape or dropping outside cancels the move.
The order survives restarts, and moving a tab keeps the current note and caret intact.
Drag empty header space to move the window. The editor opens with a caret and no
placeholder text. Returning from a native window drag releases the app's pointer state.
Tab and Shift+Tab move through the settings controls. Enter or Space
activates a control, and Esc restores the previous focus. Search keeps Enter with
an active IME composition before using it to open a result.

Buttons, switches and opening panels use brief 120 ms feedback. **Settings → Reduce
motion** makes transitions instant. The desktop's reduced-motion preference takes
priority and refreshes when the window regains focus. The first editor frame appears
without an entrance animation, and theme changes apply immediately.

Empty notes, including whitespace-only notes, delete immediately. Notes with content
show a confirmation with **Cancel** selected. Deleting the final note creates a fresh
blank draft in the same transaction.
Closing the window or quitting preserves all notes.

Search matches titles and note text, ignoring letter case. Markdown supports
headings, lists, tables, links, emphasis and code blocks. Code blocks have a Copy
button and use the theme's text color. Remote images and executable HTML are not loaded.

## Storage and recovery

- macOS: `~/Library/Application Support/Quick Note/`
- Windows: `%LOCALAPPDATA%\Quick Note\`

`notes.sqlite3` stores notes and UTF-16 selection offsets for compatibility with the
original macOS prototype. The database text encoding is **UTF-8**. JSON themes,
recovery files and Markdown/text exports are UTF-8 too.

Edits use SQLite WAL with `synchronous=FULL`; macOS also enables `fullfsync`.
A failed database write keeps the text in the editor, blocks switching and closing,
and attempts an atomic `recovery.json` copy. The next launch restores an uncommitted
copy against its saved revision, without replacing a later successful save.
If both writes fail, keep the window open and use **Export text…**. A second app
cannot edit the same folder concurrently.

The original schema is migrated without changing note IDs or text. Before migration,
Quick Note writes `notes.v1-backup.sqlite3`. It refuses corrupt databases and newer
schema versions rather than replacing them. Quit the app and copy the entire data
folder to make your own backup.

Use isolated data for experiments:

```sh
cargo run --release -- --data-directory /tmp/quick-note-sandbox
```

## Language

Choose **Settings → Language → English / 한국어**. English is the default, and the
choice is saved. UI messages, accessibility labels and errors live in
[`i18n/en.json`](../i18n/en.json) and [`i18n/ko.json`](../i18n/ko.json). Notes and their
user-written titles stay unchanged.

Edit source JSON for the next build, or copy complete catalogs to the note data
folder's `i18n/` directory and select the language again to apply edits without
rebuilding. See [catalog format and override locations](../i18n/README.md).

## Settings

The header’s **Open settings** button and Cmd/Ctrl+, open a settings page inside
the app. Preferences are grouped by language, appearance, editor and theme files.
**Back to note**, Escape or Cmd/Ctrl+W return to the same document and cursor.
Cmd/Ctrl+W closes only the settings page while it is active.

## Markdown

Markdown support is on by default. Toggle **Settings → Markdown** with a click,
or Tab to the switch and press Space or Enter. The preference survives restarts.
Turning it off returns Preview to the source editor and removes Preview controls
and its shortcut. Notes, Markdown syntax and cursor positions remain intact.

Preview caches are created on the first preview and dropped when Markdown is off.
The source editor already skips Markdown parsing, so OFF does not promise a large
idle memory or CPU reduction. Window and font rendering are shared by both modes.

## Themes

Open **Settings → Export theme…**, edit the JSON, then **Import theme…**.
You can adjust colors, editor fonts, text size, line spacing, padding and header height.
**Choose fonts…** searches all installed families. Check fonts to add or remove them;
use ↑/↓ priority buttons to reorder the selected stack. Search supports ↑/↓ and Enter,
and Tab reaches every action. Choose 1–8 families, tried in order before bundled fallbacks.
Source labels show which fonts are built in or installed. A mixed-script preview
shows the active stack, and selection limits explain disabled controls.
The stack persists across theme changes and restarts. **Use theme font** restores the
active theme's stack. Shared JSON accepts `fontFamily` and optional `fontFallbacks`.
The font picker starts with SF Pro, Pretendard, Noto Sans CJK, Menlo and Consolas.
UI text uses installed SF Pro with bundled Pretendard and Noto Sans CJK fallbacks.
Variable fonts use the selected point size for optical spacing, keeping small text
out of the font's display-size spacing. Explicit variation coordinates retain priority.
The default editor uses Menlo on macOS and Consolas on Windows. Its grid gives Latin
letters, digits and spaces one cell, and Hangul/CJK and emoji clusters two cells,
without stretching their outlines. Combining accents share their base cell.
Selecting a proportional local font keeps its natural widths.
Imported themes reload when the app regains focus, or with the reload shortcut.
Invalid changes leave the current design and notes intact.

[Theme format](themes.md) · [JSON Schema](theme.schema.json) ·
[Helium example](../assets/themes/helium.json) · [Font licenses and sources](../assets/licenses/README.md)
