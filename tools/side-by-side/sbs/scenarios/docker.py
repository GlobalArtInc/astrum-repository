import gzip
import hashlib
import io
import json
import os
import shutil
import tarfile

from ..compare import Spec
from ..harness import Skip, scenario
from .common import NONE, group, hosted, rest_listing, settle_search

MANIFEST_V2 = "application/vnd.docker.distribution.manifest.v2+json"
OCI_MANIFEST = "application/vnd.oci.image.manifest.v1+json"
OCI_INDEX = "application/vnd.oci.image.index.v1+json"
CONFIG_V2 = "application/vnd.docker.container.image.v1+json"
LAYER_V2 = "application/vnd.docker.image.rootfs.diff.tar.gzip"
ACCEPT = ", ".join([MANIFEST_V2, OCI_MANIFEST, OCI_INDEX, "application/vnd.docker.distribution.manifest.list.v2+json"])
JSON_BODY = Spec(body="json")
BLOB = Spec(body="bytes")
ERROR = Spec(body="json", error_body=True, transform=lambda v: [e.get("code") for e in v.get("errors", [])] if isinstance(v, dict) else v)


def digest(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


def layer(content):
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w") as archive:
        data = content.encode()
        info = tarfile.TarInfo("sbs/file.txt")
        info.size = len(data)
        info.mtime = 1577836800
        archive.addfile(info, io.BytesIO(data))
    return gzip.compress(raw.getvalue(), mtime=0), raw.getvalue()


def config(diff_ids):
    return json.dumps({"architecture": "amd64", "os": "linux", "config": {"Cmd": ["/sbs/file.txt"]},
                       "rootfs": {"type": "layers", "diff_ids": diff_ids},
                       "created": "2020-01-01T00:00:00Z"}, sort_keys=True).encode()


def manifest(config_blob, layers, media=MANIFEST_V2, subject=None, artifact_type=None):
    out = {"schemaVersion": 2, "mediaType": media,
           "config": {"mediaType": CONFIG_V2 if media == MANIFEST_V2 else "application/vnd.oci.image.config.v1+json",
                      "size": len(config_blob), "digest": digest(config_blob)},
           "layers": [{"mediaType": LAYER_V2 if media == MANIFEST_V2 else "application/vnd.oci.image.layer.v1.tar+gzip",
                       "size": len(data), "digest": digest(data)} for data in layers]}
    if artifact_type:
        out["artifactType"] = artifact_type
        out["config"] = {"mediaType": "application/vnd.oci.empty.v1+json", "size": 2,
                         "digest": digest(b"{}")}
    if subject:
        out["subject"] = subject
    return json.dumps(out, indent=3).encode()


def push_blob(t, step, name, data, record=True):
    started = t.request("POST", "/v2/%s/blobs/uploads/" % name)
    if record:
        t.record_response(step + " start", started, NONE)
    location = started.header("Location")
    if not location:
        return None
    joined = location + ("&" if "?" in location else "?") + "digest=" + digest(data)
    response = t.request("PUT", joined, body=data, headers={"Content-Type": "application/octet-stream"})
    if record:
        t.record_response(step + " finish", response, NONE)
    return response


def dockerd(t):
    t.need("dockerd", "docker")
    state = t.context.state.setdefault("dockerd", {})
    if state.get("socket"):
        return state["socket"]
    import subprocess
    import time
    root = os.path.join(t.context.work, "dockerd")
    os.makedirs(root, exist_ok=True)
    socket = os.path.join(root, "docker.sock")
    log = open(os.path.join(root, "dockerd.log"), "wb")
    process = subprocess.Popen(["dockerd", "--host", "unix://" + socket, "--data-root", os.path.join(root, "data"),
                                "--exec-root", os.path.join(root, "exec"), "--pidfile", os.path.join(root, "dockerd.pid"),
                                "--bridge", "none", "--iptables=false", "--ip6tables=false"],
                               stdout=log, stderr=subprocess.STDOUT)
    t.context.on_close(lambda: stop_dockerd(process, root))
    for _ in range(60):
        if os.path.exists(socket):
            code = subprocess.run(["docker", "--host", "unix://" + socket, "version"], capture_output=True).returncode
            if code == 0:
                state["socket"] = socket
                return socket
        time.sleep(0.5)
    raise Skip("dockerd did not start")


def stop_dockerd(process, root):
    process.terminate()
    try:
        process.wait(timeout=30)
    except Exception:
        process.kill()
    shutil.rmtree(root, ignore_errors=True)


@scenario("docker", formats=["docker"])
def docker(t):
    """docker hosted with path routing: registry API v2 and OCI 1.1 requests, then the docker CLI (login, push, pull)."""
    p = t.prefix
    repo = t.create_repository("docker", "hosted", hosted(p + "docker-hosted", {
        "docker": {"v1Enabled": False, "forceBasicAuth": True, "pathEnabled": True}}), step="create hosted")
    other = t.create_repository("docker", "hosted", hosted(p + "docker-other", {
        "docker": {"v1Enabled": False, "forceBasicAuth": True, "pathEnabled": True}}), step="create other")
    grp = t.create_repository("docker", "group", group(p + "docker-group", [repo], {
        "docker": {"v1Enabled": False, "forceBasicAuth": True, "pathEnabled": True}}), step="create group")
    name = "%s/sbs/app" % repo
    t.step("base", "GET", "/v2/", spec=NONE)
    t.step("base anonymous", "GET", "/v2/", auth=None, spec=NONE)
    t.step("base repository path", "GET", "/repository/%s/v2/" % repo, spec=NONE)
    first, first_raw = layer("one\n")
    second, second_raw = layer("two\n")
    cfg = config(["sha256:" + hashlib.sha256(first_raw).hexdigest(), "sha256:" + hashlib.sha256(second_raw).hexdigest()])
    push_blob(t, "blob layer", name, first)
    push_blob(t, "blob config", name, cfg)
    t.step("monolithic upload", "POST", "/v2/%s/blobs/uploads/?digest=%s" % (name, digest(second)), body=second,
           headers={"Content-Type": "application/octet-stream"}, spec=NONE)
    started = t.step("chunked start", "POST", "/v2/%s/blobs/uploads/" % name, spec=NONE)
    location = started.header("Location") or ""
    chunk = b"chunked blob data " * 100
    patched = t.step("chunked patch 1", "PATCH", location, body=chunk[:1000],
                     headers={"Content-Type": "application/octet-stream", "Content-Range": "0-999"}, spec=NONE)
    location = patched.header("Location") or location
    t.step("chunked patch out of order", "PATCH", location, body=chunk[1000:],
           headers={"Content-Type": "application/octet-stream", "Content-Range": "1200-1799"}, spec=NONE)
    t.step("upload status", "GET", location, spec=NONE)
    patched = t.step("chunked patch 2", "PATCH", location, body=chunk[1000:],
                     headers={"Content-Type": "application/octet-stream", "Content-Range": "1000-%d" % (len(chunk) - 1)},
                     spec=NONE)
    location = patched.header("Location") or location
    t.step("chunked finish", "PUT", location + ("&" if "?" in location else "?") + "digest=" + digest(chunk),
           headers={"Content-Type": "application/octet-stream"}, body=b"", spec=NONE)
    started = t.step("bad digest start", "POST", "/v2/%s/blobs/uploads/" % name, spec=NONE)
    bad = started.header("Location") or ""
    t.step("bad digest finish", "PUT", bad + ("&" if "?" in bad else "?") + "digest=" + digest(b"other"), body=b"data",
           headers={"Content-Type": "application/octet-stream"}, spec=ERROR)
    t.step("cancel upload", "DELETE", bad, spec=NONE)
    t.step("unknown upload", "GET", "/v2/%s/blobs/uploads/00000000-0000-0000-0000-000000000000" % name, spec=ERROR)

    t.step("blob head", "HEAD", "/v2/%s/blobs/%s" % (name, digest(first)), spec=NONE)
    t.step("blob get", "GET", "/v2/%s/blobs/%s" % (name, digest(first)), spec=BLOB)
    t.step("blob range", "GET", "/v2/%s/blobs/%s" % (name, digest(chunk)), headers={"Range": "bytes=0-9"}, spec=BLOB)
    t.step("blob missing", "GET", "/v2/%s/blobs/%s" % (name, digest(b"missing")), spec=ERROR)
    t.step("blob bad digest", "GET", "/v2/%s/blobs/sha256:xyz" % name, spec=NONE)

    body = manifest(cfg, [first, second])
    t.step("manifest put", "PUT", "/v2/%s/manifests/1.0" % name, body=body, headers={"Content-Type": MANIFEST_V2},
           spec=NONE)
    t.step("manifest put same by digest", "PUT", "/v2/%s/manifests/%s" % (name, digest(body)), body=body,
           headers={"Content-Type": MANIFEST_V2}, spec=NONE)
    t.step("manifest put second tag", "PUT", "/v2/%s/manifests/latest" % name, body=body,
           headers={"Content-Type": MANIFEST_V2}, spec=NONE)
    t.step("manifest put missing blob", "PUT", "/v2/%s/manifests/broken" % name,
           body=manifest(cfg, [first, b"not pushed"]), headers={"Content-Type": MANIFEST_V2}, spec=ERROR)
    t.step("manifest put wrong type", "PUT", "/v2/%s/manifests/odd" % name, body=b"{}",
           headers={"Content-Type": "application/json"}, spec=NONE)
    t.step("manifest get", "GET", "/v2/%s/manifests/1.0" % name, headers={"Accept": ACCEPT}, spec=BLOB)
    t.step("manifest head", "HEAD", "/v2/%s/manifests/1.0" % name, headers={"Accept": ACCEPT}, spec=NONE)
    t.step("manifest by digest", "GET", "/v2/%s/manifests/%s" % (name, digest(body)), headers={"Accept": ACCEPT},
           spec=BLOB)
    t.step("manifest no accept", "GET", "/v2/%s/manifests/1.0" % name, spec=NONE)
    t.step("manifest oci only accept", "GET", "/v2/%s/manifests/1.0" % name, headers={"Accept": OCI_MANIFEST}, spec=NONE)
    t.step("manifest missing tag", "GET", "/v2/%s/manifests/9.9" % name, headers={"Accept": ACCEPT}, spec=ERROR)
    t.step("manifest unknown name", "GET", "/v2/%s/sbs/nothing/manifests/1.0" % repo, headers={"Accept": ACCEPT},
           spec=ERROR)
    t.step("manifest through group", "GET", "/v2/%s/sbs/app/manifests/1.0" % grp, headers={"Accept": ACCEPT}, spec=BLOB)
    t.step("blob through group", "HEAD", "/v2/%s/sbs/app/blobs/%s" % (grp, digest(first)), spec=NONE)
    t.step("push to group", "POST", "/v2/%s/sbs/app/blobs/uploads/" % grp, spec=NONE)

    oci_layer, _ = layer("oci\n")
    push_blob(t, "oci layer", name, oci_layer)
    push_blob(t, "oci empty config", name, b"{}")
    oci_cfg = config(["sha256:" + hashlib.sha256(b"x").hexdigest()])
    push_blob(t, "oci config", name, oci_cfg)
    oci = manifest(oci_cfg, [oci_layer], media=OCI_MANIFEST)
    t.step("oci manifest put", "PUT", "/v2/%s/manifests/oci" % name, body=oci, headers={"Content-Type": OCI_MANIFEST},
           spec=NONE)
    index = json.dumps({"schemaVersion": 2, "mediaType": OCI_INDEX, "manifests": [
        {"mediaType": OCI_MANIFEST, "size": len(oci), "digest": digest(oci),
         "platform": {"architecture": "amd64", "os": "linux"}}]}).encode()
    t.step("oci index put", "PUT", "/v2/%s/manifests/multi" % name, body=index, headers={"Content-Type": OCI_INDEX},
           spec=NONE)
    t.step("oci index get", "GET", "/v2/%s/manifests/multi" % name, headers={"Accept": ACCEPT}, spec=BLOB)
    subject = {"mediaType": MANIFEST_V2, "size": len(body), "digest": digest(body)}
    artifact = manifest(b"{}", [oci_layer], media=OCI_MANIFEST, subject=subject,
                        artifact_type="application/vnd.example.sbom")
    t.step("referrer put", "PUT", "/v2/%s/manifests/%s" % (name, digest(artifact)), body=artifact,
           headers={"Content-Type": OCI_MANIFEST}, spec=NONE)
    t.step("referrers", "GET", "/v2/%s/referrers/%s" % (name, digest(body)), spec=JSON_BODY)
    t.step("referrers filtered", "GET", "/v2/%s/referrers/%s?artifactType=application/vnd.example.sbom"
           % (name, digest(body)), spec=JSON_BODY)
    t.step("referrers none", "GET", "/v2/%s/referrers/%s" % (name, digest(oci)), spec=JSON_BODY)

    t.step("tags", "GET", "/v2/%s/tags/list" % name, spec=JSON_BODY)
    t.step("tags page", "GET", "/v2/%s/tags/list?n=1" % name, spec=JSON_BODY)
    t.step("tags after", "GET", "/v2/%s/tags/list?n=2&last=1.0" % name, spec=JSON_BODY)
    t.step("tags unknown", "GET", "/v2/%s/sbs/nothing/tags/list" % repo, spec=ERROR)
    t.step("catalog", "GET", "/v2/_catalog", spec=Spec(body="json", transform=lambda v: sorted(
        r for r in v.get("repositories", []) if r.startswith(p)) if isinstance(v, dict) else v))
    t.step("mount", "POST", "/v2/%s/sbs/mounted/blobs/uploads/?mount=%s&from=%s" % (other, digest(first), name),
           spec=NONE)
    t.step("mount missing", "POST", "/v2/%s/sbs/mounted/blobs/uploads/?mount=%s&from=%s" % (
        other, digest(b"nothing"), name), spec=NONE)
    t.step("delete by tag", "DELETE", "/v2/%s/manifests/latest" % name, spec=NONE)
    t.step("delete manifest", "DELETE", "/v2/%s/manifests/%s" % (name, digest(oci)), spec=NONE)
    t.step("deleted manifest", "GET", "/v2/%s/manifests/oci" % name, headers={"Accept": ACCEPT}, spec=ERROR)
    t.step("delete blob", "DELETE", "/v2/%s/blobs/%s" % (name, digest(chunk)), spec=NONE)
    t.step("tags after delete", "GET", "/v2/%s/tags/list" % name, spec=JSON_BODY)
    rest_listing(t, "rest", repo)
    settle_search(t, repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s&format=docker" % repo)
    cli(t, repo)


def cli(t, repo):
    try:
        socket = dockerd(t)
    except Skip:
        t.fact("docker cli", "skipped")
        return
    user, password = t.client_user()
    registry = t.server.netloc
    env = {"DOCKER_HOST": "unix://" + socket, "DOCKER_CONFIG": t.path("docker-config", "x")[:-2]}
    image = "%s/%s/sbs/cli" % (registry, repo)
    t.normalize_rule(r"sha256:[0-9a-f]{64}", "<digest>")
    rootfs = t.path("rootfs.tar")
    with tarfile.open(rootfs, "w") as archive:
        data = ("built for %s\n" % "side-by-side").encode()
        info = tarfile.TarInfo("hello.txt")
        info.size = len(data)
        info.mtime = 1577836800
        archive.addfile(info, io.BytesIO(data))
    t.run("docker import", ["docker", "import", rootfs, image + ":1.0"], env=env)
    t.run("docker login", ["docker", "login", registry, "-u", user, "--password-stdin"], env=env,
          input=password.encode())
    t.run("docker push", ["docker", "push", "-q", image + ":1.0"], env=env)
    t.run("docker push again", ["docker", "push", "-q", image + ":1.0"], env=env)
    t.run("docker tag", ["docker", "tag", image + ":1.0", image + ":stable"], env=env)
    t.run("docker push tag", ["docker", "push", "-q", image + ":stable"], env=env)
    t.run("docker rmi", ["docker", "rmi", image + ":1.0", image + ":stable"], env=env)
    t.run("docker pull", ["docker", "pull", "-q", image + ":1.0"], env=env)
    t.run("docker pull missing", ["docker", "pull", "-q", image + ":9.9"], env=env)
    code, output = t.run("docker inspect", ["docker", "image", "inspect", "--format", "{{json .RepoDigests}}",
                                            image + ":1.0"], env=env, record=False)
    t.fact("pulled digest", "<digest>" if "sha256:" in output else output.strip())
    t.run("docker logout", ["docker", "logout", registry], env=env)
    t.run("docker pull anonymous", ["docker", "pull", "-q", image + ":1.0"], env=env)
    t.run("docker rmi all", ["docker", "rmi", "-f", image + ":1.0"], env=env, record=False)
