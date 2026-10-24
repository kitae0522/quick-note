#!/usr/bin/env python3
"""Check search safe areas and keyboard capture in isolated native macOS windows."""
import json
from pathlib import Path
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("safe-area-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
app = None


def assert_safe(report):
    left, top, right, bottom = report["search_popup_bounds"]
    x, y, end_x, end_y = report["content_bounds"]
    assert left >= x + 16 and right <= end_x - 16
    assert top >= y + 48 and bottom <= end_y - 16


def resize(width, height):
    app.command("resize the native window", "resize", width=width, height=height)
    for _ in range(30):
        report = app.command("wait for native resize", "report")
        x, y, end_x, end_y = report["content_bounds"]
        if abs(end_x - x - width) < 1 and abs(end_y - y - height) < 1:
            report = app.command("settled search layout", "report")
            assert_safe(report)
            return report
    raise AssertionError(f"Native window did not resize to {width}x{height}")


try:
    with tempfile.TemporaryDirectory(prefix="quick-note-safe-area-") as temporary:
        data = Path(temporary)
        app = Native(data, "search", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        body = "# Search capture\n\nA safe place for café ideas. 아이디어 💡"
        report = app.command("save UTF-8 fixture", "text", text=body)
        note_id = report["note_id"]
        for index in range(8):
            app.command("new fixture note", key="n", command=True)
            app.command("save fixture title", "text", text=f"# Idea {index:02}\n\nA thought worth keeping.")
        app.command("open search", key="p", command=True)
        report = app.command("settled search results", "report")
        assert_safe(report)
        app.command("desktop search safe area", "screenshot", name="search-desktop")
        for width, height in [(320, 240), (620, 240), (920, 600)]:
            resize(width, height)
            app.command("capture resized search", "screenshot", name=f"search-{width}")
        checks.append("Search remains within 16pt window insets and 8pt below the header at native minimum and standard sizes")

        resize(320, 240)
        report = app.command("search after resizing", "text", text="café")
        assert report["query"] == "café"
        app.command("settled filtered search", "report")
        report = app.command("open matching note with Enter", key="enter")
        assert report["note_id"] == note_id and report["body"] == body and report["editor_focused"]
        assert db_note(data, note_id)[0] == body
        checks.append("The search query retains focus after resizing; Enter opens the original UTF-8 note")

        app.command("open settings", key="comma", command=True)
        app.command("settled settings", "report")
        app.command("focus Korean language", key="tab")
        app.command("select Korean", key="enter")
        app.command("return to writing", key="escape")
        app.command("open Korean search", key="p", command=True)
        report = app.command("settled Korean search", "report")
        assert report["language"] == "ko"
        assert_safe(report)
        app.command("Korean search safe area", "screenshot", name="search-korean-320")
        report = app.command("return without changing the note", key="escape")
        assert report["body"] == body and report["editor_focused"]
        checks.append("Korean search fits the minimum window and Escape restores writing without changing note data")
        app.close()
        app = None
    (OUTPUT / "result.json").write_text(json.dumps({"checks": checks, "startups": startups,
        "actions": actions, "input": "Scripted egui events in real native windows; isolated data; not physical keyboard"},
        ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(OUTPUT), "checks_passed": len(checks)}, ensure_ascii=False))
finally:
    if app is not None:
        app.cleanup()
