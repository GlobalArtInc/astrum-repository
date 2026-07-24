import io
import zipfile

from ..compare import Spec
from ..harness import scenario
from .common import BYTES, NONE, hosted, multipart, rest_listing, settle_search

ID = "Sbs.Lib"


def nupkg(package_id, version):
    out = io.BytesIO()
    nuspec = ("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<package xmlns=\"http://schemas.microsoft.com/packaging/2013/05/"
              "nuspec.xsd\"><metadata><id>%s</id><version>%s</version><authors>side-by-side</authors>"
              "<description>side-by-side test package</description><tags>sbs test</tags></metadata></package>\n"
              % (package_id, version))
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
        stamp = (2020, 1, 1, 0, 0, 0)
        archive.writestr(zipfile.ZipInfo("%s.nuspec" % package_id, stamp), nuspec)
        archive.writestr(zipfile.ZipInfo("lib/netstandard2.0/_._", stamp), "")
        archive.writestr(zipfile.ZipInfo("[Content_Types].xml", stamp),
                         "<?xml version=\"1.0\" encoding=\"utf-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/"
                         "package/2006/content-types\"><Default Extension=\"nuspec\" ContentType=\"application/octet\"/>"
                         "</Types>")
    return out.getvalue()


def push(t, step, repo, data, spec=NONE):
    body, ctype = multipart([], [("package", "package.nupkg", data, "application/octet-stream")])
    return t.step(step, "PUT", "/repository/%s/" % repo, body=body, headers={"Content-Type": ctype}, spec=spec)


def resources(t, repo):
    response = t.request("GET", "/repository/%s/index.json" % repo)
    out = {}
    if response.status == 200:
        for item in response.json().get("resources", []):
            out.setdefault(item.get("@type", "").split("/")[0], item.get("@id"))
    return out


@scenario("nuget", formats=["nuget"])
def nuget(t):
    """nuget hosted: multipart push as nuget.exe sends it, V3 service index, flat container, registration, search, V2 feed."""
    p = t.prefix
    repo = t.create_repository("nuget", "hosted", hosted(p + "nuget-hosted"),
                               step="create hosted")
    base = "/repository/%s/" % repo
    for version in ("1.0.0", "1.1.0-beta"):
        push(t, "push " + version, repo, nupkg(ID, version))
    push(t, "push again", repo, nupkg(ID, "1.0.0"))
    push(t, "push not a package", repo, b"nope")
    t.step("service index", "GET", base + "index.json", spec=Spec(body="json"))
    found = resources(t, repo)
    t.fact("service resources", sorted(found))
    lower = ID.lower()
    if found.get("PackageBaseAddress"):
        flat = found["PackageBaseAddress"].rstrip("/") + "/"
        t.step("flat versions", "GET", flat + lower + "/index.json", spec=Spec(body="json"))
        t.step("flat nupkg", "GET", flat + "%s/1.0.0/%s.1.0.0.nupkg" % (lower, lower), spec=BYTES)
        t.step("flat nuspec", "GET", flat + "%s/1.0.0/%s.nuspec" % (lower, lower), spec=NONE)
        t.step("flat missing", "GET", flat + "sbs.nothing/index.json", spec=NONE)
    if found.get("RegistrationsBaseUrl"):
        registration = found["RegistrationsBaseUrl"].rstrip("/") + "/"
        t.step("registration", "GET", registration + lower + "/index.json",
               spec=Spec(body="json", volatile=("published", "commitTimeStamp", "commitId")))
        t.step("registration leaf", "GET", registration + lower + "/1.0.0.json",
               spec=Spec(body="json", volatile=("published", "commitTimeStamp", "commitId")))
    if found.get("SearchQueryService"):
        t.step("search query", "GET", found["SearchQueryService"] + "?q=sbs&prerelease=true",
               spec=Spec(body="json", volatile=("totalDownloads", "downloads")))
    t.step("v2 root", "GET", base, spec=NONE)
    t.step("v2 metadata", "GET", base + "$metadata", spec=NONE)
    t.step("v2 find by id", "GET", base + "FindPackagesById()?id='%s'" % ID, spec=NONE)
    t.step("v2 package entry", "GET", base + "Packages(Id='%s',Version='1.0.0')" % ID, spec=NONE)
    t.step("v2 download", "GET", base + "%s/1.0.0" % ID, spec=BYTES)
    t.step("v2 search", "GET", base + "Search()?searchTerm='sbs'&includePrerelease=true", spec=NONE)
    t.step("delete version", "DELETE", base + "%s/1.1.0-beta" % ID, spec=NONE)
    rest_listing(t, "rest", repo)
    settle_search(t, repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s&name=%s" % (repo, ID))
