# Development

[Back to Quick Note](../README.md)

Run the commands below from the repository root. See the
[user guide](user-guide.md) for app behavior and data recovery.

## Run and build

Use Rust 1.95 or newer. The lockfile pins dependencies.

```sh
cargo run --locked --release
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

## Verification and speed

Authored application, test and tool files stay within 250 lines. Upstream files
under `vendor/` retain their original structure. The size guard runs with the
standard checks. See [Rust conventions](rust-style.md).

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
[Patch provenance and scope](../vendor/winit/QUICK_NOTE_PATCHES.md) explains the upstream
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
