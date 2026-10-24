#!/usr/bin/env python3
"""Verify the default-on Markdown switch and persisted plain text in native Mac windows."""
import json
from pathlib import Path
import subprocess
import tempfile
import time

from native_fixture import Native
from native_fixture import db_note


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("markdown-native-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                "--bin", "native-check"], cwd=ROOT, check=True)
actions, startups, checks = [], [], []
app = None


def toggle_markdown(window):
    window.command("open options", key="comma", command=True)
    for _ in range(5):
        window.command("focus Markdown switch", key="tab")
    return window.command("toggle Markdown with Space", key="space")


try:
    with tempfile.TemporaryDirectory(prefix="quick-note-markdown-") as temporary:
        data = Path(temporary)
        app = Native(data, "default", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        body = '# An idea worth keeping\n\n마크다운을 꺼도 그대로 💡\n\n- One small thought\n\n```rust\nlet idea = "keep it";\n```'
        before = app.command("save source", "text", text=body)
        note_id = before["note_id"]
        assert before["markdown_enabled"]
        report = app.command("preview works by default", key="m", command=True, shift=True)
        assert report["mode"] == "preview"
        app.command("rendered Markdown", "screenshot", name="default-preview")
        report = toggle_markdown(app)
        assert not report["markdown_enabled"] and report["mode"] == "edit"
        app.command("switch off", "screenshot", name="markdown-off")
        report = app.command("return to plain text", key="escape")
        assert report["editor_focused"] and report["body"] == body
        assert report["selection"] == before["selection"]
        report = app.command("preview shortcut stays off", key="m", command=True, shift=True)
        assert report["mode"] == "edit" and not report["markdown_enabled"]
        assert db_note(data, note_id)[0] == body
        checks.append("Default Markdown preview can be disabled with keyboard without changing text or cursor")
        app.close()
        app = None

        app = Native(data, "restart", output=OUTPUT, binary=ROOT / "target/release/native-check",
                     actions=actions, startups=startups)
        report = app.command("verify saved OFF choice", "report")
        assert not report["markdown_enabled"] and report["body"] == body
        report = app.command("OFF still blocks preview", key="m", command=True, shift=True)
        assert report["mode"] == "edit"
        report = toggle_markdown(app)
        assert report["markdown_enabled"] and report["mode"] == "edit"
        app.command("switch on", "screenshot", name="markdown-on")
        app.command("close options", key="escape")
        report = app.command("render after re-enabling", key="m", command=True, shift=True)
        assert report["mode"] == "preview" and report["body"] == body
        app.command("restored preview", "screenshot", name="restored-preview")
        assert db_note(data, note_id)[0] == body
        checks.append("OFF survives restart and re-enabling restores Markdown from the same saved source")
        app.close()
        app = None
    (OUTPUT / "result.json").write_text(json.dumps({"checks": checks, "startups": startups,
        "actions": actions, "input": "Scripted egui events in native macOS windows; not physical keyboard"},
        ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(OUTPUT), "checks_passed": len(checks)}, ensure_ascii=False))
finally:
    if app is not None:
        app.cleanup()
