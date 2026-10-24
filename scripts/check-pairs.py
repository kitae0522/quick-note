#!/usr/bin/env python3
"""Verify ordered bracket editing and durable Unicode cursor recovery in a native window."""
import json
from pathlib import Path
import signal
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("pairs-native-" + time.strftime("%Y%m%d-%H%M%S"))
BINARY = ROOT / "target/release/native-check"
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []


def launch(data, name):
    return Native(data, name, output=OUTPUT, binary=BINARY, actions=actions, startups=startups)


def saved(data, report, body, location, length=0):
    assert report["body"] == body and report["editor_focused"], report
    assert not report["dirty"] and report["error"] is None, report
    assert report["selection"] == {"location": location, "length": length}, report
    assert db_note(data, report["note_id"]) == (body, location, length)


with tempfile.TemporaryDirectory(prefix="quick-note-pairs-") as temporary:
    data = Path(temporary)
    app = launch(data, "01-edit")
    try:
        time.sleep(0.3)
        app.command("wake the idle native event loop", "report")
        checks.append("A fixture command wakes an idle native event loop and receives GPU readback")
        for opening, closing in [("(", ")"), ("[", "]"), ("{", "}")]:
            app.command("new blank draft", key="n", command=True)
            report = app.command("opening inserts its pair", "text", text=opening)
            saved(data, report, opening + closing, 1)
            report = app.command("empty pair Backspace deletes both", key="backspace")
            saved(data, report, "", 0)
            report = app.command("undo pair deletion", key="z", command=True)
            saved(data, report, opening + closing, 1)
            report = app.command("redo pair deletion", key="z", command=True, shift=True)
            saved(data, report, "", 0)
        checks.append("All three pairs place the caret inside and delete/undo/redo atomically")

        original = "한글 💡\ne\u0301"
        utf16_length = len(original.encode("utf-16-le")) // 2
        app.command("Unicode multiline source", "text", text=original)
        app.command("select the source", key="a", command=True)
        report = app.command("wrap selected source in braces", "text", text="{")
        saved(data, report, "{" + original + "}", 1, utf16_length)
        app.command("01-selection-wrapped-passed", "screenshot", name="01-selection-wrapped-passed")
        report = app.command("wrap the same selection again", "text", text="(")
        wrapped = "{(" + original + ")}"
        saved(data, report, wrapped, 2, utf16_length)
        report = app.command("undo the second wrap", key="z", command=True)
        saved(data, report, "{" + original + "}", 1, utf16_length)
        report = app.command("redo the second wrap", key="z", command=True, shift=True)
        saved(data, report, wrapped, 2, utf16_length)
        report = app.command("collapse to selected source end", key="right")
        saved(data, report, wrapped, 2 + utf16_length)
        for index, close in enumerate([")", "}"]):
            report = app.command("step past the existing closer", "text", text=close)
            saved(data, report, wrapped, 3 + utf16_length + index)
        checks.append("Unicode multiline wraps retain selection, undo and matching-closer movement")

        app.command("search has literal brackets", key="p", command=True)
        report = app.command("literal opening in search", "text", text="{")
        assert report["query"] == "{" and report["body"] == wrapped, report
        app.command("return to the editor", key="escape")
        app.command("new composition draft", key="n", command=True)
        for text in ["ㅎ", "하", "한"]:
            report = app.command("Korean composition remains literal", "preedit", text=text)
            assert report["body"] == text, report
        report = app.command("commit Korean syllable", "commit", text="한")
        saved(data, report, "한", 1)
        report = app.command("pair after Korean commit", "text", text="{")
        saved(data, report, "한{}", 2)
        report = app.command("delete just the empty pair", key="backspace")
        saved(data, report, "한", 1)
        app.command("IME-produced punctuation is literal", "preedit", text="(")
        report = app.command("commit literal punctuation", "commit", text="(")
        saved(data, report, "한(", 2)
        checks.append("Search and Korean preedit/commit remain literal; pairing resumes after commit")

        app.command("new recovery draft", key="n", command=True)
        app.command("opening before recovery", "text", text="(")
        report = app.command("Unicode inside pair", "text", text="한💡")
        saved(data, report, "(한💡)", 4)
        retained = report["note_id"]
        app.command("02-before-crash-passed", "screenshot", name="02-before-crash-passed")
        app.process.send_signal(signal.SIGKILL)
        app.process.wait(timeout=5)
        assert app.process.returncode == -signal.SIGKILL
        assert db_note(data, retained) == ("(한💡)", 4, 0)
    finally:
        app.cleanup()

    app = launch(data, "02-recovery")
    try:
        report = app.command("restore pair and Unicode cursor", "report")
        assert report["note_id"] == retained, report
        saved(data, report, "(한💡)", 4)
        report = app.command("restored closer is not duplicated", "text", text=")")
        saved(data, report, "(한💡)", 5)
        app.command("03-recovery-passed", "screenshot", name="03-recovery-passed")
        checks.append("SIGKILL preserves paired text and UTF-16 caret; next closer moves without duplication")
        app.close()
    finally:
        app.cleanup()

result = {"passed": True, "checks": checks, "actions": actions, "startups": startups,
          "input": "Scripted egui key/text/IME events inside a native GPU window",
          "caveats": ["Physical keyboard and macOS input-method callbacks are not driven here.",
                      "Windows execution is not verified by this macOS replay."]}
(OUTPUT / "result.json").write_text(json.dumps(result, indent=2, ensure_ascii=False), encoding="utf-8")
(OUTPUT / "assertions.md").write_text(
    "# Native bracket input checks\n\n" + "\n".join("- passed: " + check for check in checks)
    + "\n\nScripted native-window capture; not a physical-typing screen recording.\n",
    encoding="utf-8")
print(json.dumps({"passed": True, "checks": checks, "evidence": str(OUTPUT)}, indent=2))
