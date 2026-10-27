# Quick Note text input fixes

Base: crates.io `egui` 0.37.0. Keep the upstream tree, Cargo metadata and licenses.

The crates.io archive omits the workspace license files. Restored `LICENSE-MIT`
and `LICENSE-APACHE` from upstream Git blobs
`673ea5f0505d6fa65c6a2b6a753c93b8e19d11b9` and
`11069edd79019f7dafbe3138841cf289209270dd`, respectively.

- Add opt-in `TextEdit::input_handler` and `TextEditMutation`. The callback runs
  inside TextEdit's existing event loop, using the current selection after earlier
  events and pointer interaction. It is skipped during active IME composition.
- Apply handled replacements through the existing TextBuffer, layout and undo
  paths. Each custom text replacement records its preceding state. Cursor-only
  operations do not mark the document changed. Unhandled events use upstream code.
- Only Quick Note's source editor opts into the hook. Search and settings use
  standard text editing; the undo-boundary fix below applies to all TextEdit users.
- Remove the deprecated `core::f32` module import in `Scene` so path-dependency
  builds use the primitive's constants without a deprecation warning.
- Leave the opt-in epaint monospace-cell policy unchanged in the font-tweak
  debug inspector; application font definitions own that policy.
- Record an undo boundary before Cmd+Backspace, including when an IME commit
  precedes deletion in the same event batch. Undo restores the visible composed text.

Post-layout completion and frame-level event rewriting cannot reliably wrap the
current selection across mixed navigation, IME, paste and fast input batches.
Application pairing rules belong in `src/ui/editor_pairs.rs`; the vendor hook
contains no bracket-specific behavior.

Regression coverage is in `tests/interface/pair*.rs` and `scripts/check-pairs.py`.
Remove the local hook when a compatible egui release provides equivalent ordered
input customization, then rerun the pairing, IME, undo and native recovery checks.
