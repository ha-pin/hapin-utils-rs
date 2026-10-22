"""Build all JS-compatible WASM targets; no publishing or credentials involved."""
import pathlib
import subprocess
import shutil

ROOT = pathlib.Path(__file__).resolve().parent.parent
for target in ("nodejs", "web", "bundler"):
    subprocess.run([
        "wasm-pack", "build", "--target", target, "--release",
        "--out-dir", str(ROOT / "pkg" / target), "--out-name", "hapin_utils_rs",
        "--", "--features", "wasm", "--locked",
    ], cwd=ROOT, check=True)
    for name in ("LICENSE", "README.zh-CN.md"):
        shutil.copy2(ROOT / name, ROOT / "pkg" / target / name)
