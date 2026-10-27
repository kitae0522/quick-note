<a id="readme-top"></a>

<div align="center">
  <img src="assets/icon-256.png" alt="Quick Note icon" width="80" height="80">
  <h1>Quick Note</h1>
  <p>A small, local-first notebook. Open it and write.</p>
  <p><strong>English</strong> &middot; <a href="README.ko.md" lang="ko">한국어</a></p>
  <p>
    <a href="https://github.com/kitae0522/quick-note/releases/tag/v0.2.0-alpha.1"><strong>Download the alpha</strong></a>
    &middot;
    <a href="docs/user-guide.md">User guide</a>
    &middot;
    <a href="https://github.com/kitae0522/quick-note/issues/new">Report an issue</a>
  </p>
  <p>
    <a href="https://github.com/kitae0522/quick-note/actions/workflows/ci.yml"><img src="https://github.com/kitae0522/quick-note/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI status on main"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT license"></a>
  </p>
</div>

<details>
  <summary>Contents</summary>

- [About](#about)
- [Getting started](#getting-started)
- [Usage](#usage)
- [Customization](#customization)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

</details>

## About

Ideas shouldn't have to wait for a new document or a save dialog. Quick Note opens
with an editable draft on first launch. On return, it restores your last note and
cursor. Edits save to a local SQLite database as you type.

[![Quick Note editor with three note tabs, line numbers, Markdown source and saved status](docs/images/editor.png)](docs/images/editor.png)

<p>
  <a href="docs/images/markdown.png"><img src="docs/images/markdown.png" alt="Markdown preview with headings, a list and a quote" width="49%"></a>
  <a href="docs/images/editor-dark.png"><img src="docs/images/editor-dark.png" alt="The same note in the source editor with the Graphite dark theme" width="49%"></a>
</p>

*Sample notes in the macOS app. Click an image for the full size.*

- Keep every note in one compact tab strip. Drag tabs to reorder them.
- Search all notes, or find and replace text in the current note.
- Write with line numbers, an active-line indicator and automatic bracket pairs.
- Switch between source and Markdown preview, or turn Markdown off.
- Use installed fonts in a fallback stack and share JSON themes and shortcuts.
- Choose English or Korean controls. Notes and exports use UTF-8.

Built with [Rust](https://www.rust-lang.org/),
[egui / eframe](https://github.com/emilk/egui) and
[SQLite](https://www.sqlite.org/). No account or webview.

## Getting started

### Download

The first public release is **v0.2.0-alpha.1**.

| Platform | Download | Install |
| --- | --- | --- |
| macOS 14+, Apple Silicon | [macOS ZIP](https://github.com/kitae0522/quick-note/releases/download/v0.2.0-alpha.1/Quick-Note-macos-arm64.zip) | Extract and move `Quick Note.app` to Applications. |
| Windows, x86-64, experimental | [Windows ZIP](https://github.com/kitae0522/quick-note/releases/download/v0.2.0-alpha.1/Quick-Note-windows-x86_64.zip) | Extract the full ZIP, then run `Quick Note.exe`. |

[Release notes](https://github.com/kitae0522/quick-note/releases/tag/v0.2.0-alpha.1)
include the current limitations. Download checksums are in
[SHA256SUMS](https://github.com/kitae0522/quick-note/releases/download/v0.2.0-alpha.1/SHA256SUMS).

Both packages are unsigned by a verified publisher. The macOS app uses local
ad-hoc signing and is not notarized. Physical Korean IME, VoiceOver and actual
Windows window/input behavior still need manual verification. Back up your note
data before trying the alpha.

### Build from source

Use Rust 1.95 or newer and the native build tools for your platform.

```sh
git clone https://github.com/kitae0522/quick-note.git
cd quick-note
cargo run --locked --release
```

For macOS app bundles, Windows C++ Build Tools, cross-compilation and portable
packages, see the [development guide](docs/development.md#run-and-build).

## Usage

Open the app and start typing. Closing the window or quitting keeps your notes.

| Action | macOS | Windows |
| --- | --- | --- |
| New note | Cmd+N | Ctrl+N |
| Search all notes | Cmd+P | Ctrl+P |
| Find in current note | Cmd+F | Ctrl+F |
| Replace in current note | Cmd+Option+F | Ctrl+Alt+F |
| Toggle Markdown preview | Cmd+Shift+M | Ctrl+Shift+M |
| Delete current note | Cmd+W | Ctrl+W |
| Close the window, keeping notes | Cmd+Shift+W | Ctrl+Shift+W |
| Settings | Cmd+, | Ctrl+, |

**Cmd/Ctrl+W deletes the active note.** Empty notes delete immediately; notes with
content ask for confirmation. In Settings, the same shortcut returns to the note.
Tab inserts four spaces in the editor. Use F6 to move focus between the editor and
tabs, and Esc to close a panel and return to writing.

See the [user guide](docs/user-guide.md) for every default shortcut, tab navigation,
search and replace, Markdown behavior and failed-save recovery.

### Where notes live

| Platform | Data folder |
| --- | --- |
| macOS | `~/Library/Application Support/Quick Note/` |
| Windows | `%LOCALAPPDATA%\Quick Note\` |

To back up your notes, quit the app and copy the entire data folder. If a save
fails, the editor keeps the text and blocks switching or closing. You can retry
saving or export a copy. Read [storage and recovery](docs/user-guide.md#storage-and-recovery)
before moving data or testing recovery behavior.

## Customization

Open Settings with Cmd/Ctrl+,. Preferences survive restarts.

| Setting | What you can change | Reference |
| --- | --- | --- |
| Appearance | Colors, spacing, editor size and JSON theme files | [Themes](docs/themes.md) |
| Fonts | Installed families, fallback order and mixed-script preview | [Font settings](docs/user-guide.md#themes) |
| Language | English / 한국어 and editable UTF-8 JSON catalogs | [Localization](i18n/README.md) |
| Shortcuts | Action bindings in `shortcut/bindings.json` | [Shortcut configuration](shortcut/README.md) |
| Markdown and motion | Preview support and reduced-motion settings | [User guide](docs/user-guide.md) |

## Roadmap

The alpha's next priorities are verification and distribution:

- [ ] Verify physical Korean IME input and VoiceOver on macOS.
- [ ] Verify input, accessibility and window behavior on a Windows desktop.
- [ ] Add macOS Developer ID signing and notarization, and Windows code signing.
- [ ] Reduce cold startup time. **200 ms is a budget, not a guarantee.**

Report bugs and suggest improvements in
[GitHub Issues](https://github.com/kitae0522/quick-note/issues).
Measurement methods and their limits are in the
[development guide](docs/development.md#verification-and-speed).

## Contributing

For a bug report, include the app version, operating system, reproduction steps
and a short input example. For a contribution, fork the repository, create a
branch and open a pull request with a focused change.

Follow the [Rust conventions](docs/rust-style.md). Keep each authored source file
at or below 250 lines; upstream files in `vendor/` keep their original structure.
Use isolated data for tests and preserve atomic saves, UTF-8 text and recovery.

Run the standard checks before submitting code:

```sh
./scripts/check.sh
```

The [development guide](docs/development.md) covers GUI checks, release builds,
startup measurements and CPU/memory profiling. Automated input replays do not
replace physical IME or screen-reader testing.

## License

Quick Note's own code is [MIT licensed](LICENSE).
External libraries, fonts and icons retain their respective licenses.
See [third-party notices](assets/licenses/README.md); release packages include
them in the `Licenses` directory.

## Acknowledgments

- Helium and Sublime Text inspired the compact tabs and persistent draft workflow.
- Font and icon authors are credited in the [asset notices](assets/licenses/README.md).
- README structure adapted from [Best-README-Template](https://github.com/othneildrew/Best-README-Template).

<p align="right"><a href="#readme-top">Back to top</a></p>
