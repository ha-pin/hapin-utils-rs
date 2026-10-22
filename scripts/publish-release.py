"""Create/recover a draft release, verify its tag and assets, then publish once."""
import hashlib
import json
import os
import pathlib
import subprocess
import tomllib
import urllib.error
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent

def run(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()

def api(endpoint):
    request = urllib.request.Request("https://api.github.com/" + endpoint,
        headers={"Authorization": "Bearer " + os.environ["GH_TOKEN"], "User-Agent": "hapin-utils-rs-release"})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise RuntimeError(f"GitHub lookup failed with HTTP {error.code}") from None

def main():
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    tag = "v" + version
    repository = os.environ["GITHUB_REPOSITORY"]
    sha = os.environ["GITHUB_SHA"]
    # Validate all expected files before creating a public tag or release.
    assets = [ROOT / "release-dist" / f"hapin-utils-rs-{version}-wasm-{target}.tar.gz" for target in ("web", "nodejs", "bundler")]
    sums = ROOT / "release-dist/SHA256SUMS"
    expected_sums = {name: digest for digest, name in (line.split() for line in sums.read_text().splitlines())}
    for asset in assets:
        assert hashlib.sha256(asset.read_bytes()).hexdigest() == expected_sums[asset.name], f"Bad checksum: {asset.name}"
    assets.append(sums)
    release = api(f"repos/{repository}/releases/tags/{tag}")
    if release and not release["draft"]:
        print(f"Published {tag} already exists; leaving immutable release assets unchanged")
        return
    remote_tag = run("git", "ls-remote", "--tags", "origin", "refs/tags/" + tag)
    if remote_tag:
        run("git", "fetch", "origin", "tag", tag)
        if run("git", "rev-parse", tag + "^{commit}") != sha:
            raise RuntimeError(f"{tag} belongs to another commit; rerun the original commit or bump Cargo.toml version")
    else:
        run("git", "-c", "user.name=Herbert He", "-c", "user.email=Herbert.He0229@gmail.com", "tag", "-a", tag, sha, "-m", "Release " + tag)
        run("git", "push", "origin", "refs/tags/" + tag)
    if not release:
        run("gh", "release", "create", tag, "--repo", repository, "--verify-tag", "--draft", "--title", "hapin-utils-rs " + tag, "--notes", "Native Rust crate and web, Node.js, bundler WASM packages. See README for crates.io publishing status and usage. Verify assets with SHA256SUMS.")
    run("gh", "release", "upload", tag, *map(str, assets), "--repo", repository, "--clobber")
    # Inspect uploaded asset names/sizes before making a recovered draft public.
    uploaded = api(f"repos/{repository}/releases/tags/{tag}")
    actual = {a["name"]: a["size"] for a in uploaded["assets"]}
    for asset in assets:
        assert actual.get(asset.name) == asset.stat().st_size
    args = ["gh", "release", "edit", tag, "--repo", repository, "--draft=false"]
    if '-' in version.split('+')[0]:
        args.append("--prerelease")
    run(*args)
    print(f"Published GitHub Release {tag}")

if __name__ == "__main__":
    main()
