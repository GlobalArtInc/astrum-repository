import gzip
import hashlib
import os

from ..harness import Skip, scenario
from .common import BYTES, NONE, group, hosted, proxy, rest_listing

TEXT = b"plain text from the side-by-side harness\n"
BINARY = gzip.compress(bytes((i * 7) % 256 for i in range(200000)), mtime=0)
UNDETECTABLE = bytes((i * 13) % 256 for i in range(4096))


@scenario("raw", formats=["raw"])
def raw(t):
    """raw hosted (ALLOW, ALLOW_ONCE, DENY), strict content types, ranges, conditional GETs, group, curl."""
    p = t.prefix
    repo = t.create_repository("raw", "hosted", hosted(p + "raw-hosted", {"raw": {"contentDisposition": "ATTACHMENT"}}),
                               step="create hosted")
    once = t.create_repository("raw", "hosted", hosted(p + "raw-once", write_policy="ALLOW_ONCE"), step="create once")
    deny = t.create_repository("raw", "hosted", hosted(p + "raw-deny", write_policy="DENY"), step="create deny")
    loose = t.create_repository("raw", "hosted", hosted(p + "raw-loose", strict=False,
                                                        fmt_extra={"raw": {"contentDisposition": "INLINE"}}),
                                step="create not strict")
    grp = t.create_repository("raw", "group", group(p + "raw-group", [repo, loose]), step="create group")

    base = "/repository/%s/" % repo
    t.step("put text", "PUT", base + "dir/a.txt", body=TEXT, headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("put binary", "PUT", base + "dir/b.gz", body=BINARY, headers={"Content-Type": "application/gzip"},
           spec=NONE)
    t.step("put undetectable binary", "PUT", base + "dir/u.bin", body=UNDETECTABLE,
           headers={"Content-Type": "application/octet-stream"}, spec=NONE)
    t.step("put no content type", "PUT", base + "dir/c.txt", body=TEXT, spec=NONE)
    t.step("put deep", "PUT", base + "x/y/z/deep.txt", body=TEXT, headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("put space name", "PUT", base + "dir/with%20space.txt", body=TEXT, headers={"Content-Type": "text/plain"},
           spec=NONE)
    t.step("put again", "PUT", base + "dir/a.txt", body=TEXT, headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("put mismatched type", "PUT", base + "dir/image.png", body=TEXT, headers={"Content-Type": "image/png"},
           spec=NONE)
    t.step("put png named txt", "PUT", base + "dir/fake.txt", body=b"\x89PNG\r\n\x1a\n" + bytes(64),
           headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("put empty", "PUT", base + "dir/empty.txt", body=b"", headers={"Content-Type": "text/plain"}, spec=NONE)

    t.step("get text", "GET", base + "dir/a.txt", spec=BYTES)
    t.step("get binary", "GET", base + "dir/b.gz", spec=BYTES)
    t.step("get undetectable binary", "GET", base + "dir/u.bin", spec=NONE)
    t.step("get no content type", "GET", base + "dir/c.txt", spec=BYTES)
    t.step("get space name", "GET", base + "dir/with%20space.txt", spec=BYTES)
    t.step("get empty", "GET", base + "dir/empty.txt", spec=BYTES)
    t.step("head text", "HEAD", base + "dir/a.txt", spec=NONE)
    t.step("head binary", "HEAD", base + "dir/b.gz", spec=NONE)
    t.step("head missing", "HEAD", base + "dir/missing.txt", spec=NONE)
    t.step("get missing", "GET", base + "dir/missing.txt", spec=NONE)
    t.step("get folder", "GET", base + "dir/", spec=NONE)
    t.step("get folder without slash", "GET", base + "dir", spec=NONE)
    t.step("range first", "GET", base + "dir/b.gz", headers={"Range": "bytes=0-99"}, spec=BYTES)
    t.step("range suffix", "GET", base + "dir/b.gz", headers={"Range": "bytes=-10"}, spec=BYTES)
    t.step("range open", "GET", base + "dir/b.gz", headers={"Range": "bytes=%d-" % (len(BINARY) - 10)}, spec=BYTES)
    t.step("range beyond", "GET", base + "dir/b.gz", headers={"Range": "bytes=%d-%d" % (len(BINARY) + 10, len(BINARY) + 20)}, spec=NONE)
    t.step("range multiple", "GET", base + "dir/b.gz", headers={"Range": "bytes=0-1,5-6"}, spec=NONE)
    t.step("range invalid", "GET", base + "dir/b.gz", headers={"Range": "lines=1-2"}, spec=BYTES)
    first = t.request("GET", base + "dir/a.txt")
    etag = first.header("ETag")
    modified = first.header("Last-Modified")
    t.fact("etag is quoted sha1", etag == '"%s"' % hashlib.sha1(TEXT).hexdigest())
    if etag:
        t.step("if-none-match", "GET", base + "dir/a.txt", headers={"If-None-Match": etag}, spec=NONE)
        t.step("if-none-match other", "GET", base + "dir/a.txt", headers={"If-None-Match": '"0"'}, spec=BYTES)
        t.step("if-match other", "GET", base + "dir/a.txt", headers={"If-Match": '"0"'}, spec=NONE)
    if modified:
        t.step("if-modified-since", "GET", base + "dir/a.txt", headers={"If-Modified-Since": modified}, spec=NONE)
        t.step("if-unmodified-since past", "GET", base + "dir/a.txt",
               headers={"If-Unmodified-Since": "Mon, 01 Jan 2001 00:00:00 GMT"}, spec=NONE)
        t.step("if-range", "GET", base + "dir/a.txt", headers={"Range": "bytes=0-3", "If-Range": modified}, spec=BYTES)
    t.step("get through group", "GET", "/repository/%s/dir/a.txt" % grp, spec=BYTES)
    t.step("head through group", "HEAD", "/repository/%s/dir/a.txt" % grp, spec=NONE)
    t.step("missing through group", "GET", "/repository/%s/dir/missing.txt" % grp, spec=NONE)
    t.step("put to group", "PUT", "/repository/%s/dir/g.txt" % grp, body=TEXT, headers={"Content-Type": "text/plain"},
           spec=NONE)

    t.step("loose put mismatched type", "PUT", "/repository/%s/dir/image.png" % loose, body=TEXT,
           headers={"Content-Type": "image/png"}, spec=NONE)
    t.step("loose get mismatched type", "GET", "/repository/%s/dir/image.png" % loose, spec=BYTES)
    t.step("loose get through group", "GET", "/repository/%s/dir/image.png" % grp, spec=BYTES)

    t.step("once first put", "PUT", "/repository/%s/f.txt" % once, body=TEXT, headers={"Content-Type": "text/plain"},
           spec=NONE)
    t.step("once second put", "PUT", "/repository/%s/f.txt" % once, body=TEXT, headers={"Content-Type": "text/plain"},
           spec=NONE)
    t.step("once delete", "DELETE", "/repository/%s/f.txt" % once, spec=NONE)
    t.step("deny put", "PUT", "/repository/%s/f.txt" % deny, body=TEXT, headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("deny delete missing", "DELETE", "/repository/%s/f.txt" % deny, spec=NONE)

    t.step("post to hosted", "POST", base + "dir/post.txt", body=TEXT, headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("options", "OPTIONS", base + "dir/a.txt", spec=NONE)
    t.step("delete", "DELETE", base + "dir/c.txt", spec=NONE)
    t.step("delete again", "DELETE", base + "dir/c.txt", spec=NONE)
    t.step("get deleted", "GET", base + "dir/c.txt", spec=NONE)
    t.step("delete folder", "DELETE", base + "x/", spec=NONE)
    rest_listing(t, "rest", repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s&name=dir/a.txt" % repo)
    rest_listing(t, "group rest", grp)
    listed = sorted(t.server.get_json("/service/rest/v1/assets?repository=" + grp)["items"],
                    key=lambda asset: asset.get("path", ""))
    if listed:
        t.json_step("group asset by id", "/service/rest/v1/assets/" + listed[0]["id"])

    curl(t, repo)


def curl(t, repo):
    try:
        t.need("curl")
    except Skip:
        return
    user, password = t.client_user()
    url = t.repo_url(repo)
    upload = t.path("upload.gz")
    with open(upload, "wb") as handle:
        handle.write(BINARY)
    netrc = t.path("netrc")
    with open(netrc, "w") as handle:
        handle.write("machine %s login %s password %s\n" % (t.server.host, user, password))
    os.chmod(netrc, 0o600)
    t.run("curl upload", ["curl", "-sS", "-f", "--netrc-file", netrc, "-T", upload, url + "curl/upload.gz"])
    t.run("curl download", ["curl", "-sS", "-f", "--netrc-file", netrc, "-o", t.path("down.bin"), url + "curl/upload.gz"])
    t.fact("curl download same bytes", read(t.path("down.bin")) == BINARY)
    t.run("curl missing", ["curl", "-sS", "-f", "--netrc-file", netrc, "-o", t.path("missing.bin"), url + "curl/missing"])
    t.run("curl anonymous", ["curl", "-sS", "-f", "-o", t.path("anon.bin"), url + "curl/upload.gz"])
    t.run("curl resume", ["curl", "-sS", "-f", "--netrc-file", netrc, "-C", "-", "-o", t.path("down.bin"),
                          url + "curl/upload.gz"])


def read(path):
    try:
        with open(path, "rb") as handle:
            return handle.read()
    except OSError:
        return None


@scenario("raw-proxy", formats=["raw"], proxies=True)
def raw_proxy(t):
    """raw proxy of a local upstream: first fetch, cached fetch, missing paths, negative cache, group with a proxy."""
    if not t.context.proxies:
        raise Skip("proxies to 127.0.0.1 are not allowed")
    up = t.context.upstream()
    up.add("/raw/file.txt", TEXT, "text/plain")
    up.add("/raw/data.gz", BINARY, "application/gzip")
    p = t.prefix
    repo = t.create_repository("raw", "proxy", proxy(p + "raw-proxy", up.base + "/raw/"), step="create proxy")
    hostedrepo = t.create_repository("raw", "hosted", hosted(p + "raw-proxy-hosted"), step="create hosted")
    grp = t.create_repository("raw", "group", group(p + "raw-proxy-group", [hostedrepo, repo]), step="create group")
    base = "/repository/%s/" % repo
    t.step("first fetch", "GET", base + "file.txt", spec=BYTES)
    t.step("cached fetch", "GET", base + "file.txt", spec=BYTES)
    t.step("head", "HEAD", base + "data.gz", spec=NONE)
    t.step("binary", "GET", base + "data.gz", spec=BYTES)
    t.step("range", "GET", base + "data.gz", headers={"Range": "bytes=10-19"}, spec=BYTES)
    t.step("missing", "GET", base + "missing.txt", spec=NONE)
    t.step("missing again", "GET", base + "missing.txt", spec=NONE)
    t.step("put to proxy", "PUT", base + "new.txt", body=TEXT, headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("through group", "GET", "/repository/%s/file.txt" % grp, spec=BYTES)
    t.step("group missing", "GET", "/repository/%s/nothing.txt" % grp, spec=NONE)
    t.step("invalidate cache", "POST", "/service/rest/v1/repositories/%s/invalidate-cache" % repo, spec=NONE)
    t.step("after invalidate", "GET", base + "file.txt", spec=BYTES)
    rest_listing(t, "rest", repo)
    t.fact("upstream fetches of file.txt", min(up.count("/raw/file.txt"), 3) > 0)
    t.json_step("repository", "/service/rest/v1/repositories/raw/proxy/" + repo)
    t.json_step("group", "/service/rest/v1/repositories/raw/group/" + grp)
