"""Build the Rust/WASM application into dist/ using Python's standard library.

Run from any directory with Python 3.11 or newer. Install the Rust WASM target
and the exact wasm-bindgen CLI version recorded in Cargo.lock before building.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from rust_command import configure_windows


ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "dist"
TARGET = "wasm32-unknown-unknown"


def run(*command: str) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=ROOT, check=True)


def check_tools() -> None:
    if shutil.which("cargo") is None:
        raise RuntimeError("cargo is missing. Install Rust from https://rustup.rs/.")

    lockfile = ROOT / "Cargo.lock"
    if not lockfile.is_file():
        raise RuntimeError("Cargo.lock is missing. Run cargo generate-lockfile first.")
    with lockfile.open("rb") as stream:
        lock = tomllib.load(stream)
    versions = {
        package["version"]
        for package in lock.get("package", [])
        if package["name"] == "wasm-bindgen"
    }
    if len(versions) != 1:
        raise RuntimeError("Cargo.lock must contain exactly one wasm-bindgen version.")
    expected = versions.pop()
    install = f"cargo install wasm-bindgen-cli --version {expected} --locked"
    if shutil.which("wasm-bindgen") is None:
        raise RuntimeError(f"wasm-bindgen is missing. Run: {install}")
    result = subprocess.run(
        ["wasm-bindgen", "--version"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    actual = result.stdout.strip().split()
    if actual != ["wasm-bindgen", expected]:
        raise RuntimeError(
            f"wasm-bindgen CLI must match Cargo.lock ({expected}); "
            f"found {result.stdout.strip()!r}. Run: {install} --force"
        )


def check_output(site: Path) -> None:
    forbidden = {".mp4", ".mp3", ".wav", ".webm", ".ogg", ".m4a", ".flac", ".avi", ".mov", ".aac"}
    for asset in site.rglob("*"):
        if asset.is_file() and asset.suffix.lower() in forbidden:
            raise RuntimeError(f"Encoded audio/video is forbidden: {asset.relative_to(site)}")
    required = ("index.html", "style.css", "pkg/aihistory.js", "pkg/aihistory_bg.wasm")
    for relative in required:
        path = site / relative
        if not path.is_file() or not path.stat().st_size:
            raise RuntimeError(f"Build output is missing or empty: {relative}")
    with (site / "pkg/aihistory_bg.wasm").open("rb") as stream:
        if stream.read(8) != b"\x00asm\x01\x00\x00\x00":
            raise RuntimeError("The generated WASM file has an invalid module header.")


def replace_output(site: Path) -> None:
    # Resolve immediately before removal. A symlink or junction must never let
    # generated-output cleanup touch a directory outside this checkout.
    if DIST.is_symlink() or DIST.resolve() != ROOT / "dist":
        raise RuntimeError("Refusing to replace dist/: it points outside the output directory.")
    if DIST.exists():
        if not DIST.is_dir():
            raise RuntimeError("Refusing to replace dist/: the path is not a directory.")
        shutil.rmtree(DIST)
    # Create the public output beneath the checkout so it inherits its ACL.
    # Moving a Windows TemporaryDirectory here would retain its private ACL.
    shutil.copytree(site, DIST)


def main() -> int:
    try:
        configure_windows()
        check_tools()
        if not (ROOT / "web/index.html").is_file():
            raise RuntimeError("The web/index.html source file is missing.")
        run("cargo", "build", "--release", "--locked", "--target", TARGET)
        module = ROOT / "target" / TARGET / "release/aihistory.wasm"
        if not module.is_file():
            raise RuntimeError(f"Cargo did not produce the expected module: {module}")

        # Keep the previous working dist/ intact until the new build is complete.
        with tempfile.TemporaryDirectory(prefix=".aihistory-build-", dir=ROOT) as temporary:
            site = Path(temporary) / "site"
            shutil.copytree(ROOT / "web", site)
            run(
                "wasm-bindgen",
                str(module),
                "--target",
                "web",
                "--no-typescript",
                "--out-dir",
                str(site / "pkg"),
                "--out-name",
                "aihistory",
            )
            (site / ".nojekyll").touch()
            check_output(site)
            replace_output(site)
        size = (DIST / "pkg/aihistory_bg.wasm").stat().st_size
        print(f"Built {DIST} (WASM: {size:,} bytes)")
        print("Preview: python -m http.server 8000 --directory dist --bind 127.0.0.1")
        return 0
    except (OSError, RuntimeError, subprocess.CalledProcessError, tomllib.TOMLDecodeError) as error:
        print(f"Build failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
