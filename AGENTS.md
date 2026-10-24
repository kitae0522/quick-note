# Quick Note project rules

- Follow the supplied Google Rust Style Guide and the Cargo application conventions
  summarized in [docs/rust-style.md](docs/rust-style.md).
- Keep each authored source file at or below 250 physical lines, including comments
  and blank lines. This covers `src/`, `tests/`, `tools/`, `examples/`, `scripts/` and `i18n/`.
  Preserve upstream structure in `vendor/`; do not reformat or split it to satisfy
  the application limit.
- Split by responsibility. Do not compress formatting, remove useful comments,
  invent organizing traits, or add abstraction layers to meet the line limit.
- Non-leaf modules contain module declarations and re-exports only. Leaf modules
  own behavior, and names identify their primary type or capability.
- Keep note transactions atomic, retain UTF-8 text and persisted cursor formats,
  and preserve failed-write recovery. Tests and diagnostics use isolated data.
- Use `./scripts/check.sh` for source-size, formatting, Clippy and behavior checks.
  Use `python3 scripts/check-ui.py` for native recovery verification on a GUI Mac.
- CPU and memory claims require measured release builds. Report process CPU time,
  the observation period, physical footprint and RSS separately. The native driver
  polls and captures screenshots, so its resource use is not an idle-app benchmark.

- Keep user-facing app copy in UTF-8 `i18n/en.json` and `i18n/ko.json`. Use typed
  catalog fields and named placeholders; never translate or rewrite user note data.
