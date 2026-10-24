#!/usr/bin/env python3
"""Verify installed-font selection, rendering and restoration in an isolated Mac window."""
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("fonts-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
app = None


def open_fonts(window):
    window.command("open options", key="comma", command=True)
    for _ in range(4):
        window.command("reach font picker", key="tab")
    window.command("open font picker", key="enter")


try:
    with tempfile.TemporaryDirectory(prefix="quick-note-fonts-") as temporary:
        data = Path(temporary)
        app = Native(data, "select", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        deadline = time.monotonic() + 3
        while True:
            defaults = app.command("wait for native font defaults", "report")
            if defaults["platform_font"] is not None or time.monotonic() >= deadline:
                break
        assert defaults["platform_font"]["cell_em"] > 0
        assert defaults["ui_font"] == "native-ui"
        body = ("iiii|\nWWWW|\n1234|\n한글|\n中文|\n💡💡|\n🇰🇷🇺🇸|\n"
                "👨‍👩‍👧‍👦💡|\ne\u0301e\u0301e\u0301e\u0301|\n\n한글도 안전하게. 아이디어 💡")
        before = app.command("save UTF-8 fixture", "text", text=body)
        app.command("default grid for Latin, Hangul and emoji", "screenshot", name="default-grid")
        note_id = before["note_id"]
        open_fonts(app)
        app.command("search local Arial family", "text", text="Arial")
        app.command("browse next family", key="down")
        app.command("return to exact match", key="up")
        report = app.command("apply installed font", key="enter")
        deadline = time.monotonic() + 3
        while report["editor_font"] != "Arial" and time.monotonic() < deadline:
            report = app.command("apply when catalog is ready", key="enter")
        assert report["editor_font"] == "Arial"
        deadline = time.monotonic() + 3
        while report["local_font"] is None and time.monotonic() < deadline:
            report = app.command("wait for rendered font", "report")
        assert report["local_font"]["bytes"] > 0
        selected = report["local_font"]
        app.command("font picker", "screenshot", name="font-picker")
        report = app.command("return to writing", key="escape")
        assert report["editor_focused"] and report["body"] == body
        assert report["selection"] == before["selection"]
        app.command("actual local font", "screenshot", name="local-font-editor")
        assert db_note(data, note_id)[0] == body
        checks.append("Keyboard search and selection render a real local font without changing notes")
        app.close()
        app = None

        app = Native(data, "restart", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        deadline = time.monotonic() + 3
        while True:
            report = app.command("wait for saved font", "report")
            if report["local_font"] is not None or time.monotonic() >= deadline:
                break
            time.sleep(0.01)
        assert report["editor_font"] == "Arial" and report["local_font"] == selected
        assert report["body"] == body and report["selection"] == before["selection"]
        app.command("restored local font", "screenshot", name="restored-font")
        checks.append("Restart restores the selected native font, UTF-8 text and cursor")
        app.close()
        app = None

        with sqlite3.connect(data / "notes.sqlite3") as database:
            database.execute("UPDATE app_state SET value=? WHERE key='editorFont'",
                             ("Missing Local Font 92ab",))
        app = Native(data, "missing", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        report = app.command("write despite missing font", "text", text=" Still here.")
        assert report["editor_focused"] and report["local_font"] is None
        assert report["body"] == body + " Still here."
        app.command("missing font notice", "screenshot", name="missing-font")
        open_fonts(app)
        app.command("focus reset from search", key="tab")
        report = app.command("use theme font", key="enter")
        assert report["editor_font"] == "monospace"
        report = app.command("wait for theme font frame", "report")
        assert report["local_font"] is None
        app.command("return to editor", key="escape")
        assert db_note(data, note_id)[0] == body + " Still here."
        checks.append("Missing saved font shows a usable fallback and keyboard reset preserves writing")
        app.close()
        app = None
    result = {"checks": checks, "startups": startups, "actions": actions,
              "input": "Scripted egui events in a real macOS window; not physical keyboard input"}
    (OUTPUT / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps({"output": str(OUTPUT), "checks_passed": len(checks)}, ensure_ascii=False))
finally:
    if app is not None:
        app.cleanup()
