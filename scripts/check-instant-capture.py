#!/usr/bin/env python3
"""Verify first-input capture and the real macOS status menu using isolated data."""
import argparse
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note
from resource_usage import frontmost_pid

ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--binary", type=Path)
arguments = parser.parse_args()
if arguments.binary is None:
    subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                    "--bin", "native-check"], cwd=ROOT, check=True)
binary = (arguments.binary or ROOT / "target/release/native-check").resolve()
output = ROOT / ".artifacts" / ("instant-capture-" + time.strftime("%Y%m%d-%H%M%S"))
output.mkdir(mode=0o700)
actions, startups, checks = [], [], []


def hide(app):
    app.sequence += 1
    (app.folder / "command.json").write_text(json.dumps({
        "sequence": app.sequence, "type": "key", "key": "escape"}), encoding="utf-8")
    deadline = time.monotonic() + 3
    while frontmost_pid() == app.process.pid and time.monotonic() < deadline:
        time.sleep(0.01)
    assert app.process.poll() is None and frontmost_pid() != app.process.pid


def count(data):
    with sqlite3.connect(data / "notes.sqlite3") as db:
        return db.execute("SELECT count(*) FROM notes").fetchone()[0]


with tempfile.TemporaryDirectory(prefix="quick-note-instant-") as temporary:
    data = Path(temporary)
    app = Native(data, "resident", output=output, binary=binary, actions=actions,
                 startups=startups, arguments=["--resident"])
    try:
        report = app.command("registered shortcuts and resident menu", "report")
        assert report["capture_registered"] and report["resident_menu"]
        assert not report["focus_helpers_enabled"]
        app.command("existing note text", "text", text="Original 한💡 note")
        original = app.command("set original cursor", key="left")
        before = count(data)
        app.command("new quick note", key="n", command=True, alt=True, shift=True)
        report = app.command("unused new capture", "report")
        assert report["body"] == "" and report["editor_focused"]
        assert count(data) == before and db_note(data, report["note_id"]) is None
        hide(app)
        report = app.command("hidden empty capture is discarded", "resident_report")
        assert report["note_id"] == original["note_id"] and count(data) == before
        assert report["selection"] == original["selection"]
        checks.append("Unused new captures leave no row and restore the original note and cursor")

        text = "Clipboard 한글 💡\r\nKeep UTF-8 and whitespace.\n"
        app.command("isolated clipboard text", "clipboard_fixture", text=text)
        foreground = frontmost_pid()
        report = app.command("capture from hidden native status menu", "resident",
                             item="Save clipboard as note")
        assert frontmost_pid() == foreground and foreground != app.process.pid
        assert report["note_id"] == original["note_id"]
        assert report["selection"] == original["selection"]
        assert count(data) == before + 1
        with sqlite3.connect(data / "notes.sqlite3") as db:
            captured = db.execute("SELECT id,body,closed FROM notes ORDER BY updated_at DESC LIMIT 1").fetchone()
        assert captured[1:] == (text, 1)
        time.sleep(0.4)
        report = app.command("recent list refreshes while hidden", "resident_report")
        recent = next(item for item in report["resident_menu"] if item["title"] == "Recent notes")
        assert recent["items"][0]["title"] == "Clipboard 한글 💡"
        checks.append("The actual NSStatusItem saves clipboard text while hidden without taking focus")

        app.command("open captured recent note", "resident", item="Clipboard 한글 💡")
        report = app.command("recent note is editable", "report")
        assert report["note_id"] == captured[0] and report["body"] == text
        assert report["editor_focused"] and report["window_focused"]
        checks.append("Recent notes reopen closed captures through the real AppKit status menu")
        app.command("start fresh from status menu", "resident", item="New quick note")
        app.command("first capture input", "text", text="First input 한💡")
        report = app.command("first input committed", "report")
        assert db_note(data, report["note_id"])[0] == "First input 한💡"
        assert count(data) == before + 2 and not report["dirty"]
        app.command("fresh capture screenshot", "screenshot", name="new-quick-note")
        checks.append("New quick notes commit their first input once with the editor already focused")

        hide(app)
        with sqlite3.connect(data / "notes.sqlite3") as db:
            db.execute("CREATE TRIGGER fail_capture BEFORE INSERT ON notes BEGIN SELECT RAISE(ABORT,'capture fixture'); END;")
        app.command("isolated failing clipboard", "clipboard_fixture", text="Recover clipboard 한💡")
        app.command("failed capture opens recovery editor", "resident", item="Save clipboard as note")
        report = app.command("failed text remains visible", "report")
        assert report["dirty"] and report["body"] == "Recover clipboard 한💡"
        assert report["window_focused"] and (data / "recovery.json").exists()
        assert count(data) == before + 2
        app.command("escape cannot discard failed capture", key="escape")
        with sqlite3.connect(data / "notes.sqlite3") as db:
            db.execute("DROP TRIGGER fail_capture")
        report = app.command("retry first write", key="s", command=True)
        assert not report["dirty"] and count(data) == before + 3
        assert not (data / "recovery.json").exists()
        checks.append("A failed background capture reveals its recoverable text and retry commits once")
        app.command("restore original clipboard", "clipboard_restore")
        app.close()
    finally:
        if app.process.poll() is None:
            app.command("restore original clipboard after check", "clipboard_restore")
            app.cleanup()
        (output / "progress.json").write_text(json.dumps(actions, ensure_ascii=False, indent=2), encoding="utf-8")

with tempfile.TemporaryDirectory(prefix="quick-note-clipboard-cleanup-") as temporary:
    original = subprocess.run(["pbpaste"], check=True, stdout=subprocess.PIPE).stdout
    app = Native(Path(temporary), "cleanup", output=output, binary=binary, actions=actions,
                 startups=startups, arguments=["--resident"])
    try:
        app.command("isolated cleanup clipboard", "clipboard_fixture", text="Cleanup sentinel 한💡")
        try:
            app.command("simulate diagnostic command failure", "invalid_fixture_command")
        except RuntimeError as error:
            assert "Unknown command: invalid_fixture_command" in str(error)
        else:
            raise AssertionError("the fixture must fail before cleanup")
        app.cleanup()
        restored = subprocess.run(["pbpaste"], check=True, stdout=subprocess.PIPE).stdout
        assert restored == original
        checks.append("Diagnostic cleanup acknowledges clipboard restoration even after a prior command failure")
    finally:
        app.cleanup()

result = {"passed": True, "checks": checks, "actions": actions, "startups": startups,
          "caveats": ["AppKit status menu actions and scripted egui input; no focus helpers.",
                      "Physical global shortcut delivery and Korean IME require separate checks.",
                      "Login registration is opt-in and is not changed by this test.",
                      "Diagnostic timings are not an idle resource benchmark."]}
(output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"passed": True, "checks": checks, "evidence": str(output)}, ensure_ascii=False, indent=2))
