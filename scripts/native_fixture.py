"""Shared isolated native-window driver; never opens the personal note directory."""
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import time

def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError):
        return None


class Native:
    def __init__(self, data, name, *, output, binary, actions, startups, arguments=()):
        self.actions = actions
        self.folder = output / name
        self.folder.mkdir()
        self.sequence = 0
        self.clipboard_used = False
        self.log = (self.folder / "app.log").open("w", encoding="utf-8")
        self.process = subprocess.Popen([str(binary), "--data-directory", str(data), "--output", str(self.folder), *arguments], stdout=self.log, stderr=self.log)
        self.started = self.wait("startup.json")
        assert self.started["window_focused"] and self.started["editor_focused"]
        startups.append({"scenario": name, **self.started})

    def wait(self, filename, sequence=None, prior_failure=None):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            failure = read_json(self.folder / "failure.json")
            if failure and failure != prior_failure:
                raise RuntimeError(failure)
            report = read_json(self.folder / filename)
            if report and (sequence is None or report.get("sequence") == sequence):
                return report
            if self.process.poll() is not None:
                raise RuntimeError(f"Native window exited before {filename}; see {self.folder}")
            time.sleep(0.005)
        if sys.platform == "darwin" and self.process.poll() is None:
            try:
                subprocess.run(["sample", str(self.process.pid), "1", "1", "-file",
                                str(self.folder / "timeout-stack.txt")],
                               capture_output=True, timeout=5, check=False)
            except subprocess.TimeoutExpired:
                pass
        raise TimeoutError(f"No {filename} from {self.folder}")

    def command(self, label, kind="key", **arguments):
        if kind == "clipboard_fixture" or arguments.get("clipboard"):
            self.clipboard_used = True
        prior_failure = (read_json(self.folder / "failure.json")
                         if kind in ("clipboard_restore", "restore_clipboard") else None)
        self.sequence += 1
        command = {"sequence": self.sequence, "type": kind, **arguments}
        temporary = self.folder / "next.json"
        temporary.write_text(json.dumps(command, ensure_ascii=False), encoding="utf-8")
        temporary.replace(self.folder / "command.json")
        result = self.wait("reply.json", self.sequence, prior_failure=prior_failure)
        if kind in ("clipboard_restore", "restore_clipboard"):
            self.clipboard_used = False
        self.actions.append({"scenario": self.folder.name, "action": label, **result})
        return result

    def close(self):
        if self.clipboard_used:
            self.command("restore clipboard before closing", "clipboard_restore")
        self.sequence += 1
        (self.folder / "command.json").write_text(json.dumps({"sequence": self.sequence, "type": "key", "key": "q", "command": True}), encoding="utf-8")
        self.process.wait(timeout=10)
        assert self.process.returncode == 0
        self.log.close()

    def cleanup(self):
        if self.process.poll() is None:
            if self.clipboard_used:
                # A failed restore leaves the process and its in-memory backup available.
                self.command("restore clipboard before cleanup", "clipboard_restore")
            self.process.kill()
            self.process.wait(timeout=5)
        self.log.close()


def db_note(data, note_id):
    with sqlite3.connect(data / "notes.sqlite3") as database:
        assert database.execute("PRAGMA encoding").fetchone()[0] == "UTF-8"
        assert database.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
        return database.execute("SELECT body, selection_location, selection_length FROM notes WHERE id=? AND deleted_at IS NULL", (note_id,)).fetchone()
