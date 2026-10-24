"""macOS process counters from the public RUSAGE_INFO_V0 layout in sys/resource.h."""
import ctypes
import statistics
import re
import subprocess
import time


class MachTimebase(ctypes.Structure):
    _fields_ = [("numer", ctypes.c_uint32), ("denom", ctypes.c_uint32)]


TIMEBASE = MachTimebase()
if ctypes.CDLL("/usr/lib/libSystem.B.dylib").mach_timebase_info(ctypes.byref(TIMEBASE)) != 0:
    raise OSError("mach_timebase_info failed")


def cpu_seconds_between(before, after):
    # proc_pid_rusage CPU fields use Mach absolute ticks, not nanoseconds.
    ticks = after.user + after.system - before.user - before.system
    return ticks * TIMEBASE.numer / TIMEBASE.denom / 1e9


def frontmost_pid():
    # LaunchServices is queried afresh: NSWorkspace caches require a running Cocoa loop.
    command = subprocess.check_output(["/usr/bin/lsappinfo", "front"], text=True).strip()
    info = subprocess.check_output(
        ["/usr/bin/lsappinfo", "info", "-only", "pid", "-app", command], text=True
    )
    match = re.search(r"pid\s*=\s*(\d+)", info)
    return int(match.group(1)) if match else None


class ProcessUsage(ctypes.Structure):
    _fields_ = [("uuid", ctypes.c_uint8 * 16)] + [
        (name, ctypes.c_uint64)
        for name in [
            "user", "system", "idle_wakeups", "interrupt_wakeups", "pageins",
            "wired", "resident", "footprint", "started", "exited",
        ]
    ]


def read_usage(process_id):
    library = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
    library.proc_pid_rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]
    library.proc_pid_rusage.restype = ctypes.c_int
    usage = ProcessUsage()
    if library.proc_pid_rusage(process_id, 0, ctypes.byref(usage)) != 0:
        raise OSError(ctypes.get_errno(), "proc_pid_rusage failed")
    return usage


def measure(process, seconds):
    previous = read_usage(process.pid)
    previous_time = time.monotonic()
    first, first_time = previous, previous_time
    samples = []
    for _ in range(seconds):
        time.sleep(1)
        if process.poll() is not None:
            raise RuntimeError("The measured process exited early")
        current_time = time.monotonic()
        current = read_usage(process.pid)
        cpu_seconds = cpu_seconds_between(previous, current)
        samples.append({
            "cpu_one_core_percent": cpu_seconds / (current_time - previous_time) * 100,
            "physical_footprint_mib": current.footprint / 1024**2,
            "resident_mib": current.resident / 1024**2,
            "foreground": frontmost_pid() == process.pid,
        })
        previous, previous_time = current, current_time
    return {
        "seconds": seconds,
        "wall_seconds": current_time - first_time,
        "cpu_seconds": cpu_seconds_between(first, current),
        "cpu_window_percent": cpu_seconds_between(first, current) / (current_time - first_time) * 100,
        "cpu_timebase": {"numer": TIMEBASE.numer, "denom": TIMEBASE.denom},
        "cpu_mean_percent": statistics.mean(row["cpu_one_core_percent"] for row in samples),
        "cpu_peak_percent": max(row["cpu_one_core_percent"] for row in samples),
        "footprint_median_mib": statistics.median(row["physical_footprint_mib"] for row in samples),
        "footprint_peak_mib": max(row["physical_footprint_mib"] for row in samples),
        "resident_median_mib": statistics.median(row["resident_mib"] for row in samples),
        "resident_peak_mib": max(row["resident_mib"] for row in samples),
        "foreground_samples": sum(row["foreground"] for row in samples),
        "samples": samples,
    }
