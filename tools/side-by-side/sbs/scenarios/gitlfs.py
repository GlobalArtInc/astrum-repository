import hashlib
import json
import os

from ..compare import Spec
from ..harness import scenario
from .common import BYTES, NONE, hosted, rest_listing

LFS = "application/vnd.git-lfs+json"
JSON = Spec(body="json", volatile=("expires_at", "expires_in", "locked_at", "href", "id"))


def git(t, step, args, cwd, env, record=True):
    return t.run(step, ["git"] + args, cwd=cwd, env=env, record=record)


@scenario("gitlfs", formats=["gitlfs"], clients=["git", "git-lfs"])
def gitlfs(t):
    """git lfs hosted: batch API, uploads, verify, downloads, locks; then git-lfs push, clone and lock commands."""
    p = t.prefix
    repo = t.create_repository("gitlfs", "hosted", hosted(p + "lfs-hosted"), step="create hosted")
    base = "/repository/%s/info/lfs/" % repo
    data = b"large file content from the side-by-side harness\n" * 50
    oid = hashlib.sha256(data).hexdigest()
    headers = {"Accept": LFS, "Content-Type": LFS}
    batch = {"operation": "upload", "transfers": ["basic"], "objects": [{"oid": oid, "size": len(data)}]}
    response = t.step("batch upload", "POST", base + "objects/batch", body=json.dumps(batch), headers=headers, spec=JSON)
    action = ((response.json() or {}).get("objects") or [{}])[0].get("actions", {}).get("upload", {}) \
        if response.status == 200 else {}
    target = action.get("href") or (t.server.base + base + "objects/" + oid)
    t.step("upload object", "PUT", target, body=data, headers={"Content-Type": "application/octet-stream"}, spec=NONE)
    t.step("upload wrong content", "PUT", t.server.base + base + "objects/" + "0" * 64, body=b"wrong",
           headers={"Content-Type": "application/octet-stream"}, spec=NONE)
    t.step("verify", "POST", base + "objects/verify", body=json.dumps({"oid": oid, "size": len(data)}), headers=headers,
           spec=NONE)
    t.step("verify missing", "POST", base + "objects/verify", body=json.dumps({"oid": "1" * 64, "size": 1}),
           headers=headers, spec=NONE)
    batch["operation"] = "download"
    batch["objects"].append({"oid": "2" * 64, "size": 3})
    t.step("batch download", "POST", base + "objects/batch", body=json.dumps(batch), headers=headers, spec=JSON)
    t.step("download object", "GET", base + "objects/" + oid, spec=BYTES)
    t.step("download missing", "GET", base + "objects/" + "3" * 64, spec=NONE)
    t.step("batch bad operation", "POST", base + "objects/batch", body=json.dumps({"operation": "nope", "objects": []}),
           headers=headers, spec=NONE)
    t.step("batch anonymous", "POST", base + "objects/batch", body=json.dumps(batch), headers=headers, auth=None, spec=NONE)
    created = t.step("lock create", "POST", base + "locks", body=json.dumps({"path": "big/file.bin"}), headers=headers,
                     spec=JSON)
    t.step("lock create again", "POST", base + "locks", body=json.dumps({"path": "big/file.bin"}), headers=headers,
           spec=Spec(body="json", volatile=("id", "locked_at")))
    t.step("locks", "GET", base + "locks", headers={"Accept": LFS}, spec=JSON)
    t.step("locks by path", "GET", base + "locks?path=big/file.bin", headers={"Accept": LFS}, spec=JSON)
    t.step("locks verify", "POST", base + "locks/verify", body=b"{}", headers=headers, spec=JSON)
    lock_id = ((created.json() or {}).get("lock") or {}).get("id") if created.status == 201 else None
    if lock_id:
        t.step("unlock", "POST", base + "locks/%s/unlock" % lock_id, body=b"{}", headers=headers, spec=JSON)
    t.step("unlock missing", "POST", base + "locks/999999/unlock", body=b"{}", headers=headers, spec=NONE)
    rest_listing(t, "rest", repo)
    cli(t, repo)


def cli(t, repo):
    t.need("git", "git-lfs")
    user, password = t.client_user()
    home = t.path("home", "x")[:-2]
    env = {"HOME": home, "GIT_CONFIG_NOSYSTEM": "1", "GIT_TERMINAL_PROMPT": "0", "GIT_AUTHOR_NAME": "sbs",
           "GIT_AUTHOR_EMAIL": "sbs@example.invalid", "GIT_COMMITTER_NAME": "sbs",
           "GIT_COMMITTER_EMAIL": "sbs@example.invalid", "GIT_LFS_SKIP_PUSH_ACTIONS": ""}
    with open(os.path.join(home, ".git-credentials"), "w") as handle:
        handle.write("http://%s:%s@%s\n" % (user, password, t.server.netloc))
    os.chmod(os.path.join(home, ".git-credentials"), 0o600)
    t.normalize_rule(r"/locks/\d+/", "/locks/<id>/")
    with t.recording("git-lfs", spec=lambda m, p: JSON if "/locks" in p else None,
                     keep=lambda m, p: "/info/lfs" in p) as proxied:
        with open(os.path.join(home, ".git-credentials"), "a") as handle:
            handle.write("http://%s:%s@%s\n" % (user, password, proxied.split("://", 1)[1]))
        lfs_url = "%s/repository/%s/info/lfs" % (proxied, repo)
        bare = t.path("origin.git", "x")[:-2]
        work = t.path("work", "x")[:-2]
        git(t, "init bare", ["init", "-q", "--bare", "-b", "main", bare], t.work, env, record=False)
        git(t, "init", ["init", "-q", "-b", "main", work], t.work, env, record=False)
        for args in (["config", "credential.helper", "store"], ["config", "lfs.url", lfs_url],
                     ["config", "lfs.locksverify", "true"], ["lfs", "install", "--local"],
                     ["lfs", "track", "*.bin"], ["remote", "add", "origin", bare]):
            git(t, "setup", args, work, env, record=False)
        with open(os.path.join(work, "model.bin"), "wb") as handle:
            handle.write(b"model weights " * 4000)
        git(t, "add", ["add", ".gitattributes", "model.bin"], work, env, record=False)
        git(t, "commit", ["commit", "-q", "-m", "add model"], work, env, record=False)
        git(t, "push", ["push", "-q", "origin", "main"], work, env)
        git(t, "lfs lock", ["lfs", "lock", "model.bin"], work, env)
        code, output = git(t, "lfs locks", ["lfs", "locks"], work, env, record=False)
        t.fact("lfs locks", {"code": code, "count": sum(1 for l in output.splitlines() if "model.bin" in l)})
        git(t, "lfs unlock", ["lfs", "unlock", "model.bin"], work, env)
        clone = t.path("clone", "x")[:-2]
        clone_env = dict(env, GIT_LFS_SKIP_SMUDGE="1")
        git(t, "clone", ["clone", "-q", bare, clone], t.work, clone_env, record=False)
        git(t, "clone config", ["config", "lfs.url", lfs_url], clone, env, record=False)
        git(t, "clone credentials", ["config", "credential.helper", "store"], clone, env, record=False)
        git(t, "lfs pull", ["lfs", "pull"], clone, env)
        pulled = os.path.join(clone, "model.bin")
        t.fact("lfs pulled content", open(pulled, "rb").read() == b"model weights " * 4000 if os.path.exists(pulled) else None)
