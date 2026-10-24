#!/usr/bin/env python3
"""Check Mach CPU counters against the independent process CPU clock on this Mac."""
import json
import os
import time

from resource_usage import TIMEBASE
from resource_usage import cpu_seconds_between
from resource_usage import read_usage


before = read_usage(os.getpid())
reference_start = time.process_time()
value = 1
while time.process_time() - reference_start < 0.25:
    value = (value * 17 + 11) % 999983
reference_seconds = time.process_time() - reference_start
after = read_usage(os.getpid())
measured_seconds = cpu_seconds_between(before, after)
assert abs(measured_seconds - reference_seconds) < max(0.015, reference_seconds * 0.08), (
    measured_seconds, reference_seconds
)
print(json.dumps({"measured_cpu_seconds": measured_seconds, "reference_cpu_seconds": reference_seconds,
                  "timebase_numer": TIMEBASE.numer, "timebase_denom": TIMEBASE.denom, "passed": True}))
