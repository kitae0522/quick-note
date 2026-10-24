#!/usr/bin/env python3
"""Enforce the 250-line limit on application, test and tool sources."""
from pathlib import Path
import sys


def main():
    root = Path(__file__).resolve().parent.parent
    extensions = {".rs", ".py", ".sh", ".ps1", ".html", ".css", ".js", ".sql", ".json"}
    sources = []
    for directory in ["src", "tests", "tools", "examples", "scripts", "i18n", "shortcut"]:
        sources.extend(
            path
            for path in (root / directory).rglob("*")
            if path.is_file() and path.suffix in extensions
        )
    failures = []
    for path in sorted(sources):
        count = len(path.read_text(encoding="utf-8").splitlines())
        if count > 250:
            failures.append(f"{path.relative_to(root)}: {count} lines (limit: 250)")
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"Source size: {len(sources)} files are within 250 lines; vendor is excluded.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
