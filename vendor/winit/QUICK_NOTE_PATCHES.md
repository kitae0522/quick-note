# Quick Note macOS backend fixes

Base: the crates.io `winit` 0.30.13 source, retaining its module tree and licenses.
Only `src/platform_impl/macos/{mod.rs,view.rs,app_state.rs,window_delegate.rs}` and
the added `ime_commit.rs` / `ime_startup.rs` change native behavior.
Windows backend source is unchanged.

- Preserve a pending composition when AppKit clears marked text before `insertText`.
  Return standard marked-text attributes. Based on upstream
  [#4650](https://github.com/rust-windowing/winit/pull/4650).
- Clear marked text immediately on commit and forward a distinct trailing key payload.
  Based on [#4478](https://github.com/rust-windowing/winit/pull/4478). The local policy
  also ignores an identical extra insert callback within that same committed key batch;
  a new key batch or new preedit still accepts legitimate repeated characters.
- Discard the input method's marked text when disabling IME, as in upstream
  [#4745](https://github.com/rust-windowing/winit/pull/4745), so focus changes cannot
  resume and commit a previous field's composition again.
- Backport the guarded, synchronous first-key policy from
  [#4744](https://github.com/rust-windowing/winit/pull/4744), head
  `5c5c4db9aff1ff5961dc4a464f62e1fbde497a10`. It retries only an initial Apple Korean
  single-jamo fallback, at most once, with unchanged source/focus and no modifier,
  repeat, preedit, command or nested-dispatch interference. No Hangul reconstruction.
- Route Cmd+Backspace directly to the editor. Commit and discard visible marked text
  before forwarding that same key, so composition completion cannot consume deletion
  or later reinsert it. Ordinary Backspace still belongs to the active input method.
- Preserve activation requested for an initially hidden window until its first reveal.
  eframe reveals its window after GPU initialization, after launch activation has run.
  Request activation once at that reveal, respecting both the window's `active` flag
  and the event loop's `activate_ignoring_other_apps` setting. Later hide/show calls
  keep the upstream behavior.

`tests/ime_policies.rs` compiles the exact two policy modules and their routing-to-editor
checks (16 tests) on every platform. Native-window egui event replays
are in `scripts/check-input.py`. Those tests do not establish physical macOS IME or
Windows behavior. Keep that distinction when reporting validation.

`examples/window_activation.rs` checks actual AppKit activation after a simulated
hidden-renderer interval. The `active`, `inactive` and `no-activate` cases check
requested activation and both opt-outs without focus retries, notes or a renderer.

Remove this patch when a compatible released winit version contains equivalent
fixes, then rerun the policy and native user-flow checks before switching versions.
