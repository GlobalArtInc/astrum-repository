import glob
import gzip
import os
import re

from ..compare import Spec
from ..harness import Skip, scenario
from .common import BYTES, NONE, hosted, rest_listing, settle_search

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.join(HERE, "..", "..", "..", "..", "web", "e2e", "fixtures", "*.rpm")
REPOMD = Spec(body="text", skip_headers=("etag", "content-length"), transform=lambda text: re.sub(
    r"[0-9a-f]{64}", "<sha256>", re.sub(r"<(size|open-size)>\d+</\1>", r"<\1>n</\1>", re.sub(r"(?<![0-9a-f])\d{10}(?![0-9a-f])", "<epoch>", text))))


def rpm_file(t):
    found = os.environ.get("SBS_RPM") or next(iter(sorted(glob.glob(FIXTURES))), None)
    if not found or not os.path.exists(found):
        raise Skip("no test RPM (set SBS_RPM)")
    with open(found, "rb") as handle:
        return os.path.basename(found), handle.read()


@scenario("yum", formats=["yum"])
def yum(t):
    """yum hosted: RPM uploads, generated repodata (repomd.xml, primary), package downloads, deletes."""
    name, data = rpm_file(t)
    t.normalize_rule(r"[0-9a-f]{64}", "<sha256>")
    p = t.prefix
    repo = t.create_repository("yum", "hosted", hosted(p + "yum-hosted", {"yum": {"repodataDepth": 1,
                                                                                 "deployPolicy": "STRICT"}}),
                               step="create hosted")
    base = "/repository/%s/" % repo
    t.step("put rpm", "PUT", base + "el9/" + name, body=data, headers={"Content-Type": "application/x-rpm"}, spec=NONE)
    t.step("put rpm again", "PUT", base + "el9/" + name, body=data, headers={"Content-Type": "application/x-rpm"}, spec=NONE)
    t.step("put rpm too deep", "PUT", base + "el9/x86_64/extra/" + name, body=data,
           headers={"Content-Type": "application/x-rpm"}, spec=NONE)
    t.step("put not rpm", "PUT", base + "el9/broken.rpm", body=b"not an rpm", headers={"Content-Type": "application/x-rpm"},
           spec=NONE)
    t.step("put non-rpm file", "PUT", base + "el9/readme.txt", body=b"text", headers={"Content-Type": "text/plain"}, spec=NONE)
    repomd = None
    for _ in range(30):
        response = t.request("GET", base + "el9/repodata/repomd.xml")
        if response.status == 200 and b"primary" in response.body:
            repomd = response
            break
        import time
        time.sleep(1)
    t.step("repomd", "GET", base + "el9/repodata/repomd.xml", spec=REPOMD)
    t.step("repomd head", "HEAD", base + "el9/repodata/repomd.xml", spec=NONE)
    if repomd is not None:
        location = re.search(rb'<data type="primary">.*?<location href="([^"]+)"', repomd.body, re.S)
        if location:
            primary = t.request("GET", base + "el9/" + location.group(1).decode())
            text = gzip.decompress(primary.body).decode() if primary.status == 200 else ""
            t.fact("primary packages", sorted(re.findall(r"<name>([^<]+)</name>", text)))
            t.fact("primary location", sorted(re.findall(r'<location href="([^"]+)"', text)))
            t.fact("primary content type", primary.header("Content-Type"))
    t.step("repomd root", "GET", base + "repodata/repomd.xml", spec=NONE)
    t.step("rpm", "GET", base + "el9/" + name, spec=BYTES)
    t.step("rpm head", "HEAD", base + "el9/" + name, spec=NONE)
    t.step("missing rpm", "GET", base + "el9/nothing-1.0-1.noarch.rpm", spec=NONE)
    t.step("repomd key", "GET", base + "repodata/repomd.xml.key", spec=NONE)
    rest_listing(t, "rest", repo, derived=lambda path: "/repodata/" in path)
    settle_search(t, repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s&format=yum" % repo)
    t.step("delete rpm", "DELETE", base + "el9/" + name, spec=NONE)
    t.step("rpm after delete", "GET", base + "el9/" + name, spec=NONE)
