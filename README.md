<img src="assets/icon.svg" width="48" height="48" alt="Quick Note">

# Quick Note

A small, local-first notebook for macOS and Windows. Open it and write.
Built in Rust with egui, native GPU rendering and SQLite. No webview or account.

**v0.2.0-alpha.1** is the first public alpha.
[Downloads and release notes](https://github.com/kitae0522/quick-note/releases/tag/v0.2.0-alpha.1).
The macOS download is for Apple Silicon on macOS 14 or newer. The Windows download
is experimental and targets x86-64. Both packages are unsigned by a verified publisher;
the macOS app has local ad-hoc signing and is not notarized.
Keep a backup of your note data while trying the alpha. Physical Korean IME,
VoiceOver and actual Windows window/input behavior still need manual verification.

Quick Note's own code is [MIT licensed](LICENSE). Vendored libraries, fonts and
icons retain their respective licenses. Packages include the full notices in `Licenses`.

- An editable draft on first launch; the last note and cursor on return.
- Every edit commits immediately. Closing a saved note needs no save prompt.
- All notes stay in one compact, horizontally scrolling tab strip. Drag tabs to reorder.
- Line numbers, cursor location, full-text search and Markdown preview.
- Click anywhere in the editing surface, including blank space and margins, to write.
- English and Korean controls, Unicode text, explicit UTF-8 storage and exports.
- Shareable JSON themes, with Helium and Graphite included.

## Run and build

Use Rust 1.95 or newer. The lockfile pins dependencies.

```sh
cargo run --release
```

On macOS 14 or newer, build a relocatable app bundle:

```sh
./scripts/build-app.sh
open "dist/Quick Note.app"
```

The bundle uses local ad-hoc signing. Developer ID signing, notarization and an
installer are not included. Icons are checked-in assets copied into the bundle
during packaging. Cargo builds the application.

On Windows, install Rust and the C++ Build Tools, then run:

```powershell
.\scripts\build-windows.ps1
```

This produces `dist/windows/Quick Note.exe` and a portable ZIP. To cross-build an
x86-64 Windows executable on a Mac with MinGW-w64 installed:

```sh
rustup target add x86_64-pc-windows-gnu
./scripts/build-windows.sh
```

The Windows target has been cross-compiled. Actual Windows input, window behavior,
accessibility and rendering still need verification on a Windows machine.

## Use

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
See [shortcut configuration](shortcut/README.md) for action IDs, aliases and examples.

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
[`i18n/en.json`](i18n/en.json) and [`i18n/ko.json`](i18n/ko.json). Notes and their
user-written titles stay unchanged.

Edit source JSON for the next build, or copy complete catalogs to the note data
folder's `i18n/` directory and select the language again to apply edits without
rebuilding. See [catalog format and override locations](i18n/README.md).

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

[Theme format](docs/themes.md) · [JSON Schema](docs/theme.schema.json) ·
[Helium example](assets/themes/helium.json) · [Font licenses and sources](assets/licenses/README.md)

## Verification and speed

Authored application, test and tool files stay within 250 lines. Upstream files
under `vendor/` retain their original structure. The size guard runs with the
standard checks. See [Rust conventions](docs/rust-style.md).

[GitHub Actions](https://github.com/kitae0522/quick-note/actions/workflows/ci.yml)
runs the standard checks and builds an Apple Silicon app on macOS. Windows CI
checks all targets, tests storage/search/shortcut/input-policy logic and builds
the MSVC package. These automated checks do not verify physical input or screen readers.

```sh
./scripts/check.sh
python3 scripts/check-ui.py        # a GUI session on macOS is required
python3 scripts/check-menu.py      # actual AppKit menus and native key equivalents
python3 scripts/check-input.py     # canvas, keyboard and Korean IME event replays
python3 scripts/check-find.py      # current-note search, literal replacements, undo and restart
python3 scripts/check-appkit-input.py # native AppKit dispatch, drag separation and input after drag
python3 scripts/check-line-delete.py # single-press line deletion, composed-text undo and recovery
python3 scripts/check-pairs.py     # bracket pairs, wrapping, undo and restart recovery
python3 scripts/check-tab-order.py # tab drags, edge scrolling, order recovery and write failures
python3 scripts/check-localization.py # language selection, restart and editable JSON
python3 scripts/check-fonts.py     # installed-font picker, restart and missing-font recovery
python3 scripts/check-font-layout.py # candidate resize focus, short-window navigation and Markdown grid
python3 scripts/check-markdown.py  # default ON, toggle, restart and source preservation
python3 scripts/check-settings.py  # page navigation, return focus and safe Cmd/Ctrl+W
cargo run --example header_stress  # real header fixtures; writes an HTML report
cargo run --example interface_gallery # light/dark, narrow, search, settings and confirmation GPU renders
cargo run --example interface_motion  # 60 Hz GPU samples, including reduced motion and settled repaints
cargo run --example window_activation -- active /tmp/quick-note-activation.json # macOS first-show activation
```

Tests cover migration, UTF-8, cursor restoration, save failures, confirmed deletion,
keyboard navigation, search, Markdown, contrast and narrow headers. The native
probe additionally kills its own process and reopens the same isolated database.
Screenshots and measured results are written under `.artifacts/`.
The offscreen examples render production widgets with fixture notes. They exclude
AppKit traffic lights, physical input and screen-reader speech. Motion samples use
simulated UI time; they are not an OS screen recording. Tests also inspect names,
roles, focus pixels, 200% zoom and keyboard input during a running transition.

The macOS input backend carries focused fixes for pending Korean commits, duplicate
native callbacks, stale composition after focus changes and first-key fallback.
[Patch provenance and scope](vendor/winit/QUICK_NOTE_PATCHES.md) explains the upstream
work and the 16 backend-policy regression tests. Event replays and policy tests do
not verify the physical macOS input-method process.

An initially hidden macOS window requests activation once when the renderer reveals
it. This preserves the launch request across GPU initialization; inactive windows
and the event loop's activation opt-out remain respected. The `window_activation`
example also accepts `inactive` and `no-activate` to check those opt-outs.

**200 ms is a performance budget, not a guarantee.** Native action checks have
completed within the budget, including synchronous saves and GPU readback. Startup
still sometimes exceeds it. `scripts/check-ui.py` reports startup and action budgets
separately; functional success does not imply a passing startup budget.

Saved font definitions prepare on a worker while AppKit/window/GPU startup runs.
The first UI pass uses the prepared native fonts when ready; slow or missing fonts
retain the usable bundled fallback and existing error notice. Startup does not wait
for the font worker, and explicit font changes override its prepared selection.

To measure startup without the interaction driver's focus helpers:

```sh
python3 scripts/measure-startup.py --output .artifacts/startup-current --trials 10
```

Keep the Mac unlocked. The probe reports the first scripted UTF-8 text accepted and
committed with window/editor focus, a focused native-font GPU capture, and process
launch to the observed input-ready report separately. The last metric includes the
loader and reporting overhead. Save a release `startup-probe` and pass `--binary`
to compare builds; do not run compilers or other GUI drivers during measurements.
Fresh data directories and restored stores are separate cases; neither flushes OS
or driver caches. First Gatekeeper verification and physical Dock clicks are untested.

The interaction driver's timing starts at Rust `main` or scripted input delivery and
ends at native GPU framebuffer readback with keyboard focus. OS/library caches are
not flushed, and loader time before `main` is excluded. Scripted events do not verify physical Korean
IME composition, VoiceOver, power-loss durability or Windows runtime behavior.

To measure CPU and memory on a Mac with isolated fixture notes:

```sh
cargo build --release --features diagnostics --bins
python3 scripts/measure-resources.py --seconds 20
python3 scripts/measure-markdown.py
python3 scripts/measure-performance.py --output .artifacts/performance-current --trials 2
```

Idle measurements use the production executable. Typing and search use the same
production UI with scripted input requested at up to 8 Hz, without screenshot
capture or command polling. Results separate physical footprint from RSS and define
one full CPU core as 100%. JSON samples and memory-region attribution are saved
under `.artifacts/`.

The performance probe covers 52 notes, 1,000 notes and a 244 KB UTF-8 Markdown note.
Use `--binaries <directory>` with saved release executables to repeat the same cases
before and after a change. Core UI timings exclude GPU presentation; native resource
samples measure the process separately. Compare foreground samples with foreground
samples: macOS can reclaim graphics memory when an app moves into the background.

The CPU sampler converts Mach ticks using `mach_timebase_info`; earlier samples
that treated these ticks as nanoseconds understated CPU usage on Apple Silicon.
`scripts/check-resource-usage.py` checks the conversion against `time.process_time`
and runs as part of the Mac checks. See Apple's [CPU counter test](https://github.com/apple-oss-distributions/xnu/blob/main/tests/recount/recount_tests.c).

Tabs, search results and Markdown render the visible portion of their scroll areas.
Search normalization, logical-line cursor indices and font filtering are reused
until their inputs change. Markdown still lays out the full document on its first
view or after invalidation; subsequent redraws use the viewport cache. Immediate
SQLite saves retain `synchronous=FULL` and macOS `fullfsync`.

The Markdown comparison runs the same release UI at 8 Hz with physical input
discarded. It compares ON/OFF before Preview and after returning from Preview,
using two 5-second samples per setting after warmup. This is a controlled redraw
workload, not idle CPU usage. Check foreground samples and run-to-run variation
before attributing small differences to the setting.
