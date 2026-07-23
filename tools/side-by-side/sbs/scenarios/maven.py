import io
import os
import re
import zipfile
from xml.etree import ElementTree

from ..compare import Spec
from ..harness import Skip, scenario
from .common import BYTES, NONE, chain, components, group, hosted, proxy, rest_listing, settle_search, without_assets

GROUP = "com.example.sbs"
DEPLOY = "org.apache.maven.plugins:maven-deploy-plugin:3.1.3"
DEPENDENCY = "org.apache.maven.plugins:maven-dependency-plugin:3.7.1"
TAIL = os.path.expanduser("~/.m2/repository")
def metadata_facts(text):
    text = re.sub(r"<(stamp|snapshot-ts|time|uuid|millis|base|host)>", r"__\1__", text)
    try:
        root = ElementTree.fromstring(text.encode("utf-8"))
    except ElementTree.ParseError:
        return text
    strip = lambda tag: tag.split("}", 1)[-1]
    found = {}
    for element in root.iter():
        name = strip(element.tag)
        if name in ("groupId", "artifactId", "release", "timestamp", "buildNumber", "localCopy"):
            found[name] = (element.text or "").strip()
        elif name == "version" and strip(root.tag) == "metadata" and element in list(root):
            found["version"] = (element.text or "").strip()
    versions = sorted((v.text or "").strip() for v in root.iter() if strip(v.tag) == "version" and v not in list(root))
    snapshots = sorted("%s:%s:%s" % (sv.findtext("{*}classifier") or sv.findtext("classifier") or "",
                                     sv.findtext("{*}extension") or sv.findtext("extension") or "",
                                     sv.findtext("{*}value") or sv.findtext("value") or "")
                       for sv in root.iter() if strip(sv.tag) == "snapshotVersion")
    found.pop("version", None)
    lines = ["%s=%s" % item for item in sorted(found.items())] + ["versions=" + ",".join(versions)]
    lines += ["snapshotVersion=" + item for item in snapshots]
    return "\n".join(lines)


METADATA = Spec(body="text", skip_headers=("etag", "content-length"), transform=metadata_facts)


def jar(name, marker):
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as archive:
        info = zipfile.ZipInfo("META-INF/MANIFEST.MF", (2020, 1, 1, 0, 0, 0))
        archive.writestr(info, "Manifest-Version: 1.0\r\nCreated-By: side-by-side\r\n\r\n")
        info = zipfile.ZipInfo("%s/marker.txt" % name, (2020, 1, 1, 0, 0, 0))
        archive.writestr(info, marker)
    return out.getvalue()


def pom(artifact, version, dependency=None):
    deps = ""
    if dependency:
        deps = ("<dependencies><dependency><groupId>%s</groupId><artifactId>%s</artifactId><version>%s</version>"
                "</dependency></dependencies>" % dependency)
    return ("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project xmlns=\"http://maven.apache.org/POM/4.0.0\">"
            "<modelVersion>4.0.0</modelVersion><groupId>%s</groupId><artifactId>%s</artifactId><version>%s</version>"
            "<packaging>jar</packaging>%s</project>\n" % (GROUP, artifact, version, deps)).encode("utf-8")


def settings(t, mirror, server_id="sbs"):
    user, password = t.client_user()
    path = t.path("settings.xml")
    with open(path, "w") as handle:
        handle.write("""<settings><servers><server><id>%s</id><username>%s</username><password>%s</password></server>
</servers><mirrors><mirror><id>%s</id><mirrorOf>*</mirrorOf><url>%s</url></mirror></mirrors>
<profiles><profile><id>sbs</id><repositories><repository><id>central</id><url>%s</url>
<releases><enabled>true</enabled></releases><snapshots><enabled>true</enabled><updatePolicy>always</updatePolicy></snapshots>
</repository></repositories></profile></profiles><activeProfiles><activeProfile>sbs</activeProfile></activeProfiles>
</settings>""" % (server_id, user, password, server_id, mirror, mirror))
    return path


def mvn(t, step, settings_path, local, *args):
    argv = ["mvn", "-B", "-ntp", "-s", settings_path, "-Dmaven.repo.local=" + local, "-Dmaven.repo.local.tail=" + TAIL,
            "-Daether.connector.http.preemptiveAuth=false"] + list(args)
    return t.run(step, argv, env={"MAVEN_OPTS": "-Xmx384m", "JAVA_TOOL_OPTIONS": ""}, timeout=300)


def deploy(t, step, settings_path, local, url, artifact, version, file, pom_file=None, classifier=None, packaging="jar"):
    args = ["%s:deploy-file" % DEPLOY, "-Durl=" + url, "-DrepositoryId=sbs", "-DgroupId=" + GROUP,
            "-DartifactId=" + artifact, "-Dversion=" + version, "-Dpackaging=" + packaging, "-Dfile=" + file]
    if pom_file:
        args.append("-DpomFile=" + pom_file)
    else:
        args.append("-DgeneratePom=%s" % ("false" if classifier else "true"))
    if classifier:
        args.append("-Dclassifier=" + classifier)
    return mvn(t, step, settings_path, local, *args)


