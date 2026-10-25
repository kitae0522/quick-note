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
| `typography.fontFamily` | Primary editor font | An installed family name, a bundled family, `system`, or `monospace` |
| `typography.fontFallbacks` | Additional families in priority order | Optional array, up to 7 family names |
| `typography.fontSize` | Editor size in points | 11–32 |
| `typography.lineSpacing` | Additional editor line height | 0–24 |
| `layout.editorPadding` | Editor edge padding | 12–64 |
| `layout.toolbarHeight` | Header height in points | 32–56 |

Every field except `colors.currentLine` and `typography.fontFallbacks` is required.
Colors are opaque six-digit HEX.
When `currentLine` is omitted, its background blends the editor background 10% toward
the foreground. Existing themes remain valid. Files are limited to 64 KB.
`system` selects SF Pro when installed, with bundled Pretendard and Noto Sans CJK
fallbacks. `monospace` uses installed Menlo on macOS and Consolas on Windows,
falling back to the OS's generic monospace family when that face is absent.
Its grid gives Latin characters one cell and wide Hangul/CJK and emoji graphemes
two cells, without resizing their outlines. Combining accents keep their base cell.
Explicit Menlo and Consolas selections use the same grid. Installed family names
are resolved through CoreText on macOS and DirectWrite on Windows, without a fixed
allowlist. Use the family name shown in **Settings → Choose fonts…** (1–100
characters, case-insensitive), for example `Menlo`, `Arial`, or `맑은 고딕` when installed.
Font collection (`.ttc`) face indices are preserved. Korean and emoji keep character
fallback when the selected family does not contain them. Font files are never exported.
Built-in themes use a 44-point header and 28-point editor padding. Header actions
use 32-point targets, adapting their height for compact custom themes; tab delete
targets remain at least 24 points. Light and dark themes share the same geometry.

The font picker searches all installed families. Check a family to add it, and use
the priority buttons to move it up or down or remove it. From search, ↑/↓ browses and
Enter adds or removes the highlighted family; Tab reaches every priority button.
The list labels built-in, installed and unavailable families. Unavailable presets
cannot be added, while missing families in an existing stack remain removable.
The selection count explains the eight-family limit. A mixed-script preview uses
the current editor stack without changing the note. The preview stays above the
independently scrolling order and candidate lists. Narrow windows use an expandable
**Font order** section; short windows keep its preview inside that section. Tab and
Enter expand it, and the move buttons retain focus after reordering or resizing.
When both lists cannot fit, they share a scroll area below the search field.
Resizing reveals the focused candidate without overriding manual scrolling.
Choose 1–8 distinct families. Fonts are tried from top to bottom for each character,
before automatic bundled fallbacks. A monospace primary keeps every fallback on its grid.
Your ordered selection is saved independently of colors and survives theme changes
and restarts. **Use theme font** clears that preference and restores the theme's full
font stack. Exporting a theme includes the current stack. Existing single-font settings
and themes remain valid. For example, a theme's `typography` object can contain:

```json
{
  "fontFamily": "Menlo",
  "fontFallbacks": ["Pretendard", "Noto Sans CJK"],
  "fontSize": 14,
  "lineSpacing": 6
}
```

The selected font applies to the source editor; the app controls retain their UI font.
Markdown code retains its own monospace grid when the editor font changes.
Installed proportional fonts retain their natural widths when explicitly selected.

The native catalog starts in a background thread at launch. The note and bundled
fallbacks remain available while native defaults load. Real medium and bold faces,
TTC indices and variable-font coordinates are retained. Files load on selection or
when needed for character fallback; unused previous selections are released. A saved
family that was removed is skipped when another configured family is available. It
stays in the saved order and is marked **Not installed** in the picker. If no configured
family is available, a notice appears and the bundled editor remains usable. Choose
another font or **Use theme font** to resolve it. Restart after installing new fonts
to refresh the catalog.

Imported files live in the app's `Themes` folder. Open it through **Open themes folder**.
Changes reload on returning to the app or with Cmd/Ctrl+Shift+R. Invalid JSON,
unresolvable font stacks or unsupported values show an error and keep the last applied
theme. At least one configured family must be available on the receiving computer
unless an editor font preference overrides it.

JSON is the supported sharing format. CSS, YAML, TOML and executable scripts are not
interpreted. See the [JSON Schema](theme.schema.json) for editor completion and validation.
