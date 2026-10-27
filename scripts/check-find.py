#!/usr/bin/env python3
"""Verify current-note find, atomic replacements and undo in isolated native Mac windows."""
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("find-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
BINARY = ROOT / "target/release/native-check"
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []


def utf16(text):
    return len(text.encode("utf-16-le")) // 2


def resize(app, width, height):
    app.command("resize the find panel", "resize", width=width, height=height)
    for _ in range(30):
        report = app.command("wait for the requested size", "report")
        if report["content_bounds"] == [0.0, 0.0, float(width), float(height)]:
            return
    raise AssertionError("Native find window did not reach the requested size")


def click(app, label, x, y):
    app.command(label + " hover", "pointer", x=x, y=y)
    app.command(label + " press", "pointer", x=x, y=y, pressed=True)
    return app.command(label + " release", "pointer", x=x, y=y, pressed=False)


with tempfile.TemporaryDirectory(prefix="quick-note-find-") as temporary:
    data = Path(temporary)
    app = Native(data, "english", output=OUTPUT, binary=BINARY, actions=actions, startups=startups)
    try:
        body = "Café 한글 💡 한글\n한글 notes\n"
        first = app.command("seed UTF-8 text", "text", text=body)["note_id"]
        app.command("create the control note", key="n", command=True)
        second = app.command("seed the control note", "text", text="Keep this note unchanged.")["note_id"]
        app.command("return to the first note", key="1", command=True)
        app.command("find within the current note", key="f", command=True)
        report = app.command("search Hangul literally", "text", text="한글")
        assert report["selection"] == {"location":utf16("Café "), "length":2}
        report = app.command("advance with Enter", key="enter")
        assert report["selection"] == {"location":utf16("Café 한글 💡 "), "length":2}
        report = app.command("return with Shift+Enter", key="enter", shift=True)
        assert report["selection"] == {"location":utf16("Café "), "length":2}
        assert report["body"] == body
        app.command("find panel", "screenshot", name="01-find")
        checks.append("Cmd+F and Enter/Shift+Enter navigate current-note UTF-16 selections")

        app.command("open replacement", key="f", command=True, alt=True)
        for _ in range(6):
            app.command("reach replacement by Tab", key="tab")
        replacement = "새($1)💡"
        app.command("enter literal replacement", "text", text=replacement)
        app.command("reach Replace", key="tab")
        report = app.command("replace one match", key="enter")
        one = body.replace("한글", replacement, 1)
        assert report["body"] == one and not report["dirty"]
        app.command("reach Replace all", key="tab")
        report = app.command("replace the remaining matches", key="enter")
        expected = body.replace("한글", replacement)
        assert report["body"] == expected and db_note(data, first)[0] == expected
        assert db_note(data, second)[0] == "Keep this note unchanged."
        app.command("replace panel", "screenshot", name="02-replaced")
        checks.append("Tab reaches Replace/Replace all; literal replacements save only the active note")

        app.command("return to writing", key="escape")
        report = app.command("undo Replace all once", key="z", command=True)
        assert report["body"] == one
        report = app.command("undo the single replacement", key="z", command=True)
        assert report["body"] == body
        for _ in range(2):
            report = app.command("redo the replacement", key="z", command=True, shift=True)
        assert report["body"] == expected and db_note(data, first)[0] == expected
        checks.append("Each replacement is one undo/redo step and remains automatically saved")

        app.command("find before the Preview click", key="f", command=True)
        report = app.command("find the unchanged prefix", "text", text="Café")
        report = click(app, "click footer Preview", 90, report["content_bounds"][3] - 16)
        assert report["mode"] == "preview" and report["body"] == expected
        app.command("Preview after Find", "screenshot", name="04-preview")
        report = app.command("reopen Find from Preview", key="g", command=True)
        assert report["mode"] == "edit" and report["body"] == expected
        app.command("close the reopened Find panel", key="escape")
        checks.append("Footer Preview closes Find; Cmd+G reopens source results without changing text")

        app.command("find before clicking header search", key="f", command=True)
        click(app, "click header search", 160, 22)
        report = app.command("find the control note", "text", text="Keep")
        assert report["query"] == "Keep" and report["note_id"] == first
        app.command("header search after Find", "screenshot", name="05-global-search")
        report = app.command("dismiss search once", key="escape")
        assert report["editor_focused"] and report["body"] == expected
        app.command("move to the line end", key="right", command=True)
        report = app.command("Enter writes instead of opening the search result", key="enter")
        assert report["note_id"] == first and len(report["body"]) == len(expected) + 1
        expected = report["body"]
        assert db_note(data, first)[0] == expected
        assert db_note(data, second)[0] == "Keep this note unchanged."
        app.command("one Escape returns to writing", "screenshot", name="06-search-dismissed")
        checks.append("Header search closes Find; one Escape restores writing and Enter stays in the note")

        app.command("search for the control note again", key="p", command=True)
        report = app.command("open the selected search result", key="enter")
        assert report["note_id"] == second and report["editor_focused"]
        assert report["body"] == "Keep this note unchanged."
        app.command("return to the first note", key="1", command=True)
        assert app.command("read the preserved first note", "report")["body"] == expected
        checks.append("Opening a search result restores its editor without reactivating header search")
        app.close()
    finally:
        app.cleanup()

    with sqlite3.connect(data / "notes.sqlite3") as database:
        database.executemany("INSERT OR REPLACE INTO app_state(key,value) VALUES(?,?)",
                             [("language", "ko"), ("theme", "graphite"),
                              ("editorFont", json.dumps(["SF Pro", "Pretendard", "monospace", "Noto Sans CJK"]))])
    app = Native(data, "korean", output=OUTPUT, binary=BINARY, actions=actions, startups=startups)
    try:
        assert app.command("read the restored note", "report")["body"] == expected
        resize(app, 320, 240)
        app.command("open Korean replace panel", key="f", command=True, alt=True)
        app.command("find the literal replacement", "text", text=replacement)
        app.command("settled Korean find panel", "report")
        app.command("minimum Korean panel", "screenshot", name="03-minimum-korean")
        report = app.command("next match through configured shortcut", key="g", command=True)
        assert report["body"] == expected and report["selection"]["length"] == utf16(replacement)
        app.command("close the find panel", key="escape")
        report = app.command("typing after find still works", "text", text="!")
        assert report["body"].count("!") == 1 and not report["dirty"]
        checks.append("Restart retains UTF-8 text; the 320×240 Korean panel returns to usable editing")
        app.close()
    finally:
        app.cleanup()

result = {"passed":True, "checks":checks, "actions":actions, "startups":startups,
          "input":"Scripted egui events in native Mac windows; isolated UTF-8 notes",
          "unverified":["Physical Korean IME", "Physical VoiceOver", "Windows runtime"]}
(OUTPUT / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"checks_passed":len(checks), "output":str(OUTPUT)}, ensure_ascii=False))
