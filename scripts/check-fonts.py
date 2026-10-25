#!/usr/bin/env python3
"""Verify ordered local fonts, rendering and restoration in an isolated Mac window."""
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


def resize_fonts(window, width, height):
    window.command("resize font settings", "resize", width=width, height=height)
    for _ in range(30):
        report = window.command("wait for font layout", "report")
        if report["content_bounds"] == [0.0, 0.0, float(width), float(height)]:
            return report
    raise AssertionError("Font settings did not reach the requested size")


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
        report = app.command("add installed font", key="enter")
        assert report["editor_fonts"] == ["monospace", "Arial"]
        for _ in range(4):
            app.command("reach Arial priority up", key="tab")
        report = app.command("make Arial the primary font", key="enter")
        assert report["editor_fonts"] == ["Arial", "monospace"]
        app.command("return to note", key="escape")
        open_fonts(app)
        app.command("search bundled Pretendard", "text", text="Pretendard")
        report = app.command("add another fallback", key="enter")
        assert report["editor_fonts"] == ["Arial", "monospace", "Pretendard"]
        for _ in range(7):
            app.command("reach Pretendard priority up", key="tab")
        report = app.command("raise fallback priority", key="enter")
        assert report["editor_fonts"] == ["Arial", "Pretendard", "monospace"]
        for _ in range(4):
            app.command("reach Menlo removal", key="tab")
        report = app.command("remove lower priority font", key="enter")
        assert report["editor_fonts"] == ["Arial", "Pretendard"]
        app.command("reach reset", key="tab")
        app.command("reach Arial move down", key="tab")
        resize_fonts(app, 320, 240)
        report = app.command("move primary to last position", key="enter")
        assert report["content_bounds"] == [0.0, 0.0, 320.0, 240.0]
        assert report["editor_fonts"] == ["Pretendard", "Arial"]
        report = app.command("move it back using transferred focus", key="enter")
        assert report["editor_fonts"] == ["Arial", "Pretendard"]
        time.sleep(0.3)
        app.command("settled narrow font order", "report")
        app.command("visible narrow font action", "screenshot", name="narrow-font-order")
        resize_fonts(app, 920, 600)
        report = app.command("reorder through preserved wide focus", key="enter")
        assert report["editor_fonts"] == ["Pretendard", "Arial"]
        report = app.command("restore Arial using transferred focus", key="enter")
        assert report["editor_fonts"] == ["Arial", "Pretendard"]
        checks.append("At 320px, reorder transfers keyboard focus to an enabled action at either end")
        checks.append("Resizing a focused font action preserves Enter activation in both layouts")
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
        with sqlite3.connect(data / "notes.sqlite3") as database:
            saved = database.execute("SELECT value FROM app_state WHERE key='editorFont'").fetchone()[0]
        assert json.loads(saved) == ["Arial", "Pretendard"]
        checks.append("Keyboard search adds fonts; priority and removal persist without changing notes")
        checks.append("The ordered stack renders real local font bytes in the native window")
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
        assert report["editor_fonts"] == ["Arial", "Pretendard"]
        assert report["body"] == body and report["selection"] == before["selection"]
        app.command("restored local font", "screenshot", name="restored-font")
        checks.append("Restart restores the font stack, UTF-8 text and cursor")
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
        assert report["editor_fonts"] == ["monospace"]
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
