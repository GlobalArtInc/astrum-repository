import base64
import gzip
import hashlib
import io
import os
import re
import tarfile
import urllib.parse
import zipfile

from ..compare import Spec
from ..harness import Skip, scenario
from .common import BYTES, NONE, group, hosted, multipart, proxy, rest_listing, settle_search, without_assets

NAME = "sbs-lib"
def links(text):
    found = []
    for tag in re.findall(r"<a\s[^>]*>", text):
        href = re.search(r'href="([^"]*)"', tag)
        name = href.group(1).split("#")[0].rsplit("/", 1)[-1] if href else ""
        digest = href.group(1).split("#", 1)[1] if href and "#" in href.group(1) else ""
        extras = sorted(re.findall(r'(data-[a-z-]+)="([^"]*)"', tag))
        found.append("%s %s %s" % (name, digest, " ".join("%s=%s" % e for e in extras)))
    return "\n".join(sorted(found))


SIMPLE = Spec(body="text", transform=links, skip_headers=("etag", "content-length"))


def metadata(name, version):
    return ("Metadata-Version: 2.1\nName: %s\nVersion: %s\nSummary: side-by-side test package\nRequires-Python: >=3.8\n"
            "License: MIT\n\n" % (name, version))


def wheel(name, version):
    module = name.replace("-", "_")
    dist = "%s-%s.dist-info" % (module, version)
    files = {
        "%s/__init__.py" % module: "VERSION = %r\n" % version,
        "%s/METADATA" % dist: metadata(name, version),
        "%s/WHEEL" % dist: "Wheel-Version: 1.0\nGenerator: side-by-side\nRoot-Is-Purelib: true\nTag: py3-none-any\n",
    }
    record = []
    for path, content in files.items():
        digest = hashlib.sha256(content.encode()).digest()
        record.append("%s,sha256=%s,%d" % (path, base64.urlsafe_b64encode(digest).rstrip(b"=").decode(), len(content)))
    record.append("%s/RECORD,," % dist)
    files["%s/RECORD" % dist] = "\n".join(record) + "\n"
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
        for path, content in files.items():
            archive.writestr(zipfile.ZipInfo(path, (2020, 1, 1, 0, 0, 0)), content)
    return "%s-%s-py3-none-any.whl" % (module, version), out.getvalue()


def sdist(name, version):
    root = "%s-%s" % (name.replace("-", "_"), version)
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w") as archive:
        for path, content in (("PKG-INFO", metadata(name, version)),
                              ("setup.py", "from setuptools import setup\nsetup(name=%r, version=%r)\n" % (name, version))):
            data = content.encode()
            info = tarfile.TarInfo("%s/%s" % (root, path))
            info.size = len(data)
            info.mtime = 1577836800
            archive.addfile(info, io.BytesIO(data))
    return root + ".tar.gz", gzip.compress(out.getvalue(), mtime=0)


def upload(t, step, repository, filename, data, filetype, name, version, override=None):
    fields = [(":action", "file_upload"), ("protocol_version", "1"), ("metadata_version", "2.1"), ("name", name),
              ("version", version), ("filetype", filetype), ("pyversion", "py3" if filetype == "bdist_wheel" else "source"),
              ("summary", "side-by-side test package"), ("requires_python", ">=3.8"),
              ("md5_digest", hashlib.md5(data).hexdigest()), ("sha256_digest", hashlib.sha256(data).hexdigest()),
              ("blake2_256_digest", hashlib.blake2b(data, digest_size=32).hexdigest())]
    fields = [(k, (override or {}).get(k, v)) for k, v in fields]
    body, ctype = multipart(fields, [("content", filename, data, "application/octet-stream")])
    return t.step(step, "POST", "/repository/%s/" % repository, body=body, headers={"Content-Type": ctype}, spec=NONE)


def pip(t, step, index, *args):
    env = {"PIP_CONFIG_FILE": os.devnull, "PIP_NO_INPUT": "1", "PIP_DISABLE_PIP_VERSION_CHECK": "1",
           "PIP_CACHE_DIR": t.path("pip-cache", "x")[:-2], "HOME": t.path("home", "x")[:-2], "PIP_INDEX_URL": index,
           "PIP_RETRIES": "0", "PIP_TIMEOUT": "30"}
    return t.run(step, ["python3", "-I", "-m", "pip"] + list(args), env=env, timeout=300)


def pip_spec(method, path):
    if "/simple/" in path:
        return Spec(body="none", skip_headers=("etag", "content-length", "last-modified"))
    return None


def keep_pypi(method, path):
    return "sbs" in path.lower()


