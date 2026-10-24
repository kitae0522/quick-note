#!/usr/bin/env python3
"""Replay canvas, Korean composition and keyboard flows in an isolated native window."""
import json
from pathlib import Path
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
HEADER_HEIGHT = json.loads((ROOT / "assets/themes/helium.json").read_text(encoding="utf-8"))["layout"]["toolbarHeight"]
OUTPUT = ROOT / ".artifacts" / ("input-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
app = None
try:
    with tempfile.TemporaryDirectory(prefix="quick-note-input-") as temporary:
        data = Path(temporary)
        app = Native(data, "input", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        for index, (x, y) in enumerate([(2, 100), (918, 100), (400, 550), (2, 550), (400, HEADER_HEIGHT + 2)]):
            app.command("leave editor for search", key="p", command=True)
            for pressed in [True, False]:
                report = app.command("click note canvas", "pointer", x=x, y=y, pressed=pressed)
            assert report["editor_focused"], (x, y, report)
            marker = f"[click-{index}]"
            report = app.command("type at clicked location", "text", text=marker)
            assert marker in report["body"] and not report["dirty"]
        checks.append("All five canvas edge/blank locations focus the note and accept durable input")

        app.command("new note", key="n", command=True)
        for text in ["ㅎ", "하", "한"]:
            report = app.command("Korean preedit", "preedit", text=text)
            assert report["body"] == text, report
        app.command("clear preedit", "preedit", text="")
        report = app.command("commit syllable", "commit", text="한")
        assert report["body"] == "한"
        app.command("space after commit", "text", text=" ")
        app.command("next preedit", "preedit", text="한")
        report = app.command("repeat syllable", "commit", text="한")
        assert report["body"] == "한 한"
        report = app.command("legitimate repeated characters", "text", text="  aa")
        assert report["body"] == "한 한  aa"
        assert db_note(data, report["note_id"])[0] == report["body"]
        checks.append("Korean preedit replacement, commit, spaces and legitimate repeats persist exactly")

        report = app.command("new indentation draft", key="n", command=True)
        indented = report["note_id"]
        report = app.command("Tab inserts four spaces", key="tab")
        assert report["editor_focused"] and report["body"] == "    "
        report = app.command("undo indentation", key="z", command=True)
        assert report["body"] == "" and report["editor_focused"]
        report = app.command("redo indentation", key="z", command=True, shift=True)
        assert report["body"] == "    "
        app.command("type after indentation", "text", text="An idea 한글 💡")
        report = app.command("Shift Tab unindents", key="tab", shift=True)
        assert report["body"] == "An idea 한글 💡" and report["editor_focused"]
        kept_body = report["body"]
        report = app.command("F6 moves to tabs", key="f6")
        assert not report["editor_focused"]
        report = app.command("Tab navigates outside the editor", key="tab")
        assert not report["editor_focused"] and report["body"] == kept_body
        report = app.command("Shift F6 returns to writing", key="f6", shift=True)
        assert report["editor_focused"]
        app.command("Ctrl Tab switches notes", key="tab", control=True)
        report = app.command("Ctrl Shift Tab restores the draft", key="tab", control=True, shift=True)
        assert report["note_id"] == indented and report["editor_focused"]
        assert report["body"] == kept_body and db_note(data, indented)[0] == kept_body
        checks.append("Tab inserts four spaces, Shift Tab unindents, undo/redo preserve text, and F6 provides keyboard access outside the editor")
        checks.append("Ctrl Tab and Ctrl Shift Tab switch notes without inserting indentation or losing editor focus")

        app.command("new searchable note", key="n", command=True)
        report = app.command("target text", "text", text="Find target 아이디어")
        target = report["note_id"]
        report = app.command("first note shortcut", key="1", command=True)
        original = report["note_id"]
        app.command("open search", key="p", command=True)
        app.command("compose search query", "preedit", text="아이디어")
        report = app.command("Enter during preedit", key="enter")
        assert report["note_id"] == original
        app.command("commit search query", "commit", text="아이디어")
        report = app.command("open matched note", key="enter")
        assert report["note_id"] == target, report
        checks.append("Search leaves Enter to Korean composition, then opens the result after commit")

        app.command("open commands", key="p", command=True, shift=True)
        app.command("options screenshot", "screenshot", name="options")
        app.command("open shortcut help", key="slash", command=True)
        app.command("help screenshot", "screenshot", name="shortcuts")
        report = app.command("return to writing", key="escape")
        assert report["editor_focused"]
        report = app.command("focus tabs", key="f6")
        assert not report["editor_focused"]
        report = app.command("return from tabs", key="escape")
        assert report["editor_focused"]
        report = app.command("switch dark appearance", key="l", command=True, shift=True)
        assert report["theme"] == "dark"
        app.command("switch light appearance", key="l", command=True, shift=True)
        app.command("final editor screenshot", "screenshot", name="after")
        checks.append("Options/help, focus cycle, Escape and appearance shortcuts work in a native window")
        kept = app.command("report current note", "report")
        app.command("nonempty note still asks", key="w", command=True)
        report = app.command("Cancel remains the default", key="enter")
        assert report["note_id"] == kept["note_id"] and report["pending_delete"] is None
        for blank_text in ["", " \t\n\u3000"]:
            report = app.command("create blank note", key="n", command=True)
            blank = report["note_id"]
            if blank_text:
                app.command("whitespace only", "text", text=blank_text)
            report = app.command("delete blank without confirmation", key="w", command=True)
            assert report["pending_delete"] is None and db_note(data, blank) is None
            assert report["note_id"] == kept["note_id"] and report["body"] == kept["body"]
            assert report["editor_focused"]
        checks.append("Empty and whitespace notes delete immediately; content still defaults to Cancel")
        app.close()
        app = None
except Exception:
    if app is not None:
        app.cleanup()
    raise

result = {"passed": True, "checks": checks, "actions": actions, "startups": startups,
          "input": "Scripted pointer/key/IME events inside a native GPU window",
          "caveats": ["Native AppKit callback policies are tested separately in tests/ime_policies.rs.",
                      "These replays do not exercise the macOS input-method process or physical typing."]}
(OUTPUT / "result.json").write_text(json.dumps(result, indent=2, ensure_ascii=False), encoding="utf-8")
print(json.dumps({"passed": True, "checks": checks, "evidence": str(OUTPUT)}, indent=2))
