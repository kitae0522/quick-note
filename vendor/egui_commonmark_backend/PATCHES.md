Based on egui_commonmark_backend 0.26.0 from crates.io (MIT OR Apache-2.0).

`src/elements.rs` has a focused code-block accessibility patch:

- Give the read-only text field and copy control meaningful accessible names.
- Use Copy/Copied text instead of an uncommon Unicode symbol requiring a system-font lookup.
- Keep the copy target at least 24 pt high and reserve its space in the code block.

`src/misc.rs` renders code using the theme's monospace foreground. Upstream's fixed
syntax RGB colors have insufficient contrast against Quick Note's light and
custom backgrounds. Markdown still renders fenced code, indentation and copy;
syntax coloring is intentionally omitted.
Strong text and headings use the configured heading font family; egui's
`.strong()` alone changes color, which does not distinguish bold text when
the app assigns one accessible foreground color to both.

Quick Note's interface tests exercise this path with AccessKit and the real renderer.
Recheck these changes when updating the upstream dependency.

List markers reserve the same first-row height as the following wrapped item UI:
the larger of the text line height (including extra line spacing) and the UI's
minimum interaction height. Bullet radii still use the font height. This aligns
filled bullets, hollow bullets and ordered numbers without enlarging the dots.
