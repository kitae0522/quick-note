#!/usr/bin/env python3
"""Verify editable app shortcuts in native windows using isolated UTF-8 notes."""
import json
from pathlib import Path
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("shortcuts-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
app = None


def write_bindings(path, bindings):
    path.write_text(json.dumps({"schemaVersion": 1, "bindings": bindings},
                              ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


try:
    with tempfile.TemporaryDirectory(prefix="quick-note-shortcuts-") as temporary:
        data = Path(temporary)
        folder = data / "shortcut"
        folder.mkdir()
        path = folder / "bindings.json"
        bindings = {"new_note": ["Primary+Shift+N"], "search_notes": ["Primary+K"],
                    "reload_shortcuts": ["Primary+Shift+S"]}
        write_bindings(path, bindings)
        app = Native(data, "custom", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        body = "# Custom capture\n\n커스텀 단축키 💡 café"
        report = app.command("save original UTF-8 note", "text", text=body)
        original = report["note_id"]
        report = app.command("old New note key is unassigned", key="n", command=True)
        assert report["note_id"] == original and report["body"] == body
        report = app.command("custom New note key", key="n", command=True, shift=True)
        second = report["note_id"]
        assert second != original and report["editor_focused"]
        report = app.command("Tab still belongs to writing", key="tab")
        assert report["body"] == "    " and report["editor_focused"]
        app.command("custom search key", key="k", command=True)
        app.command("compose custom search", "preedit", text="커스텀")
        report = app.command("IME owns Enter", key="enter")
        assert report["note_id"] == second
        app.command("commit search text", "commit", text="커스텀")
        report = app.command("open matching original note", key="enter")
        assert report["note_id"] == original and report["body"] == body
        report = app.command("old search binding is removed", key="p", command=True)
        assert report["editor_focused"]
        checks.append("Configured keys replace old aliases and preserve Tab indentation, Korean search composition and UTF-8 note data")

        app.command("open keyboard settings", key="slash", command=True)
        app.command("settled keyboard settings", "report")
        app.command("capture live bindings", "screenshot", name="custom-keyboard-settings")
        app.command("return to writing", key="escape")
        bindings["new_note"] = ["Primary+Shift+K"]
        bindings["toggle_preview"] = []
        write_bindings(path, bindings)
        app.command("apply edits through configured reload", key="s", command=True, shift=True)
        report = app.command("previous custom key is removed", key="n", command=True, shift=True)
        assert report["note_id"] == original
        report = app.command("new custom key takes effect without restart", key="k", command=True, shift=True)
        current = report["note_id"]
        assert current != original and report["editor_focused"]
        report = app.command("disabled preview key", key="m", command=True, shift=True)
        assert report["mode"] == "edit"
        checks.append("An assigned Reload shortcuts action applies replacement keys and disabled actions without restarting")

        bindings["new_note"] = ["Primary+W"]
        write_bindings(path, bindings)
        invalid = path.read_bytes()
        app.command("reject conflicting file", key="s", command=True, shift=True)
        app.command("capture conflict notice", "screenshot", name="shortcut-conflict")
        report = app.command("last valid custom key still works", key="k", command=True, shift=True)
        assert report["note_id"] != current and report["editor_focused"]
        assert path.read_bytes() == invalid and db_note(data, original)[0] == body
        checks.append("Conflicts retain the entire last valid binding set and leave the configuration and original notes untouched")
        app.close()
        app = None

        app = Native(data, "invalid-startup", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        before = app.command("defaults remain available at startup", "report")
        report = app.command("default New note key remains usable", key="n", command=True)
        assert report["note_id"] != before["note_id"] and report["editor_focused"]
        assert path.read_bytes() == invalid and db_note(data, original)[0] == body
        app.command("invalid startup notice", "screenshot", name="default-bindings-notice")
        app.close()
        app = None
        checks.append("An invalid startup file shows a notice and keeps default commands and saved notes available")
    (OUTPUT / "result.json").write_text(json.dumps({"checks": checks, "startups": startups,
        "actions": actions, "input": "Scripted egui events in native macOS windows; isolated data; not physical keyboard"},
        ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(OUTPUT), "checks_passed": len(checks)}, ensure_ascii=False))
finally:
    if app is not None:
        app.cleanup()
