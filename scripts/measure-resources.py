#!/usr/bin/env python3
"""Measure release CPU and memory with isolated notes; never open the personal store."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from resource_usage import measure
from resource_fixture import seed


ROOT = Path(__file__).resolve().parent.parent


def run_case(output, scenario, count, seconds):
    with tempfile.TemporaryDirectory(prefix="quick-note-resources-") as temporary:
        directory = Path(temporary)
        text_bytes = seed(directory, count)
        binary = ROOT / "target/release" / ("quick-note" if scenario == "idle" else "resource-workload")
        report = output / f"{scenario}-{count}-workload.json"
        arguments = ([str(binary), "--data-directory", str(directory)] if scenario == "idle"
                     else [str(binary), str(directory), str(report), scenario])
        with (output / f"{scenario}-{count}.log").open("w", encoding="utf-8") as log:
            process = subprocess.Popen(arguments, stdout=log, stderr=log)
            try:
                time.sleep(3)
                before = json.loads(report.read_text()) if report.exists() else None
                result = measure(process, seconds)
                # Region attribution is diagnostic only; it is outside the CPU sample interval.
                try:
                    attribution = subprocess.run(
                        ["/usr/bin/footprint", "-p", str(process.pid), "-j",
                         str(output / f"{scenario}-{count}-footprint.json")],
                        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=20,
                    )
                except subprocess.TimeoutExpired:
                    attribution = subprocess.CompletedProcess([], 1, "Region attribution timed out.")
                (output / f"{scenario}-{count}-footprint.txt").write_text(attribution.stdout)
                result.update({"scenario": scenario, "notes": count, "note_text_bytes": text_bytes,
                               "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                               "region_attribution_available": attribution.returncode == 0})
                if report.exists():
                    workload = json.loads(report.read_text())
                    assert not workload["dirty"] and workload["error"] is None, workload
                    if before is not None:
                        assert workload["inputs"] - before["inputs"] >= seconds * 4, "The workload window did not stay active enough to measure"
                    result["workload"] = workload
                if scenario != "idle" and not report.exists():
                    raise RuntimeError("The workload window did not render")
                (output / f"{scenario}-{count}.json").write_text(json.dumps(result, indent=2))
                return result
            finally:
                # Only the exact child created for this isolated fixture is stopped.
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seconds", type=int, default=20)
    parser.add_argument("--scenario", choices=["all", "idle", "typing", "search"], default="all")
    arguments = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("This sampler uses macOS process counters; Windows needs a native sampler.")
    if not 5 <= arguments.seconds <= 60:
        parser.error("Choose a sample period between 5 and 60 seconds.")
    output = ROOT / ".artifacts" / ("resources-" + time.strftime("%Y%m%d-%H%M%S"))
    output.mkdir(parents=True)
    cases = [("idle", 1), ("idle", 52), ("typing", 52), ("search", 52)]
    if arguments.scenario != "all":
        cases = [case for case in cases if case[0] == arguments.scenario]
    results = []
    for scenario, count in cases:
        result = run_case(output, scenario, count, arguments.seconds)
        results.append(result)
        print(json.dumps({key: value for key, value in result.items() if key != "samples"}), flush=True)
    report = {
        "platform": subprocess.check_output(["sw_vers", "-productVersion"], text=True).strip(),
        "hardware": subprocess.check_output(["sysctl", "-n", "hw.model"], text=True).strip(),
        "ram_bytes": int(subprocess.check_output(["sysctl", "-n", "hw.memsize"], text=True)),
        "results": results,
        "measurement": "CPU seconds / wall seconds; one full core is 100%. MiB = 1024^2 bytes.",
        "caveats": ["Idle uses the production release binary; active load uses the same production UI with scripted 8 Hz input.",
                    "Physical footprint and RSS are different OS counters; do not add them together.",
                    "GPU/WindowServer utilization, physical IME, Windows and long-term leak behavior are not measured."],
    }
    (output / "result.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"Evidence: {output}")


if __name__ == "__main__":
    main()
