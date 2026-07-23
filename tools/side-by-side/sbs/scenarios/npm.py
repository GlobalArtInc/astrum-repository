import base64
import gzip
import hashlib
import io
import json
import os
import tarfile

from ..compare import Spec
from ..harness import Skip, scenario
from .common import BYTES, NONE, group, hosted, proxy, rest_listing, settle_search

PACKAGE = "@sbs/lib"
ESCAPED = "@sbs%2flib"
DOC = Spec(body="json", volatile=("_rev",))
VOLATILE_DOC = ("_rev", "time", "modified", "created")


def package_json(name, version, extra=None):
    data = {"name": name, "version": version, "description": "side-by-side test package", "main": "index.js",
            "license": "MIT"}
    data.update(extra or {})
    return data


def tarball(name, version, extra=None):
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w", format=tarfile.GNU_FORMAT) as archive:
        for path, content in (("package/package.json", json.dumps(package_json(name, version, extra), indent=2) + "\n"),
                              ("package/index.js", "module.exports = %r;\n" % version)):
            data = content.encode("utf-8")
            info = tarfile.TarInfo(path)
            info.size = len(data)
            info.mtime = 499162500
            info.mode = 0o644
            archive.addfile(info, io.BytesIO(data))
    return gzip.compress(out.getvalue(), mtime=0)


def npmrc(t, registry, auth_urls):
    user, password = t.client_user()
    token = base64.b64encode(("%s:%s" % (user, password)).encode()).decode()
    lines = ["registry=" + registry, "audit=false", "fund=false", "update-notifier=false", "loglevel=warn",
             "cache=" + t.path("npm-cache", "x")[:-2], "prefer-online=true"]
    for url in auth_urls:
        lines.append("//%s:_auth=%s" % (url.split("://", 1)[1], token))
    path = t.path("npmrc")
    with open(path, "w") as handle:
        handle.write("\n".join(lines) + "\n")
    return path


def npm(t, step, rc, *args, cwd=None, record=True):
    env = {"npm_config_userconfig": rc, "npm_config_globalconfig": t.path("global-npmrc"), "HOME": t.path("home", "x")[:-2],
           "NO_UPDATE_NOTIFIER": "1"}
    return t.run(step, ["npm"] + list(args), env=env, cwd=cwd, timeout=300, record=record)


def project(t, name, version):
    folder = t.path("pkg-%s-%s" % (name.replace("/", "_"), version), "package.json")
    with open(folder, "w") as handle:
        json.dump(package_json(name, version), handle, indent=2)
    with open(os.path.join(os.path.dirname(folder), "index.js"), "w") as handle:
        handle.write("module.exports = %r;\n" % version)
    return os.path.dirname(folder)


def npm_spec(method, path):
    if "/-/" not in path and "/-rev/" not in path:
        return Spec(body="json", volatile=VOLATILE_DOC, skip_headers=("etag", "content-length"))
    return None


def keep_npm(method, path):
    return "sbs" in path


def view(t, step, rc, spec_arg):
    code, output = npm(t, step, rc, "view", spec_arg, "--json", record=False)
    try:
        value = json.loads(output[output.index("{"):]) if "{" in output else output.strip()
    except ValueError:
        value = output.strip()
    if isinstance(value, dict):
        value = {k: v for k, v in value.items() if k not in ("time", "_rev", "_cached", "_contentLength", "modified",
                                                             "created")}
    t.fact(step, {"code": code, "value": value}, spec=Spec(body="json", volatile=VOLATILE_DOC))


