import gzip
import io
import tarfile
import time

from ..compare import Spec
from ..harness import scenario
from .common import BYTES, NONE, hosted, rest_listing, settle_search


def entries(text):
    found, current, in_urls = [], None, False
    for line in text.splitlines():
        stripped = line.strip()
        if line.startswith("  - ") or line.startswith("- "):
            current = {"urls": []}
            found.append(current)
            stripped = stripped[2:]
            in_urls = False
        if current is None:
            continue
        if stripped.startswith("- ") and in_urls:
            current["urls"].append(stripped[2:].strip("'\""))
            continue
        key, _, value = stripped.partition(":")
        in_urls = key == "urls"
        if key in ("name", "version"):
            current[key] = value.strip().strip("'\"")
    return "\n".join(sorted("%s %s %s" % (e.get("name"), e.get("version"), ",".join(e["urls"])) for e in found))


INDEX = Spec(body="text", skip_headers=("etag", "content-length", "last-modified"), transform=entries)


def settle_index(t, base, ready, timeout=60):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        response = t.request("GET", base + "index.yaml")
        if response.status == 200 and ready(response.text()):
            return True
        time.sleep(1)
    return False


def chart(name, version, app="1.0"):
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w") as archive:
        for path, content in (("Chart.yaml", "apiVersion: v2\nname: %s\nversion: %s\nappVersion: \"%s\"\n"
                                             "description: side-by-side chart\ntype: application\n" % (name, version, app)),
                              ("values.yaml", "replicas: 1\n"),
                              ("templates/configmap.yaml", "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: x\n")):
            data = content.encode()
            info = tarfile.TarInfo("%s/%s" % (name, path))
            info.size = len(data)
            info.mtime = 1577836800
            archive.addfile(info, io.BytesIO(data))
    return gzip.compress(out.getvalue(), mtime=0)


@scenario("helm", formats=["helm"])
def helm(t):
    """helm hosted: chart uploads with PUT, index.yaml, chart and provenance downloads, deletes."""
    p = t.prefix
    repo = t.create_repository("helm", "hosted", hosted(p + "helm-hosted"), step="create hosted")
    base = "/repository/%s/" % repo
    for version in ("0.1.0", "0.2.0"):
        t.step("put chart " + version, "PUT", base + "sbs-chart-%s.tgz" % version, body=chart("sbs-chart", version),
               headers={"Content-Type": "application/gzip"}, spec=NONE)
    t.step("put chart again", "PUT", base + "sbs-chart-0.1.0.tgz", body=chart("sbs-chart", "0.1.0"),
           headers={"Content-Type": "application/gzip"}, spec=NONE)
    t.step("put provenance", "PUT", base + "sbs-chart-0.1.0.tgz.prov", body=b"-----BEGIN PGP SIGNED MESSAGE-----\n\n",
           headers={"Content-Type": "application/octet-stream"}, spec=NONE)
    t.step("put not a chart", "PUT", base + "broken-1.0.0.tgz", body=b"not a chart",
           headers={"Content-Type": "application/gzip"}, spec=NONE)
    t.step("put in folder", "PUT", base + "charts/sbs-chart-0.3.0.tgz", body=chart("sbs-chart", "0.3.0"),
           headers={"Content-Type": "application/gzip"}, spec=NONE)
    t.step("post chart", "POST", base + "api/charts", body=chart("sbs-posted", "1.0.0"),
           headers={"Content-Type": "application/gzip"}, spec=NONE)
    settle_index(t, base, lambda text: "version: 0.2.0" in text)
    t.step("index", "GET", base + "index.yaml", spec=INDEX)
    t.step("index head", "HEAD", base + "index.yaml", spec=Spec(body="none", skip_headers=("etag", "content-length")))
    t.step("chart", "GET", base + "sbs-chart-0.1.0.tgz", spec=BYTES)
    t.step("provenance", "GET", base + "sbs-chart-0.1.0.tgz.prov", spec=BYTES)
    t.step("missing chart", "GET", base + "sbs-chart-9.9.9.tgz", spec=NONE)
    t.step("delete chart", "DELETE", base + "sbs-chart-0.2.0.tgz", spec=NONE)
    settle_index(t, base, lambda text: "version: 0.2.0" not in text)
    t.step("index after delete", "GET", base + "index.yaml", spec=INDEX)
    rest_listing(t, "rest", repo, derived=lambda path: path.endswith("index.yaml"))
    settle_search(t, repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s&name=sbs-chart" % repo)