@scenario("pypi", formats=["pypi"], clients=["pip"])
def pypi(t):
    """pypi hosted and group: uploads as twine sends them, simple index pages (HTML and JSON), pip download."""
    p = t.prefix
    repo = t.create_repository("pypi", "hosted", hosted(p + "pypi-hosted"), step="create hosted")
    grp = t.create_repository("pypi", "group", group(p + "pypi-group", [repo]), step="create group")
    files = {}
    for version in ("1.0.0", "1.1.0"):
        for builder, filetype in ((wheel, "bdist_wheel"), (sdist, "sdist")):
            filename, data = builder(NAME, version)
            files[filename] = data
            upload(t, "upload %s %s" % (filetype, version), repo, filename, data, filetype, NAME, version)
    filename, data = wheel(NAME, "1.0.0")
    upload(t, "upload again", repo, filename, data, "bdist_wheel", NAME, "1.0.0")
    upload(t, "upload bad digest", repo, "sbs_lib-1.2.0-py3-none-any.whl", wheel(NAME, "1.2.0")[1], "bdist_wheel", NAME,
           "1.2.0", override={"sha256_digest": "0" * 64})
    upload(t, "upload to group", grp, "sbs_lib-1.3.0-py3-none-any.whl", wheel(NAME, "1.3.0")[1], "bdist_wheel", NAME,
           "1.3.0")
    t.step("upload without action", "POST", "/repository/%s/" % repo, body=b"x",
           headers={"Content-Type": "application/x-www-form-urlencoded"}, spec=NONE)
    for label, target in (("hosted", repo), ("group", grp)):
        base = "/repository/%s/simple/" % target
        t.step("%s simple root" % label, "GET", base, spec=SIMPLE)
        t.step("%s simple project" % label, "GET", base + NAME + "/", spec=SIMPLE)
        t.step("%s simple project normalized" % label, "GET", base + "SBS_Lib/", spec=NONE)
        t.step("%s simple project without slash" % label, "GET", base + NAME, spec=NONE)
        t.step("%s simple json" % label, "GET", base + NAME + "/",
               headers={"Accept": "application/vnd.pypi.simple.v1+json"}, spec=Spec(body="json"))
        t.step("%s simple missing" % label, "GET", base + "sbs-nothing/", spec=NONE)
    page = t.request("GET", "/repository/%s/simple/%s/" % (repo, NAME)).text()
    page_url = t.server.base + "/repository/%s/simple/%s/" % (repo, NAME)
    for href in sorted(re.findall(r'href="([^"]+)"', page))[:2]:
        target = urllib.parse.urljoin(page_url, href.split("#")[0])
        t.step("package link " + target.rsplit("/", 1)[-1], "GET", target, spec=BYTES)
    t.step("package path", "GET", "/repository/%s/packages/%s/1.0.0/sbs_lib-1.0.0-py3-none-any.whl" % (repo, NAME),
           spec=BYTES)
    t.step("package metadata", "GET", "/repository/%s/packages/%s/1.0.0/sbs_lib-1.0.0-py3-none-any.whl.metadata"
           % (repo, NAME), spec=NONE)
    t.step("pypi json api", "GET", "/repository/%s/pypi/%s/json" % (repo, NAME), spec=Spec(body="json"))
    t.step("search xmlrpc", "POST", "/repository/%s/pypi" % repo, spec=NONE, headers={"Content-Type": "text/xml"},
           body=b"<?xml version='1.0'?><methodCall><methodName>search</methodName><params><param><value><struct>"
                b"<member><name>name</name><value><array><data><value><string>sbs</string></value></data></array>"
                b"</value></member></struct></value></param></params></methodCall>")
    with t.recording("pip", spec=pip_spec, keep=keep_pypi) as proxied:
        user, password = t.client_user()
        index = proxied.replace("://", "://%s:%s@" % (user, password)) + "/repository/%s/simple" % grp
        pip(t, "pip download", index, "download", "--no-deps", "-d", t.path("dl", "x")[:-2], NAME + "==1.0.0")
        pip(t, "pip download sdist", index, "download", "--no-deps", "--no-binary", ":all:", "--no-build-isolation",
            "-d", t.path("dl2", "x")[:-2], NAME + "==1.1.0")
        pip(t, "pip download missing", index, "download", "--no-deps", "-d", t.path("dl3", "x")[:-2], "sbs-nothing")
        pip(t, "pip index versions", index, "index", "versions", NAME)
    t.fact("pip downloaded", sorted(os.listdir(t.path("dl", "x")[:-2])))
    rest_listing(t, "rest", repo, derived=lambda path: "/simple/" in path or path.endswith(".metadata"))
    settle_search(t, repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s&name=%s" % (repo, NAME),
                transform=without_assets(lambda path: "/simple/" in path or path.endswith(".metadata")))


@scenario("pypi-proxy", formats=["pypi"], clients=["pip"], proxies=True)
def pypi_proxy(t):
    """pypi proxy of a local simple index and a group over it."""
    if not t.context.proxies:
        raise Skip("proxies to 127.0.0.1 are not allowed")
    up = t.context.upstream()
    filename, data = wheel("sbs-remote", "2.0.0")
    digest = hashlib.sha256(data).hexdigest()
    up.add("/pypi/simple/sbs-remote/", '<!DOCTYPE html><html><body><a href="../../packages/%s#sha256=%s">%s</a></body></html>'
           % (filename, digest, filename), "text/html")
    up.add("/pypi/packages/" + filename, data, "application/octet-stream")
    p = t.prefix
    repo = t.create_repository("pypi", "proxy", proxy(p + "pypi-proxy", up.base + "/pypi/"), step="create proxy")
    grp = t.create_repository("pypi", "group", group(p + "pypi-proxy-group", [repo]), step="create group")
    t.step("simple project", "GET", "/repository/%s/simple/sbs-remote/" % repo, spec=SIMPLE)
    t.step("group simple project", "GET", "/repository/%s/simple/sbs-remote/" % grp, spec=SIMPLE)
    t.step("missing project", "GET", "/repository/%s/simple/sbs-nothing/" % repo, spec=NONE)
    with t.recording("pip", spec=pip_spec, keep=keep_pypi) as proxied:
        user, password = t.client_user()
        index = proxied.replace("://", "://%s:%s@" % (user, password)) + "/repository/%s/simple" % grp
        pip(t, "pip download", index, "download", "--no-deps", "-d", t.path("dl", "x")[:-2], "sbs-remote")
    rest_listing(t, "rest", repo, derived=lambda path: "/simple/" in path or path.endswith(".metadata"))
