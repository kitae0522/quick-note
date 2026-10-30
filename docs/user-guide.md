# User guide

[Back to Quick Note](../README.md)

## Writing and navigation

| Action | macOS | Windows |
| --- | --- | --- |
| New note | Cmd+N | Ctrl+N |
| Quick capture while running | Cmd+Option+N | Ctrl+Alt+N |
| New quick note | Cmd+Option+Shift+N | Ctrl+Alt+Shift+N |
| Save clipboard as note | Cmd+Option+V | Ctrl+Alt+V |
| Search all notes | Cmd+P | Ctrl+P |
| Find in current note | Cmd+F | Ctrl+F |
| Replace in current note | Cmd+Option+F | Ctrl+Alt+F |
| Next / previous match | Cmd+G / Cmd+Shift+G | Ctrl+G / Ctrl+Shift+G |
| Edit / Markdown preview | Cmd+Shift+M | Ctrl+Shift+M |
| Close tab, keeping the note | Cmd+W | Ctrl+W |
| Delete current note · confirms when nonempty | Cmd+Shift+Backspace | Ctrl+Shift+Backspace |
| Toggle Focus mode | Cmd+Shift+F | Ctrl+Shift+F |
| Close the window, keeping notes | Cmd+Shift+W | Ctrl+Shift+W |
| Quit | Cmd+Q | Ctrl+Q |
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

Closing a tab preserves its note. Search includes closed notes and reopens them.
Explicit deletion moves a note to **Recently deleted**; nonempty notes first show a
confirmation with **Cancel** selected. The search selector, File menu and Settings
open Recently deleted. Selecting a result restores its original ID, text and cursor.
There is no automatic purge. Closing or deleting the final tab creates a blank draft
in the same transaction. Closing the window or quitting preserves all notes.

Search matches titles and note text, ignoring letter case. Results show a matching
excerpt, highlight and relative edit time; opening one selects its first match.
An empty query lists the library without reading every note body. Markdown supports
headings, lists, tables, links, emphasis and code blocks. Code blocks have a Copy
button and use the theme's text color. Remote images and executable HTML are not loaded.

## Quick capture and Focus mode

While Quick Note is running, Cmd+Option+N on macOS or Ctrl+Alt+N on Windows opens a
520 × 340 capture window from another app. It resumes the current note and cursor;
Cmd/Ctrl+N starts a new note. Esc closes an open panel first, then saves and hides the
capture window. A failed save keeps the window visible with recovery actions.
**Expand** restores the previous window size and position.

The compact header includes search, new note and **Note actions**. Focus mode uses
the same quiet header in the full-size window and hides the gutter and status bar.
F6 moves between the editor and header controls in either mode. **Pin window** keeps
the window above other apps. Focus and pin settings last for the current session.

**New quick note** (Cmd+Option+Shift+N / Ctrl+Alt+Shift+N) opens an empty capture.
No database row is created until its first input. Esc on an unused capture returns
to the previous note and cursor. The compact header's plus button uses this same path.
**Save clipboard as note** (Cmd+Option+V / Ctrl+Alt+V) saves copied text as a closed
note without switching your current app or note. On macOS, a brief checkmark in the
status menu confirms the save. Empty or non-text clipboards show a notice; a failed
write keeps the captured text in a recovery copy and opens the editor for retry.

The macOS status menu offers new capture, resume, clipboard capture and the five
most recently saved notes, including closed notes. With that menu available or a
global shortcut registered, closing the window hides Quick Note. Cmd/Ctrl+Q quits.
On Windows, disabling every global capture binding restores quit-on-close behavior.
Shortcut changes update registration without restarting the app; failed reloads
retain the previous working bindings.

**Settings → Open at login** is optional and available when running Quick Note.app
on macOS. It reads the system's actual registration state. If macOS requires
approval, follow **Open Login Items settings**; the checkbox does not claim the app
is enabled until approval is granted. Turning it off unregisters the app. Running
an unbundled development executable does not offer login registration.

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

Schema v3 adds tab visibility and recoverable deletion without changing note IDs,
UTF-8 text or UTF-16 cursors. Before migrating an existing v1 or v2 database,
Quick Note writes `notes.v1-backup.sqlite3` or `notes.v2-backup.sqlite3`, respectively.
The migration runs in one transaction. It refuses corrupt databases and newer
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
