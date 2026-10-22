"""Read-only release planning. A 404 is absent; other registry/API failures fail closed."""
import json
import os
import pathlib
import re
import tomllib
import urllib.error
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent

def fetch(url, token=None):
    headers = {"User-Agent": "hapin-utils-rs-release"}
    if token:
        headers["Authorization"] = "Bearer " + token
    try:
        with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise RuntimeError(f"Release lookup failed with HTTP {error.code}") from None

def plan(manifest, repository, token=None):
    package = manifest["package"]
    name, version = package["name"], package["version"]
    if not re.fullmatch(r"[A-Za-z0-9_-]+", name) or not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", version):
        raise ValueError("Unexpected package name or version")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Invalid repository")
    crate = fetch(f"https://crates.io/api/v1/crates/{name}/{version}")
    release = fetch(f"https://api.github.com/repos/{repository}/releases/tags/v{version}", token)
    return {"version": version, "crate_exists": str(crate is not None).lower(),
            "release_exists": str(release is not None and not release.get("draft", False)).lower(),
            "prerelease": str('-' in version.split('+')[0]).lower()}

if __name__ == "__main__":
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    result = plan(manifest, os.environ.get("GITHUB_REPOSITORY", "ha-pin/hapin-utils-rs"), os.environ.get("GH_TOKEN"))
    print(json.dumps(result))
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a") as output:
            for key, value in result.items():
                output.write(f"{key}={value}\n")
