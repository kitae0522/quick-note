#!/usr/bin/env python3
"""Compare Markdown ON/OFF with the same release binaries and isolated UTF-8 note."""
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from resource_fixture import seed
from resource_usage import measure
from resource_usage import read_usage
from resource_usage import cpu_seconds_between


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / ".artifacts" / ("markdown-resources-" + time.strftime("%Y%m%d-%H%M%S"))
OUTPUT.mkdir(parents=True)
BODY = "# A place for ideas\n\n" + "\n\n".join(
    f"## Idea {index}\n\nKeep **one useful thought**. 한글 💡\n\n"
    "- Capture it now\n- Make room for the next idea\n\n"
    f"```rust\nlet idea = {index};\n```" for index in range(64)
)
SAMPLE_SECONDS = 5


def sample(phase, enabled, trial):
    label = f"{phase}-{'on' if enabled else 'off'}-{trial}"
    with tempfile.TemporaryDirectory(prefix="quick-note-markdown-cost-") as temporary:
        data = Path(temporary)
        seed(data, 1)
        with sqlite3.connect(data / "notes.sqlite3") as database:
            database.execute("UPDATE notes SET body=?,selection_location=?,title='A place for ideas' WHERE id='0'",
                             (BODY, len(BODY.encode("utf-16-le")) // 2))
            # Warmed cases start ON, visit Preview, then return to Edit with the target setting.
            database.execute("INSERT INTO app_state VALUES('markdownEnabled',?)",
                             ("true" if enabled or phase == "warmed" else "false",))
        binary = ROOT / "target/release/resource-workload"
        workload_report = OUTPUT / (label + "-workload.json")
        mode = f"markdown-{'fresh-' if phase == 'fresh' else ''}{'on' if enabled else 'off'}"
        arguments = [str(binary), str(data), str(workload_report), mode]
        with (OUTPUT / (label + ".log")).open("w", encoding="utf-8") as log:
            process = subprocess.Popen(arguments, stdout=log, stderr=log)
            try:
                time.sleep(3)
                report = json.loads(workload_report.read_text())
                assert report["mode"] == "edit" and report["markdown_enabled"] == enabled, report
                assert report["error"] is None and not report["dirty"], report
                assert report["body_bytes"] == len(BODY.encode("utf-8")) and report["inputs"] == 0, report
                before = read_usage(process.pid)
                started = time.monotonic()
                measured = measure(process, SAMPLE_SECONDS)
                wall_seconds = time.monotonic() - started
                after = read_usage(process.pid)
                cpu_seconds = cpu_seconds_between(before, after)
                return {"phase": phase, "enabled": enabled, "trial": trial, **measured,
                        "wall_seconds": wall_seconds, "cpu_seconds": cpu_seconds,
                        "cpu_window_percent": cpu_seconds / wall_seconds * 100,
                        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
            finally:
                process.terminate()
                process.wait(timeout=5)


results = []
for phase in ["fresh", "warmed"]:
    for trial, order in [(1, [True, False]), (2, [False, True])]:
        for enabled in order:
            result = sample(phase, enabled, trial)
            results.append(result)
            print(json.dumps({key: value for key, value in result.items() if key != "samples"}), flush=True)
report = {
    "results": results,
    "note_utf8_bytes": len(BODY.encode("utf-8")),
    "note_sha256": hashlib.sha256(BODY.encode("utf-8")).hexdigest(),
    "measurement": "5-second OS process samples after 3 seconds warmup. One full core = 100%; MiB = 1024^2 bytes.",
    "caveats": ["Two trials per setting are descriptive; small differences can be run-to-run noise.",
                "Both cases use the production UI in an 8 Hz diagnostic driver that discards physical input; this is not an idle-app CPU benchmark.",
                "Fresh never opens Preview. Warmed opens Preview and then returns to Edit. Neither records GPU readback.",
                "Physical footprint and RSS are separate counters. GPU/WindowServer, startup latency and Windows runtime are not measured.",
                "Dropped Rust caches can remain in the allocator's reserved memory and may not immediately reduce OS footprint."],
}
(OUTPUT / "result.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"Evidence: {OUTPUT}")
