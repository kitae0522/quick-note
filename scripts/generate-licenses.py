#!/usr/bin/env python3
"""Generate UTF-8 dependency notices with consistent line endings for Git and packages."""
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "assets/licenses/Third-party.html"


def main():
    subprocess.run([
        "cargo", "about", "generate", "--locked", "--all-features", "--fail",
        "about.hbs", "-o", str(OUTPUT),
    ], cwd=ROOT, check=True)
    # Upstream license texts can contain CRLF and trailing spaces; retain their words.
    lines = OUTPUT.read_text(encoding="utf-8").splitlines()
    OUTPUT.write_text("\n".join(line.rstrip() for line in lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
