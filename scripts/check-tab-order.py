#!/usr/bin/env python3
"""Replay tab drags inside a native window using isolated notes and durable order checks."""
import atexit
import json
from pathlib import Path
import signal
import sqlite3
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("tab-order-native-" + time.strftime("%Y%m%d-%H%M%S"))
BINARY = ROOT / "target/release/native-check"
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
atexit.register(lambda: (OUTPUT / "progress.json").write_text(
    json.dumps({"actions": actions, "startups": startups}, indent=2, ensure_ascii=False), encoding="utf-8"))


def launch(data, name):
    app = Native(data, name, output=OUTPUT, binary=BINARY, actions=actions, startups=startups)
    time.sleep(0.3)
    return app


def tab_ids(report):
    return [tab["id"] for tab in report["tabs"]]


def stored_order(data):
    with sqlite3.connect(data / "notes.sqlite3") as connection:
        row = connection.execute("SELECT value FROM app_state WHERE key='tabOrder'").fetchone()
        return None if row is None else json.loads(row[0])


def press(app, x):
    app.command("point at a tab title", "pointer", x=x, y=20)
    report = app.command("press a tab title", "pointer", x=x, y=20, pressed=True)
    assert report["pointer_down"], report


def move(app, x, y=20):
    report = app.command("move the held tab", "pointer", x=x, y=y)
    assert report["dragging_tab"] and report["pointer_down"], report
    return report


def release(app, x, y=20):
    return app.command("drop the held tab", "pointer", x=x, y=y, pressed=False)


def scroll_edge(app, target):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        report = app.command("observe edge auto-scroll", "report")
        assert report["dragging_tab"] and report["pointer_down"], report
        if abs(report["tab_scroll"] - target) < 1:
            return report
        time.sleep(0.1)
    raise AssertionError(f"Edge scrolling did not reach {target}: {report}")


with tempfile.TemporaryDirectory(prefix="quick-note-tab-order-") as temporary:
    data = Path(temporary)
    app = launch(data, "01-drag")
    try:
        for index, title in enumerate(["Alpha 한글", "Bravo 💡", "Charlie café"]):
            if index:
                app.command("new fixture note", key="n", command=True)
            report = app.command("write fixture note", "text", text=title)
        original = tab_ids(report)
        report = app.command("activate first note", key="1", command=True)
        active, body, selection = report["note_id"], report["body"], report["selection"]
        press(app, 730)
        report = move(app, 286)
        assert report["dragging_tab"] and tab_ids(report) == original, report
        assert stored_order(data) is None
        app.command("01-insertion-passed", "screenshot", name="01-insertion-passed")
        report = release(app, 286)
        expected = [original[2], original[0], original[1]]
        assert tab_ids(report) == expected == stored_order(data), report
        assert report["note_id"] == active and report["body"] == body, report
        assert report["selection"] == selection and report["editor_focused"], report
        press(app, 540)
        move(app, 804)
        report = release(app, 804)
        expected = [original[2], original[1], original[0]]
        assert tab_ids(report) == expected == stored_order(data), report
        assert report["note_id"] == active and report["body"] == body, report
        checks.append("Active and inactive tab drags persist once on drop without changing text or caret")

        press(app, 360)
        move(app, 804)
        report = app.command("cancel the drag", key="escape")
        assert not report["dragging_tab"], report
        report = release(app, 804)
        assert tab_ids(report) == expected == stored_order(data), report
        press(app, 360)
        move(app, 500, 180)
        report = release(app, 500, 180)
        assert tab_ids(report) == expected == stored_order(data), report
        checks.append("Escape and dropping outside the tab strip cancel without saving")

        report = app.command("move the current tab left by keyboard", key="pageup", control=True, shift=True)
        assert tab_ids(report) == [original[2], original[0], original[1]], report
        assert report["note_id"] == active and report["editor_focused"], report
        report = app.command("move the current tab through AppKit", "menu", menu="Window", item="Move tab right")
        assert tab_ids(report) == expected == stored_order(data), report
        assert db_note(data, active) == (body, selection["location"], selection["length"])
        app.command("02-order-passed", "screenshot", name="02-order-passed")
        checks.append("Configured keyboard shortcuts and the native Window menu move the same tab")
        app.process.send_signal(signal.SIGKILL)
        app.process.wait(timeout=5)
        assert app.process.returncode == -signal.SIGKILL
    finally:
        app.cleanup()

    app = launch(data, "02-recovery")
    try:
        report = app.command("restore persisted tab placement", "report")
        assert tab_ids(report) == expected and report["note_id"] == active, report
        assert report["body"] == body and report["selection"] == selection, report
        checks.append("SIGKILL and restart restore order, active note and UTF-16 caret")
        app.close()
    finally:
        app.cleanup()

    with sqlite3.connect(data / "notes.sqlite3") as connection:
        timestamp = connection.execute("SELECT MAX(created_at) FROM notes").fetchone()[0]
        for index in range(20):
            connection.execute(
                "INSERT INTO notes(id,body,title,created_at,updated_at) VALUES(?,?,?,?,?)",
                (f"extra-{index}", f"Extra {index:02}", f"Extra {index:02}", timestamp + index + 1, timestamp + index + 1))
        connection.execute("UPDATE app_state SET value=? WHERE key='activeNote'", (expected[0],))
    app = launch(data, "03-overflow")
    try:
        report = app.command("start at the first overflowing tab", "report")
        original = tab_ids(report)
        active, body = report["note_id"], report["body"]
        press(app, 370)
        move(app, 802)
        maximum = len(original) * 146 - 2 - 490
        scroll_edge(app, maximum)
        app.command("03-edge-scroll-passed", "screenshot", name="03-edge-scroll-passed")
        report = release(app, 802)
        expected = original[1:] + original[:1]
        assert tab_ids(report) == expected == stored_order(data), report
        assert report["note_id"] == active and report["body"] == body, report
        press(app, 720)
        move(app, 316)
        scroll_edge(app, 0)
        report = release(app, 316)
        assert tab_ids(report) == original == stored_order(data), report
        checks.append("Both edges auto-scroll to offscreen tabs while the dragged source is virtualized out")

        with sqlite3.connect(data / "notes.sqlite3") as connection:
            connection.execute(
                "CREATE TRIGGER reject_order BEFORE INSERT ON app_state WHEN NEW.key='tabOrder' "
                "BEGIN SELECT RAISE(ABORT,'order write rejected'); END;")
        press(app, 370)
        move(app, 570)
        report = release(app, 570)
        assert report["error"] and tab_ids(report) == original == stored_order(data), report
        assert report["note_id"] == active and report["body"] == body, report
        report = app.command("continue writing after rejected placement", "text", text=" still editable")
        assert report["body"] == body + " still editable" and not report["dirty"], report
        assert db_note(data, active)[0] == report["body"]
        checks.append("Rejected order writes retain the prior order and leave note autosave usable")
        app.command("04-final-passed", "screenshot", name="04-final-passed")
        app.close()
    finally:
        app.cleanup()

result = {"passed": True, "checks": checks, "actions": actions, "startups": startups,
          "input": "Scripted pointer/key events inside a native GPU window; actual AppKit menu action",
          "caveats": ["Physical mouse input is not driven here.", "Windows runtime is not verified."]}
(OUTPUT / "result.json").write_text(json.dumps(result, indent=2, ensure_ascii=False), encoding="utf-8")
print(json.dumps({"passed": True, "checks": checks, "evidence": str(OUTPUT)}, indent=2))
