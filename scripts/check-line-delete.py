#!/usr/bin/env python3
"""Check single-press line deletion, composed-text undo and UTF-8 durability in a native window."""
import json
from pathlib import Path
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("line-delete-native-" + time.strftime("%Y%m%d-%H%M%S"))
BINARY = ROOT / "target/release/native-check"
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []


def launch(data, name):
    return Native(data, name, output=OUTPUT, binary=BINARY, actions=actions, startups=startups)


def saved(data, report, body):
    assert report["body"] == body and report["editor_focused"], report
    assert not report["dirty"] and report["error"] is None, report
    stored, location, length = db_note(data, report["note_id"])
    assert stored == body and {"location": location, "length": length} == report["selection"]


with tempfile.TemporaryDirectory(prefix="quick-note-line-delete-") as temporary:
    data = Path(temporary)
    app = launch(data, "01-delete")
    try:
        time.sleep(0.3)
        original = "Keep this\nPrefix 한글 💡 café"
        report = app.command("write a Unicode line", "text", text=original)
        note_id = report["note_id"]
        report = app.command("one Command Backspace deletes the line", key="backspace", command=True)
        saved(data, report, "Keep this\n")
        assert report["selection"] == {"location": 10, "length": 0}, report
        report = app.command("undo restores the visible Unicode line", key="z", command=True)
        saved(data, report, original)
        report = app.command("redo is one deletion", key="z", command=True, shift=True)
        saved(data, report, "Keep this\n")
        checks.append("One Cmd+Backspace deletes the line; Unicode text and caret survive undo/redo")

        app.command("new composition fixture", key="n", command=True)
        prefix = "Keep this\nPrefix "
        app.command("write the committed prefix", "text", text=prefix)
        for text in ["ㅎ", "하", "한", "하", "ㅎ", "한"]:
            report = app.command("replace only the active composition", "preedit", text=text)
            assert report["body"] == prefix + text, report
        app.command("commit before editor-owned deletion", "commit", text="한")
        report = app.command("delete the completed line once", key="backspace", command=True)
        saved(data, report, "Keep this\n")
        app.command("01-first-press-passed", "screenshot", name="01-first-press-passed")
        report = app.command("undo includes the last composed syllable", key="z", command=True)
        saved(data, report, prefix + "한")
        report = app.command("redo the composed line deletion", key="z", command=True, shift=True)
        saved(data, report, "Keep this\n")
        retained = report["note_id"]
        checks.append("Preedit corrections preserve the prefix; committed syllable survives line-delete undo")

        app.command("open search", key="p", command=True)
        app.command("search prefix", "text", text="Prefix ")
        app.command("search preedit", "preedit", text="한")
        app.command("search commit", "commit", text="한")
        report = app.command("one deletion clears the search query", key="backspace", command=True)
        assert not report["query"] and report["body"] == "Keep this\n", report
        report = app.command("search undo retains its composed syllable", key="z", command=True)
        assert report["query"] == "Prefix 한" and report["body"] == "Keep this\n", report
        app.command("return to writing", key="escape")
        report = app.command("type after deletion", "text", text="Next line")
        saved(data, report, "Keep this\nNext line")
        app.command("02-continue-writing-passed", "screenshot", name="02-continue-writing-passed")
        checks.append("Search deletion stays in search and the editor accepts the next text immediately")
        app.close()
    finally:
        app.cleanup()

    app = launch(data, "02-recovery")
    try:
        report = app.command("restore the edited note", "report")
        assert report["note_id"] == retained, report
        saved(data, report, "Keep this\nNext line")
        assert db_note(data, note_id)[0] == "Keep this\n"
        app.command("03-recovery-passed", "screenshot", name="03-recovery-passed")
        checks.append("Restart restores the saved body, UTF-16 caret and untouched neighboring note")
        app.close()
    finally:
        app.cleanup()

result = {"passed": True, "checks": checks, "actions": actions, "startups": startups,
          "input": "Scripted egui text/key/IME events inside a native GPU window",
          "caveats": ["This replay does not drive the macOS input-method process or a physical keyboard.",
                      "Native key posting was unavailable; exact backend routing is covered by ime_policies tests.",
                      "Windows runtime is not verified."]}
(OUTPUT / "result.json").write_text(json.dumps(result, indent=2, ensure_ascii=False), encoding="utf-8")
print(json.dumps({"passed": True, "checks": checks, "evidence": str(OUTPUT)}, indent=2))
