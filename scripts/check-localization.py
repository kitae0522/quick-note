#!/usr/bin/env python3
"""Verify both languages, persisted choices and editable UTF-8 catalogs in a native window."""
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("localization-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
app = None
try:
    with tempfile.TemporaryDirectory(prefix="quick-note-i18n-") as temporary:
        data = Path(temporary)
        app = Native(data, "language", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        body = '# Saved\n\n영어·한국어 💡\n\n```rust\nlet idea = "keep it simple";\n```'
        before = app.command("save fixture", "text", text=body)
        note_id = before["note_id"]
        app.command("open options", key="comma", command=True)
        app.command("focus Korean", key="tab")
        report = app.command("select Korean", key="enter")
        assert report["language"] == "ko"
        app.command("Korean options", "screenshot", name="korean-options")
        report = app.command("return to writing", key="escape")
        assert report["editor_focused"] and report["body"] == body
        assert db_note(data, note_id)[0] == body
        assert report["selection"] == before["selection"]
        checks.append("Keyboard language selection updates Korean UI without changing text or cursor")

        report = app.command("new Korean draft", key="n", command=True)
        blank = report["note_id"]
        assert next(tab["title"] for tab in report["tabs"] if tab["id"] == blank) == "제목 없음"
        report = app.command("delete empty Korean draft", key="w", command=True)
        assert report["pending_delete"] is None and db_note(data, blank) is None
        app.command("Markdown preview", key="m", command=True, shift=True)
        app.command("Korean preview", "screenshot", name="korean-preview")
        app.command("edit again", key="m", command=True, shift=True)
        app.command("search", key="p", command=True)
        app.command("search user text", "text", text="한국어")
        report = app.command("open result", key="enter")
        assert report["note_id"] == note_id
        app.command("delete confirmation", key="w", command=True)
        app.command("Korean confirmation", "screenshot", name="korean-confirmation")
        report = app.command("default Cancel", key="enter")
        assert report["pending_delete"] is None and db_note(data, note_id)[0] == body
        checks.append("Korean titles, preview, search and confirmation retain existing note behavior")

        with sqlite3.connect(data / "notes.sqlite3") as connection:
            connection.executescript("CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
        report = app.command("failed save", "text", text=" unsaved")
        assert report["dirty"] and "노트를 저장하지 못했지만" in report["error"]
        app.command("Korean recovery notice", "screenshot", name="korean-save-error")
        with sqlite3.connect(data / "notes.sqlite3") as connection:
            connection.execute("DROP TRIGGER fail_save")
        report = app.command("retry", key="s", command=True)
        assert not report["dirty"] and report["error"] is None
        body = report["body"]
        checks.append("Failed saves explain recovery in Korean and retry preserves UTF-8 text")
        app.close()
        app = None

        (data / "i18n").mkdir()
        catalog = json.loads((ROOT / "i18n/ko.json").read_text(encoding="utf-8"))
        catalog["saved"] = "보관됨 · 사용자 문구"
        (data / "i18n/ko.json").write_text(json.dumps(catalog, ensure_ascii=False), encoding="utf-8")
        app = Native(data, "restart", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        report = app.command("restored language", "report")
        assert report["language"] == "ko" and report["body"] == body
        assert report["saved_label"] == "보관됨 · 사용자 문구"
        app.command("edited catalog", "screenshot", name="custom-json")
        app.command("open options", key="comma", command=True)
        report = app.command("select English", key="enter")
        assert report["language"] == "en" and report["saved_label"] == "Saved"
        app.command("close options", key="escape")
        assert db_note(data, note_id)[0] == body
        checks.append("Language survives restart, edited JSON loads without rebuilding, and English can be restored")
        app.close()
        app = None
except Exception:
    if app is not None:
        app.cleanup()
    raise

(OUTPUT / "result.json").write_text(json.dumps({"passed": True, "checks": checks,
    "actions": actions, "startups": startups,
    "input": "Scripted keyboard/IME events in an isolated native GPU window",
    "caveats": ["Windows runtime, physical IME and VoiceOver are not exercised."]},
    ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"passed": True, "checks": checks, "evidence": str(OUTPUT)}, indent=2))
