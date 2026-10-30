#!/usr/bin/env python3
"""Verify settings-page navigation and capture safety with isolated native windows."""
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("settings-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
app = None


def open_settings(window):
    window.command("open settings", key="comma", command=True)
    report = window.command("settled settings page", "report")
    assert report["settings_open"] and not report["editor_focused"]
    return report


try:
    with tempfile.TemporaryDirectory(prefix="quick-note-settings-") as temporary:
        data = Path(temporary)
        app = Native(data, "navigation", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        body = "# A place for ideas\n\nSettings stay out of the way. 아이디어 💡"
        app.command("save UTF-8 fixture", "text", text=body)
        before = app.command("remember cursor", key="left")
        note_id = before["note_id"]
        report = open_settings(app)
        assert report["body"] == body and report["selection"] == before["selection"]
        app.command("settings page", "screenshot", name="settings-page")
        report = app.command("close settings with Cmd W", key="w", command=True)
        assert not report["settings_open"] and report["editor_focused"]
        assert report["pending_delete"] is None and report["note_id"] == note_id
        assert report["selection"] == before["selection"] and db_note(data, note_id)[0] == body
        checks.append("Settings replaces the document; Cmd W returns to the same note and cursor")

        open_settings(app)
        for _ in range(4):
            app.command("reach font selection", key="tab")
        report = app.command("open font settings", key="enter")
        assert report["settings_open"]
        app.command("font settings page", "screenshot", name="font-settings")
        report = app.command("return from font settings", key="escape")
        assert report["editor_focused"] and report["body"] == body
        checks.append("Installed-font selection uses the settings page and Escape returns to writing")

        app.command("open keyboard help", key="slash", command=True)
        report = app.command("keyboard settings page", "report")
        assert report["settings_open"]
        app.command("keyboard shortcuts page", "screenshot", name="keyboard-settings")
        report = app.command("back to general settings", key="enter")
        assert report["settings_open"]
        app.command("settled general settings", "report")
        app.command("focus Korean", key="tab")
        report = app.command("switch language", key="enter")
        assert report["language"] == "ko"
        app.command("Korean settings", "screenshot", name="settings-page-ko")
        report = app.command("return to note", key="escape")
        assert not report["settings_open"] and report["editor_focused"] and report["body"] == body
        checks.append("Keyboard help returns to settings; language selection updates the page without changing notes")

        report = app.command("create blank draft", key="n", command=True)
        blank_id = report["note_id"]
        open_settings(app)
        report = app.command("close settings over blank draft", key="w", command=True)
        assert report["note_id"] == blank_id and db_note(data, blank_id) is not None
        assert report["pending_delete"] is None and len(report["tabs"]) == 2
        report = app.command("delete blank draft from editor", key="backspace", command=True, shift=True)
        assert db_note(data, blank_id) is None and db_note(data, note_id)[0] == body
        checks.append("Cmd W in settings keeps blank notes; explicit Cmd Shift Backspace deletes the blank draft")
        with sqlite3.connect(data / "notes.sqlite3") as database:
            database.execute("CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT, 'settings close fixture'); END;")
        report = app.command("retain a failed edit", "text", text=" UNSAVED")
        assert report["dirty"]
        unsaved = report["body"]
        open_settings(app)
        app.command("failed close from settings", key="w", command=True, shift=True)
        assert app.process.poll() is None
        app.command("keep editing", key="enter")
        report = app.command("returned to the unsaved note", "report")
        assert not report["settings_open"] and report["editor_focused"] and report["dirty"]
        assert report["body"] == unsaved and db_note(data, note_id)[0] == body
        with sqlite3.connect(data / "notes.sqlite3") as database:
            database.execute("DROP TRIGGER fail_save")
        report = app.command("save the recovered edit", key="s", command=True)
        assert not report["dirty"] and db_note(data, note_id)[0] == unsaved
        checks.append("A failed save and window close from settings preserve the draft and return Keep editing to the note")
        app.close()
        app = None
    (OUTPUT / "result.json").write_text(json.dumps({"checks": checks, "startups": startups,
        "actions": actions, "input": "Scripted egui events in native Mac windows; not physical keyboard"},
        ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(OUTPUT), "checks_passed": len(checks)}, ensure_ascii=False))
finally:
    if app is not None:
        app.cleanup()
