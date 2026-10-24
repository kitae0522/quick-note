#!/usr/bin/env python3
"""Measure fresh processes with isolated notes; preserve both input and focused-frame timings."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import platform
import resource
import shutil
import sqlite3
import statistics
import subprocess
import sys
import tempfile
import time

if sys.platform != "darwin":
    raise SystemExit("This startup benchmark currently measures macOS only.")

from native_fixture import read_json
from resource_usage import frontmost_pid
from resource_usage import read_usage


ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--binary", type=Path)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--trials", type=int, default=10)
arguments = parser.parse_args()
if arguments.trials < 2:
    parser.error("at least two fresh-process trials are required")
foreground_pid = frontmost_pid()
if foreground_pid is None:
    raise SystemExit("An active macOS GUI session is required to measure window/editor focus.")
foreground = subprocess.check_output(
    ["ps", "-p", str(foreground_pid), "-o", "comm="], text=True).strip()
if foreground.endswith("/loginwindow"):
    raise SystemExit("Unlock the Mac before measuring real window/editor focus; no timings were collected.")
if arguments.binary is None:
    subprocess.run(["cargo", "build", "--locked", "--release", "--features", "diagnostics",
                    "--bin", "startup-probe"], cwd=ROOT, check=True)
binary = (arguments.binary or ROOT / "target/release/startup-probe").resolve()
output = arguments.output.resolve()
output.mkdir(parents=True, exist_ok=False)


def run(data, name, capture=False):
    folder = output / name
    folder.mkdir()
    command = [str(binary), "--data-directory", str(data), "--output", str(folder)]
    if capture:
        command.append("--capture")
    cpu_before = resource.getrusage(resource.RUSAGE_CHILDREN)
    began = time.monotonic()
    with (folder / "app.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=log)
        ready, captured, ready_seen, memory = None, None, None, None
        try:
            deadline = began + 15
            while time.monotonic() < deadline:
                if ready is None:
                    ready = read_json(folder / "input-ready.json")
                    if ready is not None:
                        ready_seen = (time.monotonic() - began) * 1000
                        try:
                            usage = read_usage(process.pid)
                            memory = {"footprint_mib": usage.footprint / 1024**2,
                                      "resident_mib": usage.resident / 1024**2}
                        except OSError:
                            pass  # An already exited child has no live memory snapshot.
                captured = read_json(folder / "captured.json")
                if ready is not None and captured is not None:
                    break
                if process.poll() is not None:
                    raise RuntimeError(f"Startup probe exited early; see {folder}")
                time.sleep(0.001)
            else:
                raise TimeoutError(f"No focused, saved startup frame; see {folder}")
            process.wait(timeout=10)
            assert process.returncode == 0
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)
    cpu_after = resource.getrusage(resource.RUSAGE_CHILDREN)
    assert ready["body"] == captured["body"] == "startup 한💡"
    assert ready["window_focused"] and ready["editor_focused"]
    assert captured["editor_font"] == "platform-monospace"
    with sqlite3.connect(data / "notes.sqlite3") as connection:
        assert connection.execute("PRAGMA encoding").fetchone()[0] == "UTF-8"
        assert connection.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
    return {"scenario": name, "process_to_input_report_ms": ready_seen,
            "process_cpu_seconds": cpu_after.ru_utime + cpu_after.ru_stime
                - cpu_before.ru_utime - cpu_before.ru_stime,
            "input_ready": ready, "captured": captured, "memory_at_input_ready": memory,
            "capture_png": capture}


def reset_draft(data):
    connection = sqlite3.connect(data / "notes.sqlite3")
    with connection:
        connection.execute("UPDATE notes SET body='',title='Untitled',selection_location=0,selection_length=0")
    connection.close()


def distribution(values):
    ordered = sorted(values)
    return {"minimum": min(values), "median": statistics.median(values),
            "p95_nearest_rank": ordered[math.ceil(len(ordered) * 0.95) - 1],
            "maximum": max(values), "under_200_ms": sum(value < 200 for value in values)}


rows, warmup = [], None
with tempfile.TemporaryDirectory(prefix="quick-note-startup-") as temporary:
    temporary = Path(temporary)
    template = temporary / "template"
    template.mkdir()
    warmup = run(template, "warmup-excluded")
    reset_draft(template)
    for index in range(arguments.trials):
        for case in ["fresh-data", "restored-data"]:
            data = temporary / f"{case}-{index:02}"
            if case == "restored-data":
                shutil.copytree(template, data)
            else:
                data.mkdir()
            row = run(data, f"{case}-{index:02}", capture=index == 0)
            row["case"] = case
            rows.append(row)
            print(json.dumps({"case": case, "trial": index,
                              "input_ms": round(row["input_ready"]["phases"]["input_saved_ms"], 2),
                              "frame_ms": round(row["captured"]["focused_frame_captured_ms"], 2)}), flush=True)

summary = {}
for case in ["fresh-data", "restored-data"]:
    selected = [row for row in rows if row["case"] == case]
    summary[case] = {
        "main_to_input_saved_ms": distribution([row["input_ready"]["phases"]["input_saved_ms"] for row in selected]),
        "main_to_focused_native_font_frame_ms": distribution([row["captured"]["focused_frame_captured_ms"] for row in selected]),
        "process_to_input_report_ms": distribution([row["process_to_input_report_ms"] for row in selected]),
        "median_process_cpu_seconds": statistics.median(row["process_cpu_seconds"] for row in selected),
        "input_font_counts": {font: sum(row["input_ready"]["editor_font"] == font for row in selected)
                              for font in {row["input_ready"]["editor_font"] for row in selected}},
    }
result = {"passed": True, "binary": str(binary), "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
          "platform": platform.platform(), "trials_per_case": arguments.trials, "summary": summary,
          "rows": rows, "warmup_excluded": warmup,
          "measurement": {"input": "First scripted text accepted and saved, with actual window/editor focus",
                          "frame": "GPU readback of a focused frame using the final native default font",
                          "process": "Popen to observed input-ready file; includes loader and reporting/polling overhead",
                          "cpu": "Child process user+system CPU through exit, including probe/readback; not idle CPU",
                          "memory": "One live input-ready snapshot; physical footprint and RSS are separate; not peak memory"},
          "caveats": ["OS and driver caches are not flushed; these are fresh processes, not cold disk-cache trials.",
                      "Programmatic executable launch is not a physical Dock click or first Gatekeeper verification.",
                      "No helper forces visibility/focus; native options and app logic match production.",
                      "The warmup is excluded; scripts operate only on temporary fixture notes."]}
(output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"passed": True, "summary": summary, "output": str(output)}, indent=2))
