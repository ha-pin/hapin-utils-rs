"""Archive the complete WASM package for each target and write SHA-256 checksums."""
import hashlib
import pathlib
import tarfile
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
out = ROOT / "release-dist"
out.mkdir(exist_ok=True)
checksums = []
for target in ("web", "nodejs", "bundler"):
    source = ROOT / "pkg" / target
    for file in ("hapin_utils_rs.js", "hapin_utils_rs_bg.wasm", "hapin_utils_rs.d.ts", "package.json"):
        if not (source / file).is_file():
            raise RuntimeError(f"Missing generated file: {target}/{file}")
    asset = out / f"hapin-utils-rs-{version}-wasm-{target}.tar.gz"
    with tarfile.open(asset, "w:gz") as archive:
        for file in sorted(source.iterdir()):
            if file.is_file() and not file.name.startswith('.'):
                archive.add(file, arcname=file.name)
    checksums.append(f"{hashlib.sha256(asset.read_bytes()).hexdigest()}  {asset.name}")
(out / "SHA256SUMS").write_text("\n".join(checksums) + "\n")
print("\n".join(checksums))
