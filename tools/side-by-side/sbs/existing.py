import fnmatch
import re
import urllib.parse

from . import harness
from .compare import Spec
from .scenarios import served_formats
from .scenarios.common import quote, strip_keys
from .scenarios.helm import entries as helm_entries
from .scenarios.maven import metadata_asset, metadata_facts
from .scenarios.pypi import links as pypi_links

LISTED = Spec(body="json", volatile=("downloadCount", "lastDownloaded"))
DOWNLOAD = Spec(body="bytes", skip_headers=("content-length", "content-disposition", "accept-ranges"))
TEXT = Spec(body="text", skip_headers=("etag", "content-length", "last-modified"))
DOCKER_ACCEPT = ", ".join([
    "application/vnd.oci.image.index.v1+json", "application/vnd.oci.image.manifest.v1+json",
    "application/vnd.docker.distribution.manifest.list.v2+json",
    "application/vnd.docker.distribution.manifest.v2+json"])
DERIVED = {
    "maven2": metadata_asset,
    "pypi": lambda path: "/simple/" in path or path.endswith(".metadata"),
    "rubygems": lambda path: not path.endswith(".gem") and "/quick/" not in path,
    "helm": lambda path: path.endswith("index.yaml"),
    "yum": lambda path: "/repodata/" in path,
    "npm": lambda path: "/-/" not in path,
}
PATH_FORMATS = {"maven2": "maven"}


SAFE = "/@:+~!$&'()*,;="
CAPABILITY_TYPES = ("baseurl", "rapture.settings", "rapture.branding")


def content_url(name, path):
    return "/repository/%s%s" % (quote(name), urllib.parse.quote(path, safe=SAFE))


def download_path(fmt, name, path, images):
    if fmt == "docker":
        blob = re.fullmatch(r"/v2/-/blobs/(sha256:[0-9a-f]+|sha512:[0-9a-f]+)", path)
        if blob and images:
            return "/v2/%s/%s/blobs/%s" % (name, images[0], blob.group(1)), {}
        manifest = re.fullmatch(r"/v2/(.+)/manifests/([^/]+)", path)
        if manifest:
            return "/v2/%s/%s/manifests/%s" % (name, manifest.group(1), manifest.group(2)), {"Accept": DOCKER_ACCEPT}
    return content_url(name, path), {}


def listing(server, kind, repository):
    return list(server.paginate("/service/rest/v1/%s" % kind, repository=repository))


def by_key(items, key):
    return {str(item.get(key)): item for item in items if isinstance(item, dict)}


def keep_listed(derived):
    def transform(value):
        if not isinstance(value, list):
            return value
        out = []
        for item in value:
            if not isinstance(item, dict):
                continue
            if derived and derived(str(item.get("path", ""))):
                continue
            if isinstance(item.get("assets"), list):
                item = dict(item, assets=sorted((a for a in item["assets"] if not (derived and derived(
                    str(a.get("path", ""))))), key=lambda a: str(a.get("path"))))
            out.append(item)
        return out
    return transform


def served(t):
    aside = t.context.kept_aside

    def transform(value):
        if not isinstance(value, list):
            return value
        return [item for item in value
                if not (isinstance(item, dict) and (item.get("name") in aside or item.get("repository") in aside))]
    return transform


def rest(t):
    """Configuration and security of the whole installation, as both servers list it."""
    counts = ("availableSpaceInBytes",)
    t.json_step("repositories", "/service/rest/v1/repositories", volatile=("size",), transform=served(t))
    t.json_step("repositorySettings", "/service/rest/v1/repositorySettings", transform=served(t))
    t.json_step("blobstores", "/service/rest/v1/blobstores", volatile=counts)
    t.json_step("users", "/service/rest/v1/security/users")
    t.json_step("roles", "/service/rest/v1/security/roles")
    t.json_step("privileges", "/service/rest/v1/security/privileges", transform=served(t))
    t.json_step("content selectors", "/service/rest/v1/security/content-selectors")
    t.json_step("realms active", "/service/rest/v1/security/realms/active", spec=Spec(body="json", unordered=False))
    t.json_step("anonymous", "/service/rest/v1/security/anonymous")
    t.json_step("user sources", "/service/rest/v1/security/user-sources")
    t.json_step("ssrf", "/service/rest/v1/security/ssrf-protection")
    t.json_step("routing rules", "/service/rest/v1/routing-rules")
    t.json_step("http settings", "/service/rest/v1/http")
    t.json_step("email settings", "/service/rest/v1/email")
    t.json_step("capabilities", "/service/rest/v1/capabilities", transform=lambda value: [
        strip_keys("id")(i) for i in value if isinstance(i, dict) and i.get("type") in CAPABILITY_TYPES]
        if isinstance(value, list) else value)
    found = t.server.get_json("/service/rest/v1/capabilities")
    t.fact("capability types", sorted({i.get("type") for i in found if isinstance(i, dict)}))
    tasks = t.server.get_json("/service/rest/v1/tasks").get("items") or []
    t.fact("tasks", sorted("%s|%s|%s|%s" % (i.get("name"), i.get("type"), i.get("enabled"),
                                            i.get("cronExpression") or i.get("schedule")) for i in tasks))
    t.json_step("status", "/service/rest/v1/status", body="none")
    t.json_step("status writable", "/service/rest/v1/status/writable", body="none")


