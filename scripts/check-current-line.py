#!/usr/bin/env python3
"""Capture caret-line feedback and note-switch focus in an isolated native window."""
import json
from pathlib import Path
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("current-line-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
app = None


def line(report):
    prefix = report["body"].encode("utf-16-le")[:report["selection"]["location"] * 2]
    return prefix.decode("utf-16-le").count("\n") + 1


try:
    with tempfile.TemporaryDirectory(prefix="quick-note-current-line-") as temporary:
        data = Path(temporary)
        app = Native(data, "caret", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        body = "\n".join(["# A thought worth keeping", "", "Small ideas have room here.",
                          "Capture it while it is fresh.", "", "One line at a time.", "",
                          "Keep writing.", "", "Ready when you are.", "", "Another idea.", ""])
        report = app.command("save thirteen-line fixture", "text", text=body)
        note_id = report["note_id"]
        assert line(report) == 13 and report["editor_focused"]
        app.command("light current line thirteen", "screenshot", name="line-13-helium")
        app.command("dark editor", "theme", id="graphite")
        app.command("dark current line thirteen", "screenshot", name="line-13-graphite")
        report = app.command("move one line up", key="up")
        assert line(report) == 12 and report["editor_focused"]
        app.command("dark current line twelve", "screenshot", name="line-12-graphite")
        app.command("click line four", "pointer", x=100, y=137, pressed=True)
        report = app.command("release on line four", "pointer", x=100, y=137, pressed=False)
        assert line(report) == 4 and report["body"] == body
        position = report["selection"]
        app.command("pointer-selected line four", "screenshot", name="line-4-pointer")
        checks.append("Keyboard and pointer movement select lines 13, 12 and 4 without changing the note; both shipped themes are captured")

        app.command("new note", key="n", command=True)
        app.command("save second note", "text", text="# Another thought\n")
        report = app.command("return with Ctrl Shift Tab", key="tab", control=True, shift=True)
        assert report["note_id"] == note_id and report["editor_focused"]
        assert report["selection"] == position and line(report) == 4
        app.command("restored working line", "screenshot", name="line-4-restored")
        checks.append("Ctrl Shift Tab restores the original UTF-16 caret and editor focus instead of traversing toolbar controls")

        report = app.command("move focus to search", key="p", command=True)
        assert not report["editor_focused"]
        app.command("search owns focus", "screenshot", name="search-focus")
        report = app.command("return to the working line", key="escape")
        assert report["editor_focused"] and line(report) == 4
        for text in ["ㅎ", "하", "한"]:
            report = app.command("Hangul preedit", "preedit", text=text)
            assert line(report) == 4
        report = app.command("commit Hangul once", "commit", text="한")
        assert report["body"].count("한") == 1 and line(report) == 4
        assert db_note(data, note_id)[0] == report["body"]
        app.command("committed Korean current line", "screenshot", name="line-4-hangul")
        checks.append("Search relinquishes editor focus; Escape and Korean composition retain the same working line and commit once")

        report = app.command("new empty draft", key="n", command=True)
        assert report["body"] == "" and line(report) == 1 and report["editor_focused"]
        app.command("empty draft current line", "screenshot", name="line-1-empty")
        app.command("delete empty fixture", key="backspace", command=True, shift=True)
        checks.append("A fresh empty draft has a focused first line")
        app.close()
        app = None
    (OUTPUT / "result.json").write_text(json.dumps({"checks": checks, "startups": startups,
        "actions": actions, "input": "Scripted egui events in real native windows; isolated data; not physical keyboard"},
        ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(OUTPUT), "checks_passed": len(checks)}, ensure_ascii=False))
finally:
    if app is not None:
        app.cleanup()
