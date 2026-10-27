# Quick Note fallback advances

Base: crates.io `epaint` 0.37.0. Preserve the upstream source tree and licenses.

The crates.io archive omits the workspace license files. Restored `LICENSE-MIT`
and `LICENSE-APACHE` from upstream Git blobs
`673ea5f0505d6fa65c6a2b6a753c93b8e19d11b9` and
`11069edd79019f7dafbe3138841cf289209270dd`, respectively.

`FontTweak::monospace_cell_width` optionally sets a cell width in em units without
resizing outlines or changing vertical metrics. Unicode display widths come from
`unicode-width` 0.2.2; shaped and rasterized grapheme clusters keep their ink centered
in the resulting cells. Logical glyph positions and advances change together, so
wrapping, cursor placement and selection follow the same geometry. Combining marks
and emoji sequences remain clusters. Tabs and thin spaces retain explicit font settings.
The default `None` preserves upstream behavior. Width queries and overflow glyphs
use the same cell policy.

Quick Note opts in for its monospace editor families and their CJK/emoji fallbacks.
The cell width comes from the chosen native monospace face's metrics. UI and
explicitly selected proportional fonts retain natural spacing.

See `src/appearance/font_stack.rs` and `tests/font_spacing.rs` for policy and
multi-script, display-scale and cursor regression coverage. Remove this patch
when upstream supplies equivalent control over fallback advances.

Variable faces also receive their rendered point size as the default `opsz` axis.
SF Pro otherwise keeps its 28pt display spacing even in 11–14pt editor and UI text.
The font clamps to its supported range; explicit FontTweak or TextFormat coordinates
still override this default. See `tests/font_optical_size.rs` for actual text-width
and override coverage. Static fonts and monospace cell widths remain unchanged.