def repository_scenario(name, fmt, kind, args):
    def run(t):
        repository(t, name, fmt, kind, args)
    run.__doc__ = "%s %s repository %s" % (fmt, kind, name)
    return run


def repository(t, name, fmt, kind, args):
    t.json_step("config", "/service/rest/v1/repositories/%s/%s/%s" % (PATH_FORMATS.get(fmt, fmt), kind, quote(name)))
    derived = DERIVED.get(fmt)
    components = listing(t.server, "components", name)
    assets = listing(t.server, "assets", name)
    t.fact("components", [{k: c.get(k) for k in ("group", "name", "version")} for c in components],
           spec=Spec(body="json"))
    t.fact("assets", [a for a in assets if not (derived and derived(str(a.get("path", ""))))],
           spec=Spec(body="json", volatile=("downloadCount", "lastDownloaded"), transform=keep_listed(None)))
    if derived:
        t.fact("derived assets", sorted(a.get("path") for a in assets if derived(str(a.get("path", "")))))
    paths = sorted({str(a.get("path", "")) for a in assets if not (derived and derived(str(a.get("path", ""))))})
    images = sorted({str(c.get("name")) for c in components})
    if kind == "group":
        members = ((t.server.get_json("/service/rest/v1/repositories/%s/%s/%s" % (
            PATH_FORMATS.get(fmt, fmt), kind, quote(name))).get("group") or {}).get("memberNames") or [])
        paths = set()
        for member in members:
            for asset in listing(t.server, "assets", member):
                if not (derived and derived(str(asset.get("path", "")))):
                    paths.add(str(asset.get("path", "")))
            if fmt == "docker":
                images = sorted(set(images) | {str(c.get("name")) for c in listing(t.server, "components", member)})
        paths = sorted(paths)
    if args.max_assets:
        paths = paths[:args.max_assets]
    for path in paths:
        target, headers = download_path(fmt, name, path, images)
        t.step("download %s" % path, "GET", target, headers=headers, digest=True, spec=DOWNLOAD)
    if paths:
        target, headers = download_path(fmt, name, paths[0], images)
        t.step("anonymous download %s" % paths[0], "GET", target, headers=headers, auth=None, spec=Spec(body="none"))
        t.step("head %s" % paths[0], "HEAD", target, headers=headers, spec=Spec(body="none"))
    probe = PROBES.get(fmt)
    if probe:
        probe(t, name, kind, components)


def maven_probe(t, name, kind, components):
    seen = set()
    spec = Spec(body="text", skip_headers=("etag", "content-length", "last-modified"), transform=metadata_facts)
    for component in components[:50]:
        ga = (component.get("group"), component.get("name"))
        base = "/repository/%s/%s/%s/" % (name, str(ga[0]).replace(".", "/"), ga[1])
        if ga not in seen:
            seen.add(ga)
            t.step("metadata %s:%s" % ga, "GET", base + "maven-metadata.xml", spec=spec)
        version = str(component.get("version") or "")
        if "-SNAPSHOT" in version or re.search(r"-\d{8}\.\d{6}-\d+$", version):
            base_version = re.sub(r"-\d{8}\.\d{6}-\d+$", "-SNAPSHOT", version)
            key = ga + (base_version,)
            if key not in seen:
                seen.add(key)
                t.step("metadata %s:%s:%s" % key, "GET", base + base_version + "/maven-metadata.xml", spec=spec)


def npm_probe(t, name, kind, components):
    names = sorted({("@%s/%s" % (c.get("group"), c.get("name"))) if c.get("group") else c.get("name")
                    for c in components})
    for package in names[:50]:
        escaped = package.replace("/", "%2f")
        t.step("document %s" % package, "GET", "/repository/%s/%s" % (name, escaped),
               spec=Spec(body="json", volatile=("_rev",), skip_headers=("etag", "content-length", "last-modified")))


def pypi_probe(t, name, kind, components):
    t.step("simple root", "GET", "/repository/%s/simple/" % name, spec=Spec(
        body="text", skip_headers=("etag", "content-length", "last-modified"), transform=pypi_links))
    for project in sorted({c.get("name") for c in components})[:50]:
        t.step("simple %s" % project, "GET", "/repository/%s/simple/%s/" % (name, project), headers={"Accept": "text/html"},
               spec=Spec(body="text", skip_headers=("etag", "content-length", "last-modified"), transform=pypi_links))


