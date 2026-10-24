#!/usr/bin/env python3
"""Comparable release benchmarks using only isolated, reproducible note fixtures."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import time

from resource_fixture import seed
from resource_usage import measure


ROOT = Path(__file__).resolve().parent.parent
LARGE = "# Large capture\n\n" + "\n\n".join(
    f"## Idea {index}\n\n한글 💡 café. Keep one useful thought.\n\n"
    "- Capture it now\n- Make room for the next idea\n\n"
    f"```rust\nlet idea = {index};\n```" for index in range(1800)
)


def fixture(directory, notes, large=False):
    total = seed(directory, notes)
    if large:
        with sqlite3.connect(directory / "notes.sqlite3") as database:
            previous = database.execute("SELECT length(CAST(body AS BLOB)) FROM notes WHERE id='0'").fetchone()[0]
            database.execute("UPDATE notes SET body=?,title='Large capture',selection_location=? WHERE id='0'",
                             (LARGE, len(LARGE.encode("utf-16-le")) // 2))
        total += len(LARGE.encode("utf-8")) - previous
    return total


def resource(binaries, output, label, mode, notes, large, seconds):
    with tempfile.TemporaryDirectory(prefix="quick-note-performance-") as temporary:
        directory = Path(temporary)
        total = fixture(directory, notes, large)
        binary = binaries / ("quick-note" if mode == "idle" else "resource-workload")
        report = output / (label + "-workload.json")
        arguments = ([str(binary), "--data-directory", str(directory)] if mode == "idle"
                     else [str(binary), str(directory), str(report), mode])
        with (output / (label + ".log")).open("w", encoding="utf-8") as log:
            process = subprocess.Popen(arguments, stdout=log, stderr=log)
            try:
                time.sleep(3)
                result = measure(process, seconds)
                result.update({"kind": "resource", "case": label, "scenario": mode, "notes": notes,
                               "fixture_text_bytes": total, "large": large,
                               "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest()})
                if mode != "idle":
                    workload = json.loads(report.read_text())
                    assert not workload["dirty"] and workload["error"] is None, workload
                    result["workload"] = workload
                (output / (label + ".json")).write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
                return result
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binaries", type=Path, default=ROOT / "target/release")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seconds", type=int, default=10)
    parser.add_argument("--trials", type=int, default=2)
    parser.add_argument("--core-only", action="store_true")
    arguments = parser.parse_args()
    assert 5 <= arguments.seconds <= 30 and 1 <= arguments.trials <= 3
    binaries, output = arguments.binaries.resolve(), arguments.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    results = []
    for trial in range(1, arguments.trials + 1):
        cases = [("small", 52, False), ("many", 1000, False), ("large", 52, True)]
        for name, notes, large in cases if trial % 2 else reversed(cases):
            with tempfile.TemporaryDirectory(prefix="quick-note-core-performance-") as temporary:
                fixture(Path(temporary), notes, large)
                report = output / f"core-{name}-{trial}.json"
                subprocess.run([str(binaries / "performance-probe"), temporary, str(report)],
                               check=True, timeout=180, stdout=subprocess.DEVNULL)
                result = json.loads(report.read_text())
                result.update({"kind": "core", "case": name, "trial": trial,
                               "binary_sha256": hashlib.sha256((binaries / "performance-probe").read_bytes()).hexdigest()})
                results.append(result)
                print(json.dumps({"case": name, "trial": trial, "results": result}), flush=True)
        if arguments.core_only:
            continue
        cases = [("idle-one", "idle", 1, False), ("idle-52", "idle", 52, False),
                 ("typing-52", "typing", 52, False), ("search-1000", "search", 1000, False),
                 ("typing-large", "typing", 52, True), ("preview-large", "preview", 52, True)]
        for name, mode, notes, large in cases if trial % 2 else reversed(cases):
            result = resource(binaries, output, f"{name}-{trial}", mode, notes, large, arguments.seconds)
            results.append(result)
            print(json.dumps({"case": name, "trial": trial,
                              **{key: value for key, value in result.items() if key not in ["samples", "workload"]}}), flush=True)
    report = {"results": results, "large_note_bytes": len(LARGE.encode("utf-8")),
              "large_note_sha256": hashlib.sha256(LARGE.encode("utf-8")).hexdigest(),
              "sample_seconds": arguments.seconds, "trials": arguments.trials,
              "hardware": subprocess.check_output(["sysctl", "-n", "hw.model"], text=True).strip(),
              "os": subprocess.check_output(["sw_vers", "-productVersion"], text=True).strip(),
              "caveats": ["Core UI timings exclude GPU presentation; native action checks report that separately.",
                          "Idle uses production; active resource cases drive production UI at up to 8 Hz without GPU readback.",
                          "CPU seconds, elapsed time, physical footprint and RSS are separate measured counters.",
                          "OS caches are not flushed; physical IME, WindowServer/GPU CPU and Windows runtime are not measured."]}
    (output / "result.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Evidence: {output}")


if __name__ == "__main__":
    main()
