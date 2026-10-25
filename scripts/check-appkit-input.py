#!/usr/bin/env python3
"""Post real AppKit events into an isolated window instead of bypassing native routing."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--binary", type=Path)
arguments = parser.parse_args()
if sys.platform != "darwin":
    parser.error("This check exercises macOS AppKit.")
if arguments.binary is None:
    subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                    "--bin", "native-check"], cwd=ROOT, check=True)
binary = (arguments.binary or ROOT / "target/release/native-check").resolve()
output = ROOT / ".artifacts" / ("appkit-input-" + time.strftime("%Y%m%d-%H%M%S"))
output.mkdir()
actions, startups, checks = [], [], []


def native(app, label, kind, **values):
    report = app.command(label, kind, **values)
    assert report["native_event_delivered"], (label, report["native_window"])
    assert not report["focus_helpers_enabled"]
    return report


with tempfile.TemporaryDirectory(prefix="quick-note-appkit-input-") as temporary:
    data = Path(temporary)
    app = Native(data, "input", output=output, binary=binary, actions=actions, startups=startups)
    try:
        first = app.command("Seed Alpha", "text", text="Alpha")["note_id"]
        app.command("Create Bravo", key="n", command=True)
        report = app.command("Seed Bravo", "text", text="Bravo")
        second = report["note_id"]
        window_frame = report["native_window"]["window_frame"]
        assert report["native_window"]["content_can_move_window"] is False
        for pressed in [True, False]:
            report = native(app, "AppKit click selects Alpha", "native_pointer", x=350, y=22, pressed=pressed)
        assert report["note_id"] == first and report["editor_focused"]
        report = native(app, "AppKit digit key reaches the editor", "native_key")
        assert report["body"] == "Alpha7" and not report["dirty"]
        checks.append("AppKit pointer dispatch selects a tab; native key dispatch commits exactly once")

        native(app, "AppKit press Bravo", "native_pointer", x=550, y=22, pressed=True)
        report = native(app, "AppKit drag Bravo before Alpha", "native_pointer", x=300, y=22, dragged=True)
        assert report["dragging_tab"] and report["pointer_down"]
        report = native(app, "AppKit drop commits tab order", "native_pointer", x=300, y=22, pressed=False)
        assert [tab["id"] for tab in report["tabs"]] == [second, first]
        assert report["native_window"]["window_frame"] == window_frame
        assert not report["dragging_tab"] and not report["pointer_down"] and report["editor_focused"]
        report = native(app, "Native typing still works after a tab drag", "native_key")
        assert report["body"] == "Alpha77" and db_note(data, first)[0] == "Alpha77"
        checks.append("Native tab drag changes order without moving the window or blocking subsequent typing")

        for pressed in [True, False]:
            native(app, "Click the note after the tab drag", "native_pointer", x=400, y=200, pressed=pressed)
        before = app.command("Read the current saved note", "report")["body"]
        report = native(app, "Native typing after the canvas click", "native_key")
        assert len(report["body"]) == len(before) + 1 and not report["dirty"]
        assert db_note(data, first)[0] == report["body"]
        checks.append("Canvas clicks and native typing remain usable after a tab drag")
        app.command("Native input proof", "screenshot", name="appkit-input-passed")
        app.close()
    finally:
        (output / "progress.json").write_text(json.dumps(actions, ensure_ascii=False, indent=2), encoding="utf-8")
        app.cleanup()

result = {"passed": True, "checks": checks, "actions": actions, "startups": startups,
          "input": "NSEvents posted through NSApplication and AppKit hit testing; isolated notes",
          "caveats": ["No physical mouse, accessibility automation or physical Korean IME is driven.",
                      "Missing mouse-up after native window tracking is covered by the headless window-drag regression.",
                      "Focus helpers run only for fixture startup, then stay disabled for native input checks."]}
(output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"passed": True, "checks": checks, "evidence": str(output)}, ensure_ascii=False, indent=2))
