#!/usr/bin/env python3
"""Exercise real AppKit menu items and key equivalents with isolated UTF-8 notes."""
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("menu-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
text = json.loads((ROOT / "i18n/en.json").read_text())
actions, startups, checks = [], [], []
app = None


def item(report, menu, title):
    group = next(group for group in report["system_menu"] if group["title"] == menu)
    return next(child for child in group["items"] if child["title"] == title)


def choose(menu, title, clipboard=False):
    return app.command(menu + " / " + title, "menu", menu=menu, item=title, clipboard=clipboard)


try:
    with tempfile.TemporaryDirectory(prefix="quick-note-menu-") as temporary:
        data = Path(temporary)
        app = Native(data, "system-menu", output=OUTPUT,
                     binary=ROOT / "target/release/native-check", actions=actions, startups=startups)
        report = app.command("read actual AppKit menu", "report")
        assert [group["title"] for group in report["system_menu"]][1:] == ["File", "Edit", "View", "Window", "Help"]
        app_menu = report["system_menu"][0]["title"]
        count = len(report["tabs"])
        report = app.command("native Command N equivalent", "menu_key", key="n")
        assert report["menu_key_handled"] and len(report["tabs"]) == count + 1
        assert report["editor_focused"] and report["body"] == ""
        checks.append("The real AppKit menu contains every group and Command N creates exactly one focused note")

        body = "Native menu · 한글 💡 café"
        report = app.command("write a saved fixture", "text", text=body)
        note_id = report["note_id"]
        choose("Edit", "Select all")
        choose("Edit", "Copy", clipboard=True)
        choose("Edit", "Cut", clipboard=True)
        report = app.command("cut completed", "report")
        assert report["body"] == ""
        choose("Edit", "Paste", clipboard=True)
        report = app.command("paste completed", "report")
        assert report["body"] == body and db_note(data, note_id)[0] == body
        report = choose("Edit", "Undo")
        assert report["body"] == ""
        report = choose("Edit", "Redo")
        assert report["body"] == body
        app.command("restore every original clipboard representation", "restore_clipboard")
        checks.append("Edit menu Select all, Copy, Cut, Paste, Undo and Redo edit the focused UTF-8 draft")

        report = choose("Window", text["shortcut_next"])
        assert report["note_id"] != note_id
        report = choose("Window", text["shortcut_previous"])
        assert report["note_id"] == note_id and report["body"] == body
        report = choose("Help", text["keyboard_shortcuts"])
        assert report["settings_open"]
        app.command("return from Help", key="escape")

        report = choose("View", "Preview")
        assert report["mode"] == "preview"
        report = app.command("preview menu state", "report")
        assert not item(report, "Edit", "Undo")["enabled"]
        choose("View", "Edit")
        report = choose("View", text["editor_font"])
        assert report["settings_open"] and report["body"] == body
        app.command("return from font settings", key="escape")
        report = choose("File", text["shortcut_delete"])
        assert report["pending_delete"] == note_id
        assert not item(report, "File", "New note")["enabled"]
        report = app.command("deletion still defaults to Cancel", key="enter")
        assert report["pending_delete"] is None and report["body"] == body
        checks.append("View/font settings preserve the draft; nonempty deletion confirms and blocks other menu commands")

        choose(app_menu, "Settings…")
        app.command("focus Korean", key="tab")
        report = app.command("select Korean", key="enter")
        assert report["language"] == "ko"
        assert [group["title"] for group in report["system_menu"]][1:] == ["파일", "편집", "보기", "윈도우", "도움말"]
        app_menu = report["system_menu"][0]["title"]
        choose(app_menu, "설정…")
        for _ in range(5):
            app.command("focus Markdown", key="tab")
        report = app.command("turn Markdown off", key="space")
        assert not report["markdown_enabled"] and not item(report, "보기", "미리 보기")["enabled"]
        choose(app_menu, "설정…")
        report = app.command("restore English", key="enter")
        assert report["language"] == "en"
        app.command("return to writing", key="escape")
        checks.append("Language changes update AppKit titles immediately; Markdown OFF disables its menu action")

        path = data / "shortcut/bindings.json"
        path.write_text(json.dumps({"schemaVersion": 1, "bindings": {
            "new_note": ["Primary+Shift+N"], "reload_shortcuts": ["Primary+Shift+S"]}}))
        report = choose("Help", text["reload_shortcuts"])
        native_new = item(report, "File", "New note")
        assert native_new["key"].lower() == "n" and native_new["modifiers"] & (1 << 17)
        count = len(report["tabs"])
        report = app.command("old native key is removed", "menu_key", key="n")
        assert not report["menu_key_handled"] and len(report["tabs"]) == count
        report = app.command("new native key creates once", "menu_key", key="n", shift=True)
        assert report["menu_key_handled"] and len(report["tabs"]) == count + 1
        empty = report["note_id"]
        report = choose("File", text["shortcut_delete"])
        assert report["pending_delete"] is None and db_note(data, empty) is None
        assert report["body"] == body
        checks.append("Reload updates native key equivalents; old keys stop matching and blank deletion skips confirmation")

        configured = json.loads(path.read_text())
        configured["bindings"].update({"new_note": ["Primary+H"], "delete_note": ["Primary+BrowserBack"]})
        path.write_text(json.dumps(configured))
        report = choose("Help", text["reload_shortcuts"])
        assert item(report, app_menu, "Hide Quick Note")["key"] == ""
        delete = next(row for row in next(group for group in report["system_menu"] if group["title"] == "File")["items"] if row["title"].startswith(text["shortcut_delete"]))
        if not delete["key"]:
            assert "BrowserBack" in delete["title"]
        choose(app_menu, "Settings…")
        report = app.command("settings update native labels", "report")
        back = next(row for row in next(group for group in report["system_menu"] if group["title"] == "File")["items"] if row["title"].startswith(text["back_to_note"]))
        choose("File", back["title"])
        report = app.command("custom key survives label changes", "report")
        delete = next(row for row in next(group for group in report["system_menu"] if group["title"] == "File")["items"] if row["title"].startswith(text["shortcut_delete"]))
        assert delete["key"] or "BrowserBack" in delete["title"]
        configured["bindings"]["delete_note"] = ["Primary+W"]
        path.write_text(json.dumps(configured))
        choose("Help", text["reload_shortcuts"])
        report = app.command("configured Command H overrides Hide", "menu_key", key="h")
        assert report["menu_key_handled"] and report["note_id"] != note_id
        choose("File", text["shortcut_delete"])
        checks.append("App bindings take precedence over Hide; unsupported native equivalents keep visible hints across label changes")

        with sqlite3.connect(data / "notes.sqlite3") as database:
            database.executescript("CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
        report = app.command("failed save keeps a recovery draft", "text", text=" unsaved")
        assert report["dirty"]
        unsaved = report["body"]
        app_menu = report["system_menu"][0]["title"]
        choose(app_menu, "Quit Quick Note")
        report = app.command("quit is blocked on save failure", "report")
        assert app.process.poll() is None and report["dirty"] and report["body"] == unsaved
        assert db_note(data, note_id)[0] == body
        app.command("keep editing", key="enter")
        with sqlite3.connect(data / "notes.sqlite3") as database:
            database.execute("DROP TRIGGER fail_save")
        app.sequence += 1
        (app.folder / "command.json").write_text(json.dumps({"sequence": app.sequence,
            "type": "menu", "menu": app_menu, "item": "Quit Quick Note"}))
        app.process.wait(timeout=10)
        assert app.process.returncode == 0 and db_note(data, note_id)[0] == unsaved
        app.log.close()
        app = None
        checks.append("Quit uses the save-failure guard, then saves and exits after the database recovers")
except Exception:
    (OUTPUT / "result.json").write_text(json.dumps({"passed": False, "checks": checks,
        "actions": actions, "startups": startups}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    if app is not None:
        try:
            app.command("restore clipboard after failed check", "restore_clipboard")
        finally:
            app.cleanup()
    raise

(OUTPUT / "result.json").write_text(json.dumps({"passed": True, "checks": checks,
    "actions": actions, "startups": startups,
    "input": "Actual NSMenu actions and AppKit key-equivalent dispatch in an isolated native Mac process",
    "caveats": ["Cross-process Accessibility clicking, physical IME and Windows runtime are not exercised."]},
    ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"passed": True, "checks": len(checks), "output": str(OUTPUT)}, indent=2))
