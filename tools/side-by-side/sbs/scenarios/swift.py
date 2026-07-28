import io
import json
import zipfile

from ..compare import Spec
from ..harness import scenario
from .common import NONE, group, hosted, rest_listing, settle_search

JSON = "application/vnd.swift.registry.v1+json"
ZIP = "application/vnd.swift.registry.v1+zip"
SWIFT = "application/vnd.swift.registry.v1+swift"
BOUNDARY = "SwiftPMBoundarySideBySide0123"
HEADERS = ("content-version", "link", "content-disposition", "digest", "location")


def source(version):
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as archive:
        for path, content in (("LinkedList/Package.swift", "// swift-tools-version:5.7\nimport PackageDescription\n"),
                              ("LinkedList/Package@swift-5.9.swift", "// swift-tools-version: 5.9\n"),
                              ("LinkedList/Sources/LinkedList/List.swift", 'let version = "%s"\n' % version)):
            archive.writestr(zipfile.ZipInfo(path, (2020, 1, 1, 0, 0, 0)), content)
    return out.getvalue()


def form(archive, metadata):
    body = ("--%s\r\nContent-Disposition: form-data; name=\"source-archive\"\r\nContent-Type: application/zip\r\n"
            "Content-Transfer-Encoding: binary\r\n\r\n" % BOUNDARY).encode() + archive
    body += ("\r\n--%s\r\nContent-Disposition: form-data; name=\"metadata\"\r\nContent-Type: application/json\r\n"
             "Content-Transfer-Encoding: quoted-printable\r\n\r\n%s\r\n--%s--\r\n" % (BOUNDARY, json.dumps(metadata), BOUNDARY)
             ).encode()
    return body


def publish(t, step, repo, version, archive):
    metadata = {"description": "One linked list", "repositoryURLs": ["https://github.com/sbs/LinkedList.git"],
                "author": {"name": "Side"}}
    return t.step(step, "PUT", "/repository/%s/sbs/LinkedList/%s" % (repo, version), body=form(archive, metadata),
                  headers={"Accept": JSON, "Content-Type": "multipart/form-data;boundary=\"%s\"" % BOUNDARY},
                  spec=Spec(body="none", extra_headers=HEADERS))


@scenario("swift", formats=["swift"])
def swift(t):
    """swift hosted and group (SE-0292): publish, releases, release metadata, manifests, archives, identifiers, login."""
    p = t.prefix
    repo = t.create_repository("swift", "hosted", hosted(p + "swift-hosted", write_policy="ALLOW_ONCE"),
                               step="create hosted")
    grp = t.create_repository("swift", "group", group(p + "swift-group", [repo], writable=repo), step="create group")
    publish(t, "publish through group", grp, "1.1.1", source("1.1.1"))
    publish(t, "publish older", repo, "1.0.0", source("1.0.0"))
    publish(t, "publish again", repo, "1.1.1", source("1.1.1"))
    publish(t, "publish broken", repo, "2.0.0", b"not a zip")
    spec = Spec(body="json", extra_headers=HEADERS)
    for label, target in (("hosted", repo), ("group", grp)):
        base = "/repository/%s/sbs/LinkedList" % target
        t.step(label + " releases", "GET", base, headers={"Accept": JSON}, spec=spec)
        t.step(label + " releases case", "GET", "/repository/%s/SBS/linkedlist" % target, headers={"Accept": JSON},
               spec=spec)
        t.step(label + " release", "GET", base + "/1.1.1", headers={"Accept": JSON},
               spec=Spec(body="json", extra_headers=HEADERS, volatile=("publishedAt",)))
        t.step(label + " manifest", "GET", base + "/1.1.1/Package.swift", headers={"Accept": SWIFT},
               spec=Spec(body="bytes", extra_headers=HEADERS))
        t.step(label + " manifest version", "GET", base + "/1.1.1/Package.swift?swift-version=5.9",
               headers={"Accept": SWIFT}, spec=Spec(body="bytes", extra_headers=HEADERS))
        t.step(label + " manifest missing version", "GET", base + "/1.1.1/Package.swift?swift-version=6",
               headers={"Accept": SWIFT}, spec=Spec(body="none", extra_headers=HEADERS))
        t.step(label + " archive", "GET", base + "/1.1.1.zip", headers={"Accept": ZIP},
               spec=Spec(body="bytes", extra_headers=HEADERS))
        t.step(label + " identifiers", "GET", "/repository/%s/identifiers?url=https://github.com/sbs/LinkedList" % target,
               headers={"Accept": JSON}, spec=spec)
        t.step(label + " identifiers unknown", "GET", "/repository/%s/identifiers?url=https://x.org/y" % target,
               headers={"Accept": JSON}, spec=Spec(body="none", extra_headers=HEADERS))
        t.step(label + " release missing", "GET", base + "/9.9.9", headers={"Accept": JSON},
               spec=Spec(body="none", extra_headers=HEADERS))
        t.step(label + " no accept", "GET", base, spec=Spec(body="none", extra_headers=HEADERS))
    t.step("login", "POST", "/repository/%s/login" % repo, headers={"Accept": JSON}, spec=NONE)
    t.step("login anonymous", "POST", "/repository/%s/login" % repo, headers={"Accept": JSON}, auth=None, spec=NONE)
    rest_listing(t, "rest", repo)
    settle_search(t, repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s" % repo)
