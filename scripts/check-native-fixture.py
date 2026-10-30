#!/usr/bin/env python3
"""Check diagnostic cleanup ordering without reading or writing the system clipboard."""
import io
import json
from pathlib import Path
import tempfile
import unittest

from native_fixture import Native


class Process:
    def __init__(self):
        self.killed = False

    def poll(self):
        return None

    def kill(self):
        self.killed = True

    def wait(self, timeout):
        return 0


class ClipboardCleanup(unittest.TestCase):
    def fixture(self):
        app = object.__new__(Native)
        app.process = Process()
        app.clipboard_used = True
        app.log = io.StringIO()
        return app

    def test_cleanup_waits_for_restore_before_stopping_the_process(self):
        app = self.fixture()
        restored = []

        def command(label, kind):
            self.assertFalse(app.process.killed)
            restored.append(kind)

        app.command = command
        app.cleanup()
        self.assertEqual(restored, ["clipboard_restore"])
        self.assertTrue(app.process.killed)

    def test_failed_restore_keeps_the_in_memory_backup_process_alive(self):
        app = self.fixture()

        def command(label, kind):
            raise RuntimeError("fixture restore failure")

        app.command = command
        with self.assertRaises(RuntimeError):
            app.cleanup()
        self.assertFalse(app.process.killed)
        self.assertFalse(app.log.closed)

    def test_a_prior_failure_does_not_prevent_receiving_the_restore_acknowledgment(self):
        app = self.fixture()
        with tempfile.TemporaryDirectory() as temporary:
            app.folder = Path(temporary)
            failure = {"error": "earlier command failure"}
            (app.folder / "failure.json").write_text(json.dumps(failure))
            (app.folder / "reply.json").write_text('{"sequence":3}')
            self.assertEqual(app.wait("reply.json", 3, prior_failure=failure), {"sequence": 3})
            with self.assertRaises(RuntimeError):
                app.wait("reply.json", 3)


if __name__ == "__main__":
    unittest.main()
