#!/usr/bin/env python3
"""Exercise quick capture and retained notes in a real Mac window with isolated data."""
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
output = ROOT / ".artifacts" / ("quick-capture-" + time.strftime("%Y%m%d-%H%M%S"))
output.mkdir()
actions, startups, checks = [], [], []


def hide_window(app, **key):
    app.sequence += 1
    (app.folder / "command.json").write_text(json.dumps({
        "sequence": app.sequence, "type": "key", **key}), encoding="utf-8")
    deadline = time.monotonic() + 3
    while frontmost_pid() == app.process.pid and time.monotonic() < deadline:
        time.sleep(0.01)
    assert app.process.poll() is None
    assert frontmost_pid() != app.process.pid


with tempfile.TemporaryDirectory(prefix="quick-note-capture-") as temporary:
    data = Path(temporary)
    app = Native(data, "capture", output=output, binary=binary, actions=actions,
                 startups=startups, arguments=["--quick-capture"])
    try:
        report = app.command("check OS shortcut registration", "report")
        assert report["capture_registered"], "OS shortcut registration must succeed"
        report = app.command("open quick capture", key="n", command=True, alt=True)
        assert report["capture_open"] and report["editor_focused"]
        assert not report["focus_helpers_enabled"]
        report = app.command("compact window size", "report")
        assert report["content_bounds"][2:] == [520.0, 340.0], report["content_bounds"]
        text = "# Inbox\n\nA café and a quiet thought. 한글 💡"
        report = app.command("save quick UTF-8 input", "text", text=text)
        first = report["note_id"]
        assert db_note(data, first)[0] == text and not report["dirty"]
        checks.append("OS shortcut registers; the existing editor opens compact and saves UTF-8")
        app.command("quick capture image", "screenshot", name="quick-capture")

        report = app.command("close tab without deleting", key="w", command=True)
        assert report["pending_delete"] is None and report["note_id"] != first
        with sqlite3.connect(data / "notes.sqlite3") as connection:
            assert connection.execute("SELECT closed,deleted_at FROM notes WHERE id=?", (first,)).fetchone() == (1, None)
        app.command("find closed note", key="p", command=True)
        app.command("search body", "text", text="CAFÉ")
        app.command("matching sentence image", "screenshot", name="search-context")
        report = app.command("reopen at match", key="enter")
        assert report["note_id"] == first and report["editor_focused"]
        selection = report["selection"]
        selected = text.encode("utf-16-le")[2 * selection["location"]:2 * (selection["location"] + selection["length"])]
        assert selected.decode("utf-16-le") == "café"
        checks.append("Cmd W retains the note; body search reopens it at the original UTF-16 match")

        app.command("delete explicitly", key="backspace", command=True, shift=True)
        app.command("select delete", key="tab")
        report = app.command("move to recently deleted", key="enter")
        assert db_note(data, first) is None
        with sqlite3.connect(data / "notes.sqlite3") as connection:
            assert connection.execute("SELECT body FROM notes WHERE id=?", (first,)).fetchone()[0] == text
        app.command("open recently deleted", "menu", menu="File", item="Recently deleted")
        report = app.command("restore the selected note", key="enter")
        assert report["note_id"] == first and report["body"] == text
        checks.append("Explicit deletion retains text; the native menu and search restore the same note")

        with sqlite3.connect(data / "notes.sqlite3") as connection:
            connection.execute("CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT,'fixture failure'); END;")
        report = app.command("simulate failed quick save", "text", text=" unsaved")
        assert report["dirty"]
        report = app.command("escape must keep the failed draft visible", key="escape")
        assert report["dirty"] and report["window_focused"] and "unsaved" in report["body"]
        app.command("quit also protects the failed draft", key="q", command=True)
        report = app.command("cancel blocked quit", key="enter")
        assert report["dirty"] and app.process.poll() is None
        with sqlite3.connect(data / "notes.sqlite3") as connection:
            connection.execute("DROP TRIGGER fail_save")
        report = app.command("retry durable save", key="s", command=True)
        assert not report["dirty"]
        assert db_note(data, first)[0] == report["body"]
        checks.append("Escape never hides a failed write; retry saves the retained draft")

        hide_window(app, key="escape")
        checks.append("Escape returns focus to another app while keeping Quick Note alive")
    finally:
        (output / "progress.json").write_text(json.dumps(actions, ensure_ascii=False, indent=2), encoding="utf-8")
        app.cleanup()

with tempfile.TemporaryDirectory(prefix="quick-note-capture-reload-") as temporary:
    data = Path(temporary)
    path = data / "shortcut/bindings.json"
    path.parent.mkdir()
    path.write_text(json.dumps({"schemaVersion": 1, "bindings": {
        "quick_capture": ["Primary+Alt+BrowserBack"]}}), encoding="utf-8")
    app = Native(data, "registration-recovery", output=output, binary=binary,
                 actions=actions, startups=startups, arguments=["--quick-capture"])
    try:
        assert not app.command("check failed registration", "report")["capture_registered"]
        path.write_text('{"schemaVersion":1,"bindings":{}}', encoding="utf-8")
        report = app.command("retry registration after correcting shortcuts", "menu",
                             menu="Help", item="Reload shortcuts")
        assert report["capture_registered"]
        checks.append("A failed startup registration can be corrected and reloaded without restarting")
        app.command("save before quit recovery", "text", text="Keep this note")
        with sqlite3.connect(data / "notes.sqlite3") as connection:
            connection.execute("CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT,'fixture failure'); END;")
        assert app.command("fail a save before quit", "text", text=" with recovery")["dirty"]
        app.command("blocked quit", key="q", command=True)
        assert app.command("keep editing instead of quitting", key="enter")["dirty"]
        with sqlite3.connect(data / "notes.sqlite3") as connection:
            connection.execute("DROP TRIGGER fail_save")
        assert not app.command("save after canceling quit", key="s", command=True)["dirty"]
        hide_window(app, key="w", command=True, shift=True)
        checks.append("Canceling a blocked Quit restores window-close hiding and keeps capture available")
    finally:
        app.cleanup()

result = {"passed": True, "checks": checks, "actions": actions, "startups": startups,
          "caveats": ["Commands are scripted egui input inside a real AppKit window, with focus helpers disabled.",
                      "Shortcut registration is verified; physical global hotkey delivery from another app is not.",
                      "Timing includes the diagnostic command driver and GPU readback, not an idle resource benchmark.",
                      "Physical Korean IME and Windows behavior require separate checks."]}
(output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"passed": True, "checks": checks, "evidence": str(output)}, ensure_ascii=False, indent=2))