def write(t, name, data):
    path = t.path(name)
    with open(path, "wb") as handle:
        handle.write(data)
    return path


def sorted_assets(value):
    if isinstance(value, dict) and isinstance(value.get("items"), list):
        items = [dict(i, assets=sorted(i.get("assets") or [], key=lambda a: str(a.get("path")))) if isinstance(i, dict)
                 else i for i in value["items"]]
        return dict(value, items=items)
    return value


def metadata_asset(path):
    name = path.rsplit("/", 1)[-1]
    return "maven-metadata.xml" in name or "archetype-catalog.xml" in name or name.endswith(
        (".sha1", ".md5", ".sha256", ".sha512"))


def exchange_spec(method, path):
    if path.split("?")[0].endswith("maven-metadata.xml"):
        return METADATA
    if "maven-metadata.xml." in path:
        return NONE
    return None


def keep_maven(method, path):
    return "/com/example/sbs/" in path or path.endswith("/archetype-catalog.xml")


@scenario("maven", formats=["maven2"], clients=["mvn"])
def maven(t):
    """maven2 hosted releases and snapshots with mvn deploy and dependency:get through a group; metadata, checksums."""
    p = t.prefix
    layout = {"maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT", "contentDisposition": "INLINE"}}
    releases = t.create_repository("maven2", "hosted", hosted(p + "maven-releases", layout, write_policy="ALLOW_ONCE"),
                                   step="create releases")
    snap_layout = {"maven": {"versionPolicy": "SNAPSHOT", "layoutPolicy": "STRICT", "contentDisposition": "INLINE"}}
    snapshots = t.create_repository("maven2", "hosted", hosted(p + "maven-snapshots", snap_layout),
                                    step="create snapshots")
    mixed_layout = {"maven": {"versionPolicy": "MIXED", "layoutPolicy": "PERMISSIVE", "contentDisposition": "ATTACHMENT"}}
    mixed = t.create_repository("maven2", "hosted", hosted(p + "maven-mixed", mixed_layout), step="create mixed")
    grp = t.create_repository("maven2", "group", group(p + "maven-group", [releases, snapshots, mixed],
                                                         {"maven": {"versionPolicy": "MIXED", "layoutPolicy": "STRICT"}}),
                              step="create group")
    t.need("mvn")
    lib_jar = write(t, "lib.jar", jar("lib", "release 1.0.0\n"))
    lib_pom = write(t, "lib.pom", pom("lib", "1.0.0"))
    app_pom = write(t, "app.pom", pom("app", "2.0.0", (GROUP, "lib", "1.0.0")))
    app_jar = write(t, "app.jar", jar("app", "app 2.0.0\n"))
    sources = write(t, "lib-sources.jar", jar("src", "sources\n"))
    snap_jar = write(t, "snap.jar", jar("lib", "snapshot\n"))
    deploy_settings = settings(t, t.repo_url(grp))
    local = t.path("m2-deploy", "x")[:-2]
    with t.recording("mvn deploy", spec=exchange_spec, keep=keep_maven) as base:
        rel = "%s/repository/%s/" % (base, releases)
        snap = "%s/repository/%s/" % (base, snapshots)
        deploy(t, "deploy release", deploy_settings, local, rel, "lib", "1.0.0", lib_jar, lib_pom)
        deploy(t, "deploy sources", deploy_settings, local, rel, "lib", "1.0.0", sources, None, "sources")
        deploy(t, "deploy app", deploy_settings, local, rel, "app", "2.0.0", app_jar, app_pom)
        deploy(t, "deploy release again", deploy_settings, local, rel, "lib", "1.0.0", lib_jar, lib_pom)
        deploy(t, "deploy snapshot 1", deploy_settings, local, snap, "lib", "1.1.0-SNAPSHOT", snap_jar)
        deploy(t, "deploy snapshot 2", deploy_settings, local, snap, "lib", "1.1.0-SNAPSHOT", snap_jar)
        deploy(t, "deploy snapshot to releases", deploy_settings, local, rel, "lib", "1.2.0-SNAPSHOT", snap_jar)
        deploy(t, "deploy release to snapshots", deploy_settings, local, snap, "lib", "1.3.0", lib_jar)

    base = "/repository/%s/com/example/sbs/" % releases
    t.step("ga metadata", "GET", base + "lib/maven-metadata.xml", spec=METADATA)
    t.step("ga metadata sha1", "GET", base + "lib/maven-metadata.xml.sha1", spec=NONE)
    t.step("jar", "GET", base + "lib/1.0.0/lib-1.0.0.jar", spec=BYTES)
    t.step("jar head", "HEAD", base + "lib/1.0.0/lib-1.0.0.jar", spec=NONE)
    for ext in ("sha1", "md5", "sha256", "sha512"):
        t.step("jar " + ext, "GET", base + "lib/1.0.0/lib-1.0.0.jar." + ext, spec=BYTES)
    t.step("pom", "GET", base + "lib/1.0.0/lib-1.0.0.pom", spec=BYTES)
    t.step("sources jar", "GET", base + "lib/1.0.0/lib-1.0.0-sources.jar", spec=BYTES)
    t.step("missing version", "GET", base + "lib/9.9.9/lib-9.9.9.jar", spec=NONE)
    t.step("missing checksum", "GET", base + "lib/9.9.9/lib-9.9.9.jar.sha1", spec=NONE)
    t.step("folder", "GET", base + "lib/", spec=NONE)
    t.step("gav metadata in release", "GET", base + "lib/1.0.0/maven-metadata.xml", spec=NONE)
    sbase = "/repository/%s/com/example/sbs/" % snapshots
    t.step("snapshot ga metadata", "GET", sbase + "lib/maven-metadata.xml", spec=METADATA)
    t.step("snapshot gav metadata", "GET", sbase + "lib/1.1.0-SNAPSHOT/maven-metadata.xml", spec=METADATA)
    t.step("snapshot gav metadata md5", "GET", sbase + "lib/1.1.0-SNAPSHOT/maven-metadata.xml.md5", spec=NONE)
    t.step("snapshot non-unique path", "GET", sbase + "lib/1.1.0-SNAPSHOT/lib-1.1.0-SNAPSHOT.jar", spec=NONE)
    gbase = "/repository/%s/com/example/sbs/" % grp
    t.step("group ga metadata", "GET", gbase + "lib/maven-metadata.xml", spec=METADATA)
    t.step("group gav snapshot metadata", "GET", gbase + "lib/1.1.0-SNAPSHOT/maven-metadata.xml", spec=METADATA)
    t.step("group jar", "GET", gbase + "lib/1.0.0/lib-1.0.0.jar", spec=BYTES)
    t.step("group pom sha1", "GET", gbase + "lib/1.0.0/lib-1.0.0.pom.sha1", spec=BYTES)
    t.step("group missing", "GET", gbase + "lib/9.9.9/lib-9.9.9.pom", spec=NONE)
    t.step("group archetype catalog", "GET", "/repository/%s/archetype-catalog.xml" % grp, spec=NONE)
    t.step("put to group", "PUT", gbase + "lib/1.0.1/lib-1.0.1.pom", body=pom("lib", "1.0.1"),
           headers={"Content-Type": "application/xml"}, spec=NONE)
    t.step("put bad layout", "PUT", "/repository/%s/not/a/maven/file.txt" % releases, body=b"x",
           headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("put bad layout permissive", "PUT", "/repository/%s/not/a/maven/file.txt" % mixed, body=b"x",
           headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("get permissive file", "GET", "/repository/%s/not/a/maven/file.txt" % mixed, spec=BYTES)
    t.step("put pom", "PUT", "/repository/%s/com/example/sbs/raw/1.0/raw-1.0.pom" % mixed, body=pom("raw", "1.0"),
           headers={"Content-Type": "application/xml"}, spec=NONE)
    t.step("put wrong checksum", "PUT", "/repository/%s/com/example/sbs/raw/1.0/raw-1.0.pom.sha1" % mixed,
           body=b"0000000000000000000000000000000000000000", headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("checksum after wrong put", "GET", "/repository/%s/com/example/sbs/raw/1.0/raw-1.0.pom.sha1" % mixed,
           spec=BYTES)
    t.step("delete pom", "DELETE", "/repository/%s/com/example/sbs/raw/1.0/raw-1.0.pom" % mixed, spec=NONE)
    t.step("metadata after delete", "GET", "/repository/%s/com/example/sbs/raw/maven-metadata.xml" % mixed, spec=NONE)

    fresh = t.path("m2-resolve", "x")[:-2]
    with t.recording("mvn get", spec=exchange_spec, keep=keep_maven) as base:
        resolve_settings = settings(t, "%s/repository/%s/" % (base, grp))
        mvn(t, "dependency:get release", resolve_settings, fresh, "%s:get" % DEPENDENCY,
            "-Dartifact=%s:app:2.0.0" % GROUP, "-Dtransitive=true")
        mvn(t, "dependency:get snapshot", resolve_settings, fresh, "%s:get" % DEPENDENCY,
            "-Dartifact=%s:lib:1.1.0-SNAPSHOT" % GROUP, "-Dtransitive=false")
        mvn(t, "dependency:get sources", resolve_settings, fresh, "%s:get" % DEPENDENCY,
            "-Dartifact=%s:lib:1.0.0:jar:sources" % GROUP, "-Dtransitive=false")
        mvn(t, "dependency:get missing", resolve_settings, fresh, "%s:get" % DEPENDENCY,
            "-Dartifact=%s:nothing:1.0" % GROUP, "-Dtransitive=false")
    got = os.path.join(fresh, "com", "example", "sbs", "app", "2.0.0", "app-2.0.0.jar")
    t.fact("resolved app jar", os.path.exists(got))

    rest_listing(t, "rest releases", releases, derived=metadata_asset)
    rest_listing(t, "rest snapshots", snapshots, derived=metadata_asset)
    settle_search(t, releases)
    settle_search(t, snapshots)
    derived = without_assets(metadata_asset)
    t.json_step("search gav", "/service/rest/v1/search?maven.groupId=%s&maven.artifactId=lib&repository=%s" % (GROUP, releases),
                transform=derived)
    t.json_step("search base version", "/service/rest/v1/search?maven.groupId=%s&maven.baseVersion=1.1.0-SNAPSHOT&repository=%s"
                % (GROUP, snapshots), transform=derived)
    t.json_step("search classifier", "/service/rest/v1/search/assets?maven.groupId=%s&maven.classifier=sources" % GROUP,
                transform=derived)
    t.json_step("search sort version", "/service/rest/v1/search?repository=%s&sort=version" % releases,
                spec=Spec(body="json", unordered=False, transform=chain(derived, sorted_assets)))
    t.json_step("search download latest snapshot", "/service/rest/v1/search/assets/download?repository=%s&maven.groupId=%s"
                "&maven.artifactId=lib&maven.baseVersion=1.1.0-SNAPSHOT&maven.extension=jar&sort=version" % (snapshots, GROUP),
                body="none")
    t.json_step("search download release", "/service/rest/v1/search/assets/download?repository=%s&maven.groupId=%s"
                "&maven.artifactId=lib&maven.extension=jar&maven.classifier=" % (releases, GROUP), body="none")
    t.fact("component count releases", len(components(t, releases)))


@scenario("maven-proxy", formats=["maven2"], clients=["mvn"], proxies=True)
def maven_proxy(t):
    """maven2 proxy of a local upstream and a group over it, read with mvn dependency:get."""
    if not t.context.proxies:
        raise Skip("proxies to 127.0.0.1 are not allowed")
    up = t.context.upstream()
    base = "/maven/org/example/up/remote/1.0/"
    up.add(base + "remote-1.0.jar", jar("remote", "remote\n"), "application/java-archive")
    up.add(base + "remote-1.0.pom", pom("remote", "1.0").replace(GROUP.encode(), b"org.example.up"), "application/xml")
    up.add("/maven/org/example/up/remote/maven-metadata.xml",
           "<metadata><groupId>org.example.up</groupId><artifactId>remote</artifactId><versioning><latest>1.0</latest>"
           "<release>1.0</release><versions><version>1.0</version></versions><lastUpdated>20200101000000</lastUpdated>"
           "</versioning></metadata>", "application/xml")
    p = t.prefix
    repo = t.create_repository("maven2", "proxy", proxy(p + "maven-proxy", up.base + "/maven/",
                               {"maven": {"versionPolicy": "RELEASE", "layoutPolicy": "PERMISSIVE"}}), step="create proxy")
    grp = t.create_repository("maven2", "group", group(p + "maven-proxy-group", [repo],
                                                         {"maven": {"versionPolicy": "MIXED", "layoutPolicy": "STRICT"}}),
                              step="create group")
    rbase = "/repository/%s/org/example/up/remote/" % repo
    t.step("jar", "GET", rbase + "1.0/remote-1.0.jar", spec=BYTES)
    t.step("jar sha1 derived", "GET", rbase + "1.0/remote-1.0.jar.sha1", spec=BYTES)
    t.step("metadata", "GET", rbase + "maven-metadata.xml", spec=METADATA)
    t.step("missing", "GET", rbase + "2.0/remote-2.0.jar", spec=NONE)
    t.step("group metadata", "GET", "/repository/%s/org/example/up/remote/maven-metadata.xml" % grp, spec=METADATA)
    t.need("mvn")
    fresh = t.path("m2", "x")[:-2]
    with t.recording("mvn get", spec=exchange_spec, keep=lambda m, p: "/org/example/up/" in p) as rbase:
        resolve_settings = settings(t, "%s/repository/%s/" % (rbase, grp))
        mvn(t, "dependency:get", resolve_settings, fresh, "%s:get" % DEPENDENCY,
            "-Dartifact=org.example.up:remote:1.0", "-Dtransitive=false")
    rest_listing(t, "rest", repo, derived=metadata_asset)
