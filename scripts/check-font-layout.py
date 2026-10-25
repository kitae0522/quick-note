#!/usr/bin/env python3
"""Verify font resize focus, short-window navigation and the Markdown grid on a GUI Mac."""
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("font-layout-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
BINARY = ROOT / "target/release/native-check"
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []


def open_fonts(window, tabs=4):
    window.command("open options", key="comma", command=True)
    for _ in range(tabs):
        window.command("reach font picker", key="tab")
    window.command("open font picker", key="enter")


def resize(window, width, height):
    window.command("resize font settings", "resize", width=width, height=height)
    for _ in range(30):
        report = window.command("wait for native size", "report")
        if report["content_bounds"] == [0.0, 0.0, float(width), float(height)]:
            return
    raise AssertionError("The native window did not reach the requested size")


def visible_focus(window):
    time.sleep(0.2)
    for _ in range(30):
        report = window.command("wait for visible focused control", "report")
        bounds = report["focused_bounds"]
        left, top, right, bottom = report["content_bounds"]
        if (bounds and left <= bounds[0] < bounds[2] <= right
                and top <= bounds[1] < bounds[3] <= bottom):
            return report
    raise AssertionError(f"The focused control remains outside the window: {report}")


def save_settings(data, values):
    with sqlite3.connect(data / "notes.sqlite3") as database:
        database.executemany("INSERT OR REPLACE INTO app_state(key,value) VALUES(?,?)", values.items())


def font_report(window):
    for _ in range(30):
        report = window.command("wait for native font installation", "report")
        if report["platform_font"] is not None:
            return report
    raise AssertionError("Native fonts did not finish installing")


with tempfile.TemporaryDirectory(prefix="quick-note-font-layout-") as temporary:
    data = Path(temporary)
    app = Native(data, "candidate", output=OUTPUT, binary=BINARY, actions=actions, startups=startups)
    try:
        body = "# Capture\n\nCafé 한글 💡\n\n```\nAAAA|\n한글|\n💡💡|\n```\n"
        before = app.command("save UTF-8 grid fixture", "text", text=body)
        note_id = before["note_id"]
        widths = font_report(app)["monospace_widths"]
        open_fonts(app)
        for _ in range(4):
            app.command("reach the Noto Sans CJK checkbox", key="tab")
        visible_focus(app)
        resize(app, 320, 240)
        visible_focus(app)
        app.command("focused candidate after resizing", "screenshot", name="01-visible-candidate")
        report = app.command("activate the visible candidate", key="enter")
        assert report["editor_fonts"] == ["monospace", "Noto Sans CJK"]
        assert report["body"] == body and report["selection"] == before["selection"]
        checks.append("Resizing reveals the focused candidate and Enter adds that same family")
        app.close()
    finally:
        app.cleanup()

    stack = ["monospace", "Pretendard", "Noto Sans CJK", "Geist", "Geist Mono",
             "Geist Medium", "Geist Bold", "Noto Emoji"]
    theme = json.loads((ROOT / "assets/themes/helium.json").read_text(encoding="utf-8"))
    theme["layout"]["toolbarHeight"] = 56
    (data / "Themes/compact.json").write_text(json.dumps(theme), encoding="utf-8")
    save_settings(data, {"editorFont": json.dumps(stack), "language": "ko", "theme": "compact.json"})
    app = Native(data, "full-stack", output=OUTPUT, binary=BINARY, actions=actions, startups=startups)
    try:
        resize(app, 320, 240)
        # The imported active theme adds an appearance choice before the font picker.
        open_fonts(app, tabs=5)
        app.command("reach font order disclosure", key="tab")
        app.command("expand font order", key="enter")
        # Reset, 22 enabled row actions, then the first enabled candidate (Pretendard).
        for _ in range(24):
            app.command("navigate the full stack to its candidate", key="tab")
        visible_focus(app)
        app.command("full Korean stack has a visible candidate", "screenshot", name="02-full-stack-candidate")
        report = app.command("remove Pretendard using the candidate checkbox", key="enter")
        assert report["editor_fonts"] == [font for font in stack if font != "Pretendard"]
        visible_focus(app)
        assert db_note(data, note_id)[0] == body
        checks.append("At 320×240 with a 56pt header, the full Korean stack keeps its candidate reachable")
        app.close()
    finally:
        app.cleanup()

    save_settings(data, {"editorFont": json.dumps(["Geist Mono", "Pretendard"]),
                         "language": "en", "theme": "helium"})
    app = Native(data, "markdown", output=OUTPUT, binary=BINARY, actions=actions, startups=startups)
    try:
        report = font_report(app)
        assert report["editor_fonts"] == ["Geist Mono", "Pretendard"]
        assert report["monospace_widths"] == widths
        assert max(widths) - min(widths) < 0.01
        app.command("open Markdown preview", key="m", command=True, shift=True)
        report = app.command("settled Markdown preview", "report")
        assert report["mode"] == "preview" and report["body"] == body
        app.command("Markdown grid after the editor font change", "screenshot", name="03-independent-markdown-grid")
        assert db_note(data, note_id)[0] == body
        checks.append("Changing the editor to Geist Mono preserves native Markdown glyph advances")
        app.close()
    finally:
        app.cleanup()

result = {"passed": True, "checks": checks, "actions": actions, "startups": startups,
          "input": "Scripted egui input in isolated native macOS windows; no physical keyboard or VoiceOver"}
(OUTPUT / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"output": str(OUTPUT), "checks_passed": len(checks)}, ensure_ascii=False))