@scenario("npm", formats=["npm"], clients=["npm"])
def npm_scenario(t):
    """npm hosted and group: npm publish, view, dist-tag, deprecate, install, unpublish; package documents."""
    p = t.prefix
    repo = t.create_repository("npm", "hosted", hosted(p + "npm-hosted"), step="create hosted")
    grp = t.create_repository("npm", "group", group(p + "npm-group", [repo]), step="create group")
    base = "/repository/%s/" % repo
    t.step("put document", "PUT", base + "sbs-direct", spec=NONE, headers={"Content-Type": "application/json"},
           body=json.dumps(publish_document("sbs-direct", "1.0.0", tarball("sbs-direct", "1.0.0"), t.server.base + base)))
    t.step("direct document", "GET", base + "sbs-direct", spec=Spec(body="json", volatile=VOLATILE_DOC))
    t.step("direct tarball", "GET", base + "sbs-direct/-/sbs-direct-1.0.0.tgz", spec=BYTES)
    t.need("npm")
    with t.recording("npm", spec=npm_spec, keep=keep_npm) as proxied:
        hosted_url = "%s/repository/%s/" % (proxied, repo)
        group_url = "%s/repository/%s/" % (proxied, grp)
        rc = npmrc(t, group_url, [hosted_url, group_url])
        for version in ("1.0.0", "1.1.0", "2.0.0-beta.1"):
            folder = project(t, PACKAGE, version)
            args = ["publish", "--registry", hosted_url]
            if "beta" in version:
                args += ["--tag", "next"]
            npm(t, "publish " + version, rc, *args, cwd=folder)
        npm(t, "publish again", rc, "publish", "--registry", hosted_url, cwd=project(t, PACKAGE, "1.0.0"))
        npm(t, "publish unscoped", rc, "publish", "--registry", hosted_url, cwd=project(t, "sbs-tool", "0.1.0"))
        view(t, "view", rc, PACKAGE)
        view(t, "view version", rc, PACKAGE + "@1.0.0")
        npm(t, "dist-tag add", rc, "dist-tag", "add", PACKAGE + "@1.0.0", "stable", "--registry", hosted_url)
        code, output = npm(t, "dist-tag ls", rc, "dist-tag", "ls", PACKAGE, record=False)
        t.fact("dist-tag ls", {"code": code, "tags": sorted(output.split("\n"))})
        npm(t, "dist-tag rm", rc, "dist-tag", "rm", PACKAGE, "stable", "--registry", hosted_url)
        npm(t, "deprecate", rc, "deprecate", PACKAGE + "@1.1.0", "use 2.x", "--registry", hosted_url)
        view(t, "view after deprecate", rc, PACKAGE + "@1.1.0")
        app = project(t, "sbs-app", "0.0.1")
        npm(t, "install", rc, "install", PACKAGE + "@1.0.0", "--no-package-lock", cwd=app)
        installed = os.path.join(app, "node_modules", "@sbs", "lib", "package.json")
        t.fact("installed version", json.load(open(installed)).get("version") if os.path.exists(installed) else None)
        npm(t, "install missing", rc, "install", "@sbs/nothing", "--no-package-lock", cwd=app)
        npm(t, "unpublish version", rc, "unpublish", PACKAGE + "@2.0.0-beta.1", "--force", "--registry", hosted_url)
        view(t, "view after unpublish", rc, PACKAGE)
        npm(t, "ping", rc, "ping", "--registry", hosted_url)
        code, output = npm(t, "whoami", rc, "whoami", "--registry", hosted_url, record=False)
        t.fact("whoami", {"code": code, "user": output.strip().splitlines()[-1] if output.strip() else ""})
        npm(t, "search", rc, "search", "sbs", "--registry", hosted_url)

    for label, path, headers in (("document", ESCAPED, {}), ("document abbreviated", ESCAPED,
                                                            {"Accept": "application/vnd.npm.install-v1+json"}),
                                 ("document group", None, {}), ("version document", ESCAPED + "/1.0.0", {}),
                                 ("dist-tags", "-/package/%s/dist-tags" % ESCAPED, {})):
        target = (base + path) if path else "/repository/%s/%s" % (grp, ESCAPED)
        t.step(label, "GET", target, headers=headers, spec=Spec(body="json", volatile=VOLATILE_DOC))
    t.step("tarball", "GET", base + "%s/-/lib-1.0.0.tgz" % PACKAGE, spec=BYTES)
    t.step("tarball head", "HEAD", base + "%s/-/lib-1.0.0.tgz" % PACKAGE, spec=NONE)
    t.step("tarball through group", "GET", "/repository/%s/%s/-/lib-1.0.0.tgz" % (grp, PACKAGE), spec=BYTES)
    t.step("missing package", "GET", base + "@sbs%2fnothing", spec=NONE)
    t.step("missing tarball", "GET", base + "%s/-/lib-9.9.9.tgz" % PACKAGE, spec=NONE)
    t.step("search endpoint", "GET", base + "-/v1/search?text=sbs&size=20", spec=Spec(body="json"))
    t.step("audit hosted", "POST", base + "-/npm/v1/security/audits/quick", body=b"{}",
           headers={"Content-Type": "application/json"}, spec=NONE)
    t.step("publish to group", "PUT", "/repository/%s/sbs-group-pkg" % grp, spec=NONE,
           headers={"Content-Type": "application/json"},
           body=json.dumps(publish_document("sbs-group-pkg", "1.0.0", tarball("sbs-group-pkg", "1.0.0"), t.server.base)))
    rest_listing(t, "rest", repo, generated=lambda path: "/-/" not in path)
    settle_search(t, repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s&npm.scope=sbs" % repo)


def publish_document(name, version, data, base):
    filename = "%s-%s.tgz" % (name.split("/")[-1], version)
    manifest = package_json(name, version)
    manifest["_id"] = "%s@%s" % (name, version)
    manifest["dist"] = {"tarball": "%s%s/-/%s" % (base, name, filename)}
    manifest["dist"]["shasum"] = hashlib.sha1(data).hexdigest()
    return {"_id": name, "name": name, "description": manifest["description"], "dist-tags": {"latest": version},
            "versions": {version: manifest},
            "_attachments": {filename: {"content_type": "application/octet-stream", "data": base64.b64encode(data).decode(),
                                        "length": len(data)}}}


@scenario("npm-proxy", formats=["npm"], clients=["npm"], proxies=True)
def npm_proxy(t):
    """npm proxy of a local upstream registry and a group over it: documents, tarballs, npm install."""
    if not t.context.proxies:
        raise Skip("proxies to 127.0.0.1 are not allowed")
    up = t.context.upstream()
    data = tarball("sbs-remote", "3.0.0")
    document = {"_id": "sbs-remote", "name": "sbs-remote", "dist-tags": {"latest": "3.0.0"},
                "versions": {"3.0.0": dict(package_json("sbs-remote", "3.0.0"), _id="sbs-remote@3.0.0",
                                           dist={"tarball": up.base + "/npm/sbs-remote/-/sbs-remote-3.0.0.tgz",
                                                 "shasum": hashlib.sha1(data).hexdigest()})},
                "time": {"3.0.0": "2020-01-01T00:00:00.000Z"}}
    up.add("/npm/sbs-remote", json.dumps(document), "application/json")
    up.add("/npm/sbs-remote/-/sbs-remote-3.0.0.tgz", data, "application/octet-stream")
    p = t.prefix
    repo = t.create_repository("npm", "proxy", proxy(p + "npm-proxy", up.base + "/npm/"), step="create proxy")
    grp = t.create_repository("npm", "group", group(p + "npm-proxy-group", [repo]), step="create group")
    t.step("document", "GET", "/repository/%s/sbs-remote" % repo, spec=Spec(body="json", volatile=VOLATILE_DOC))
    t.step("tarball", "GET", "/repository/%s/sbs-remote/-/sbs-remote-3.0.0.tgz" % repo, spec=BYTES)
    t.step("group document", "GET", "/repository/%s/sbs-remote" % grp, spec=Spec(body="json", volatile=VOLATILE_DOC))
    t.step("missing", "GET", "/repository/%s/sbs-nothing" % repo, spec=NONE)
    t.need("npm")
    with t.recording("npm", spec=npm_spec, keep=keep_npm) as proxied:
        rc = npmrc(t, "%s/repository/%s/" % (proxied, grp), ["%s/repository/%s/" % (proxied, grp)])
        app = project(t, "sbs-proxy-app", "0.0.1")
        npm(t, "install", rc, "install", "sbs-remote", "--no-package-lock", cwd=app)
    rest_listing(t, "rest", repo, generated=lambda path: "/-/" not in path)
