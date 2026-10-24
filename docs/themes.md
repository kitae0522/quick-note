# Themes

Start with **Settings → Export theme…**, or copy [helium.json](../assets/themes/helium.json).
Edit the UTF-8 JSON and choose **Import theme…**. Share that single file.

| Field | Purpose | Values |
| --- | --- | --- |
| `schemaVersion` | Format version | `1` |
| `name` | Name shown in Settings | 1–60 characters |
| `appearance` | Base control appearance | `light`, `dark` |
| `colors.background` | Editor and status background | `#RRGGBB` |
| `colors.toolbar` | Header and secondary surfaces | `#RRGGBB` |
| `colors.foreground` | Main text and controls | `#RRGGBB` |
| `colors.secondary` | Hints and secondary text | `#RRGGBB` |
| `colors.accent` | Focus rings and links | `#RRGGBB` |
| `colors.divider` | Surface boundaries | `#RRGGBB` |
| `colors.selection` | Selected text and hovered controls | `#RRGGBB` |
| `colors.currentLine` | Focused caret line background in the number gutter | Optional `#RRGGBB` |
| `typography.fontFamily` | Editor font | An installed family name, a bundled family, `system`, or `monospace` |
| `typography.fontSize` | Editor size in points | 11–32 |
| `typography.lineSpacing` | Additional editor line height | 0–24 |
| `layout.editorPadding` | Editor edge padding | 12–64 |
| `layout.toolbarHeight` | Header height in points | 32–56 |

Every field except `colors.currentLine` is required. Colors are opaque six-digit HEX.
When `currentLine` is omitted, its background blends the editor background 10% toward
the foreground. Existing themes remain valid. Files are limited to 64 KB.
`system` selects SF Pro when installed, with bundled Pretendard and Noto Sans CJK
fallbacks. `monospace` uses installed Menlo on macOS and Consolas on Windows,
falling back to the OS's generic monospace family when that face is absent.
Its grid gives Latin characters one cell and wide Hangul/CJK and emoji graphemes
two cells, without resizing their outlines. Combining accents keep their base cell.
Explicit Menlo and Consolas selections use the same grid. Installed family names
are resolved through CoreText on macOS and DirectWrite on Windows, without a fixed
allowlist. Use the family name shown in **Settings → Choose font…** (1–100
characters, case-insensitive), for example `Menlo`, `Arial`, or `맑은 고딕` when installed.
Font collection (`.ttc`) face indices are preserved. Korean and emoji keep character
fallback when the selected family does not contain them. Font files are never exported.
Built-in themes use a 44-point header and 28-point editor padding. Header actions
use 32-point targets, adapting their height for compact custom themes; tab delete
targets remain at least 24 points. Light and dark themes share the same geometry.

The font picker searches all installed families and supports ↑/↓ and Enter from its
search field, plus Tab navigation. Your choice is saved independently of colors and
survives theme changes and restarts. **Use theme font** clears that preference and
restores `typography.fontFamily`. Exporting a theme includes the currently selected font.
The selected font applies to the source editor; the app controls retain their UI font.
Installed proportional fonts retain their natural widths when explicitly selected.

The native catalog starts in a background thread at launch. The note and bundled
fallbacks remain available while native defaults load. Real medium and bold faces,
TTC indices and variable-font coordinates are retained. Files load on selection or
when needed for character fallback; unused previous selections are released. A saved
font that was removed produces a notice and leaves the editor usable with its default
font. Choose another font or **Use theme font** to resolve it. Restart after installing
new fonts to refresh the catalog.

Imported files live in the app's `Themes` folder. Open it through **Open themes folder**.
Changes reload on returning to the app or with Cmd/Ctrl+Shift+R. Invalid JSON, unknown
fonts or unsupported values show an error and keep the last applied theme. A theme's
font must be installed on the receiving computer unless an editor font preference
overrides it.

JSON is the supported sharing format. CSS, YAML, TOML and executable scripts are not
interpreted. See the [JSON Schema](theme.schema.json) for editor completion and validation.
