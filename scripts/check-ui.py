#!/usr/bin/env python3
"""Drive the real Rust window with scripted egui input, then test abrupt recovery."""
import json
from pathlib import Path
import signal
import sqlite3
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("native-rust-" + time.strftime("%Y%m%d-%H%M%S"))
BINARY = ROOT / "target/release/native-check"
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--release", "--features", "diagnostics", "--bin", "native-check"], cwd=ROOT, check=True)
checks, actions, startups = [], [], []


from functools import partial
from native_fixture import Native as NativeWindow
from native_fixture import db_note
from native_fixture import read_json

Native = partial(NativeWindow, output=OUTPUT, binary=BINARY, actions=actions, startups=startups)


try:
    with tempfile.TemporaryDirectory(prefix="quick-note-rust-") as temporary:
        data = Path(temporary)
        app = Native(data, "01-capture")
        try:
            text = "# Capture\n\n아이디어 💡 café e\u0301\n\n- [ ] Keep it simple\n"
            report = app.command("capture and durable UTF-8 save", "text", text=text)
            first = report["note_id"]
            assert report["body"] == text and not report["dirty"]
            assert db_note(data, first)[0] == text
            checks.append("UTF-8 input commits before any close event")
            report = app.command("new note", key="n", command=True)
            assert len(report["tabs"]) == 2 and report["body"] == ""
            second = report["note_id"]
            app.command("second note save", "text", text="# Second thought\n\nA quieter place to think.")
            report = app.command("Markdown rendering", key="m", command=True, shift=True)
            assert report["mode"] == "preview"
            app.command("open search", key="p", command=True)
            app.command("search across note bodies", "text", text="CAFÉ")
            report = app.command("open search result", key="enter")
            assert report["note_id"] == first
            report = app.command("show delete confirmation", key="backspace", command=True, shift=True)
            assert report["pending_delete"] == first and db_note(data, first)
            report = app.command("Enter defaults to Cancel", key="enter")
            assert report["pending_delete"] is None and db_note(data, first)
            app.command("show confirmation again", key="backspace", command=True, shift=True)
            app.command("focus Delete note", key="tab")
            report = app.command("confirm deletion", key="enter")
            assert db_note(data, first) is None and report["note_id"] == second
            checks += ["Markdown switch and full-text search", "deletion waits for confirmation; Enter cancels by default", "confirmed deletion preserves the next note"]
            app.command("new capture before abrupt exit", key="n", command=True)
            final = "# Recovery\n\n마지막 글자 💡 café e\u0301 END"
            report = app.command("save the final character", "text", text=final)
            retained = report["note_id"]
            app.command("move cursor before crash", key="left")
            expected = db_note(data, retained)
            assert expected[0] == final
            app.process.send_signal(signal.SIGKILL)
            app.process.wait(timeout=5)
            assert app.process.returncode == -signal.SIGKILL
            assert db_note(data, retained) == expected
            checks.append("SIGKILL preserves the last committed character and cursor")
        finally:
            app.cleanup()

        app = Native(data, "02-recovery")
        try:
            report = app.command("restore the last note after SIGKILL", "report")
            assert report["note_id"] == retained and report["body"] == final
            assert db_note(data, retained) == expected
            assert (report["selection"]["location"], report["selection"]["length"]) == expected[1:]
            checks.append("a new native process restores the same note and UTF-16 cursor")
            with sqlite3.connect(data / "notes.sqlite3") as database:
                database.execute("CREATE TRIGGER fail_write BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT, 'verification failure'); END;")
            report = app.command("failed save keeps editor text", "text", text=" UNSAVED")
            assert report["dirty"] and "UNSAVED" in report["body"] and db_note(data, retained)[0] == final
            report = app.command("failed save blocks note switching", key="n", command=True)
            assert report["note_id"] == retained and report["dirty"]
            app.command("failed save cancels window close", key="w", command=True, shift=True)
            report = app.command("show save recovery dialog", "report")
            assert app.process.poll() is None and report["dirty"]
            report = app.command("keep editing after failed close", key="escape")
            unsaved_body = report["body"]
            app.process.send_signal(signal.SIGKILL)
            app.process.wait(timeout=5)
            app.cleanup()
            app = Native(data, "03-recover-failed-write")
            report = app.command("restore recovery copy after failed SQL write and SIGKILL", "report")
            assert report["body"] == unsaved_body and report["dirty"]
            assert db_note(data, retained)[0] == final
            checks.append("a failed SQL write also survives SIGKILL through the UTF-8 recovery copy")
            with sqlite3.connect(data / "notes.sqlite3") as database:
                database.execute("DROP TRIGGER fail_write;")
            report = app.command("retry saving", key="s", command=True)
            assert not report["dirty"] and db_note(data, retained)[0] == report["body"]
            assert not (data / "recovery.json").exists()
            checks.append("failed writes retain text, block switching and close, and recover on retry")
            app.close()
        finally:
            app.cleanup()

        # Fixed fixture workload; personal notes never enter this benchmark.
        with sqlite3.connect(data / "notes.sqlite3") as database:
            for index in range(50):
                body = f"# Idea {index + 1:03}\n\n" + "A small idea, saved immediately. " * 32 + f"\nmarker-{index:03}"
                database.execute("INSERT INTO notes(id,body,created_at,updated_at,selection_location,selection_length,title) VALUES (?,?,?,?,?,?,?)", (f"fixture-{index}", body, 1000 + index, 1000 + index, 0, 0, f"Idea {index + 1:03}"))
        for iteration in range(5):
            app = Native(data, f"03-startup-52-notes-{iteration + 1}")
            try:
                if iteration == 0:
                    app.command("search 52 saved notes", key="p", command=True)
                    app.command("full-text query over 52 notes", "text", text="marker-049")
                    report = app.command("open the matched note", key="enter")
                    assert report["note_id"] == "fixture-49"
                app.close()
            finally:
                app.cleanup()

    result = {
        "passed": True, "checks": checks, "startups": startups, "actions": actions,
        "budget_200_ms": {"actions_passed": all(item["frame_captured_ms"] < 200 for item in actions), "startup_passed": all(item["first_frame_captured_ms"] < 200 for item in startups)},
        "measurement": "main entry to native framebuffer readback with keyboard focus; action event to framebuffer readback, including synchronous SQLite commit where applicable",
        "caveats": ["scripted egui input in a native GPU window", "OS/library caches were not flushed", "loader time before main and physical key delivery are excluded", "Windows runtime, physical Korean IME, VoiceOver, power loss, and universal 200 ms guarantees are not verified"],
    }
    (OUTPUT / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps({"passed": True, "checks": len(checks), "startup_ms": [round(item["first_frame_captured_ms"], 2) for item in startups], "maximum_action_ms": round(max(item["frame_captured_ms"] for item in actions), 2), "evidence": str(OUTPUT)}, ensure_ascii=False, indent=2))
except BaseException as error:
    (OUTPUT / "failure.json").write_text(json.dumps({"error": str(error), "checks": checks, "startups": startups, "actions": actions}, ensure_ascii=False, indent=2), encoding="utf-8")
    raise
