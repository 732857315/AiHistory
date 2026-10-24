"""Run Cargo, discovering installed MSVC tools on Windows when needed."""
from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import sys


def configure_windows() -> None:
    if os.name != "nt" or shutil.which("link.exe"):
        return
    program_files = Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"))
    candidates = sorted((program_files / "Microsoft Visual Studio").glob("*/BuildTools/VC/Tools/MSVC/*/bin/Hostx64/x64/link.exe"))
    if not candidates:
        return
    binary = candidates[-1].parent
    msvc = binary.parents[2]
    sdk_candidates = sorted((program_files / "Windows Kits/10/Lib").glob("*/um/x64/kernel32.lib"))
    if not sdk_candidates:
        return
    sdk_lib = sdk_candidates[-1].parents[2]
    libraries = [msvc / "lib/x64", sdk_lib / "um/x64", sdk_lib / "ucrt/x64"]
    os.environ["PATH"] = str(binary) + os.pathsep + os.environ.get("PATH", "")
    os.environ["LIB"] = os.pathsep.join(map(str, libraries)) + os.pathsep + os.environ.get("LIB", "")


if __name__ == "__main__":
    configure_windows()
    raise SystemExit(subprocess.call(["cargo", *sys.argv[1:]], cwd=Path(__file__).resolve().parents[1]))