def docker_probe(t, name, kind, components):
    images = sorted({c.get("name") for c in components})
    for image in images[:30]:
        t.step("tags %s" % image, "GET", "/v2/%s/%s/tags/list" % (name, image), spec=Spec(body="json"))
    for component in sorted(components, key=lambda c: (c.get("name"), c.get("version")))[:60]:
        path = "/v2/%s/%s/manifests/%s" % (name, component.get("name"), component.get("version"))
        t.step("manifest %s:%s" % (component.get("name"), component.get("version")), "HEAD", path,
               headers={"Accept": DOCKER_ACCEPT}, spec=Spec(body="none"))


def helm_probe(t, name, kind, components):
    t.step("index", "GET", "/repository/%s/index.yaml" % name, spec=Spec(
        body="text", skip_headers=("etag", "content-length", "last-modified"), transform=helm_entries))


def nuget_probe(t, name, kind, components):
    t.step("service index", "GET", "/repository/%s/index.json" % name, spec=Spec(body="json"))
    for package in sorted({c.get("name") for c in components})[:30]:
        t.step("versions %s" % package, "GET", "/repository/%s/v3/content/%s/index.json" % (name, str(package).lower()),
               spec=Spec(body="json"))


def rubygems_probe(t, name, kind, components):
    generated = Spec(body="text", skip_headers=("etag", "content-length", "last-modified"))
    t.step("versions", "GET", "/repository/%s/versions" % name, spec=generated)
    t.step("specs", "GET", "/repository/%s/specs.4.8.gz" % name, spec=Spec(body="gunzip", skip_headers=("etag",)))
    for gem in sorted({c.get("name") for c in components})[:30]:
        t.step("info %s" % gem, "GET", "/repository/%s/info/%s" % (name, gem), spec=generated)


def yum_probe(t, name, kind, components):
    t.step("repomd", "HEAD", "/repository/%s/repodata/repomd.xml" % name, spec=Spec(body="none", skip_headers=(
        "etag", "content-length", "last-modified")))


def terraform_probe(t, name, kind, components):
    for component in components[:30]:
        if "/" in str(component.get("name")):
            module, system = str(component.get("name")).split("/", 1)
            t.step("module versions %s/%s" % (component.get("group"), component.get("name")), "GET",
                   "/repository/%s/v1/modules/%s/%s/%s/versions" % (name, component.get("group"), module, system),
                   spec=Spec(body="json"))


def swift_probe(t, name, kind, components):
    for component in components[:30]:
        t.step("releases %s.%s" % (component.get("group"), component.get("name")), "GET",
               "/repository/%s/%s/%s" % (name, component.get("group"), component.get("name")),
               headers={"Accept": "application/vnd.swift.registry.v1+json"}, spec=Spec(body="json"))


PROBES = {"maven2": maven_probe, "npm": npm_probe, "pypi": pypi_probe, "docker": docker_probe, "helm": helm_probe,
          "nuget": nuget_probe, "rubygems": rubygems_probe, "yum": yum_probe, "terraform": terraform_probe,
          "swift": swift_probe}


def run(context, allow):
    args = context.args
    reference = {r["name"]: r for r in context.servers["reference"].get_json("/service/rest/v1/repositories")}
    candidate = {r["name"]: r for r in context.servers["candidate"].get_json("/service/rest/v1/repositories")}
    types = {t.strip() for t in args.types.split(",") if t.strip()}
    served_formats(context)
    context.kept_aside = {name for name, repo in reference.items()
                          if repo.get("format") not in context.formats["candidate"] and name not in candidate}
    items = []
    if not args.no_rest:
        items.append(harness.Scenario("rest", rest, (), (), rest.__doc__, False))
    missing = sorted(set(reference) - set(candidate))
    extra = sorted(set(candidate) - set(reference))
    for name in sorted(reference):
        repo = reference[name]
        if args.repository and not any(fnmatch.fnmatchcase(name, p) for p in args.repository):
            continue
        if repo.get("type") not in types or name not in candidate:
            continue
        items.append(harness.Scenario("repo:" + name, repository_scenario(name, repo["format"], repo["type"], args),
                                      (repo["format"],), (), "", False))
    if missing:
        context.log.info("   repositories only on the reference: %s" % ", ".join(missing))
    if extra:
        context.log.info("   repositories only on the candidate: %s" % ", ".join(extra))
    outcomes = harness.run_scenarios(context, items, allow, "existing")
    presence = harness.Scenario("repositories", lambda t: None, (), (), "", False)
    trials = [harness.Trial(context, presence, side) for side in harness.SIDES]
    trials[0].fact("names", sorted(set(reference) - context.kept_aside))
    trials[1].fact("names", sorted(candidate))
    trials[0].fact("kept aside", sorted("%s (%s)" % (name, reference[name].get("format")) for name in context.kept_aside))
    trials[1].fact("kept aside", [])
    outcomes.append((presence, trials, harness.compare_trials(presence, trials, allow, "existing")))
    return outcomes
