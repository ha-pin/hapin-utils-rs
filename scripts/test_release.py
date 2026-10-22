import importlib.util
import hashlib
import os
import pathlib
import tempfile
import unittest
import urllib.error
from unittest.mock import patch

ROOT = pathlib.Path(__file__).parent

def load(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

plan = load("release_plan", "release-plan.py")
publish = load("publish_release", "publish-release.py")

class ReleaseTests(unittest.TestCase):
    def test_lookup_only_treats_404_as_absent(self):
        for code in (401,403,429,500,503):
            with patch.object(plan.urllib.request, "urlopen", side_effect=urllib.error.HTTPError("https://example.com", code, "failed", None, None)):
                with self.assertRaises(RuntimeError):
                    plan.fetch("https://example.com")
        with patch.object(plan.urllib.request, "urlopen", side_effect=urllib.error.HTTPError("https://example.com",404,"missing",None,None)):
            self.assertIsNone(plan.fetch("https://example.com"))

    def test_existing_versions_and_drafts(self):
        manifest={"package":{"name":"hapin-utils-rs","version":"5.0.0"}}
        with patch.object(plan,"fetch",side_effect=[None,None]):
            result=plan.plan(manifest,"ha-pin/hapin-utils-rs")
            self.assertEqual((result["crate_exists"],result["release_exists"]),("false","false"))
        with patch.object(plan,"fetch",side_effect=[{"version":{}},{"draft":False}]):
            result=plan.plan(manifest,"ha-pin/hapin-utils-rs")
            self.assertEqual((result["crate_exists"],result["release_exists"]),("true","true"))
        manifest["package"]["version"]="5.1.0-rc.1"
        with patch.object(plan,"fetch",side_effect=[None,{"draft":True}]):
            result=plan.plan(manifest,"ha-pin/hapin-utils-rs")
            self.assertEqual(result["release_exists"],"false")
            self.assertEqual(result["prerelease"],"true")

    def assets(self, root):
        (root/"Cargo.toml").write_text('[package]\nname="hapin-utils-rs"\nversion="5.0.0"\n')
        out=root/"release-dist";out.mkdir()
        sums=[]
        for target in ("web","nodejs","bundler"):
            path=out/f"hapin-utils-rs-5.0.0-wasm-{target}.tar.gz"
            path.write_bytes(b"fixture archive")
            sums.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}")
        (out/"SHA256SUMS").write_text("\n".join(sums))

    def test_published_release_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as folder:
            root=pathlib.Path(folder);self.assets(root)
            with patch.object(publish,"ROOT",root),patch.dict(os.environ,{"GITHUB_REPOSITORY":"ha-pin/hapin-utils-rs","GITHUB_SHA":"newsha"}),patch.object(publish,"api",return_value={"draft":False}),patch.object(publish,"run") as run:
                publish.main()
                run.assert_not_called()

    def test_tag_for_another_commit_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root=pathlib.Path(folder);self.assets(root)
            with patch.object(publish,"ROOT",root),patch.dict(os.environ,{"GITHUB_REPOSITORY":"ha-pin/hapin-utils-rs","GITHUB_SHA":"newsha"}),patch.object(publish,"api",return_value=None),patch.object(publish,"run",side_effect=["existing tag","","oldsha"]) as run:
                with self.assertRaises(RuntimeError):
                    publish.main()
                self.assertFalse(any("push" in call.args or "gh" in call.args for call in run.call_args_list))

    def test_bad_assets_cannot_create_a_public_tag(self):
        with tempfile.TemporaryDirectory() as folder:
            root=pathlib.Path(folder);self.assets(root)
            (root/"release-dist/hapin-utils-rs-5.0.0-wasm-web.tar.gz").write_bytes(b"corrupted")
            with patch.object(publish,"ROOT",root),patch.dict(os.environ,{"GITHUB_REPOSITORY":"ha-pin/hapin-utils-rs","GITHUB_SHA":"newsha"}),patch.object(publish,"run") as run:
                with self.assertRaises(AssertionError):
                    publish.main()
                run.assert_not_called()

if __name__ == "__main__":
    unittest.main()
