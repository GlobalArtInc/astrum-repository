import base64
import os

from ..compare import Spec
from ..harness import scenario
from .common import BYTES, NONE, group, hosted, rest_listing, settle_search

GEM = "sbs_gem"


def gemspec(t, version):
    folder = t.path("gem-%s" % version, "lib", "%s.rb" % GEM)
    with open(folder, "w") as handle:
        handle.write("module SbsGem\n  VERSION = %r\nend\n" % version)
    root = os.path.dirname(os.path.dirname(folder))
    with open(os.path.join(root, "%s.gemspec" % GEM), "w") as handle:
        handle.write("""Gem::Specification.new do |s|
  s.name = %r
  s.version = %r
  s.summary = "side-by-side test gem"
  s.authors = ["Side By Side"]
  s.files = ["lib/%s.rb"]
  s.license = "MIT"
  s.date = "2020-01-01"
end
""" % (GEM, version, GEM))
    return root


def gem_env(t):
    home = t.path("home", "x")[:-2]
    return {"HOME": home, "GEM_HOME": t.path("gems", "x")[:-2], "GEM_PATH": t.path("gems", "x")[:-2],
            "GEM_SPEC_CACHE": t.path("spec-cache", "x")[:-2], "GEMRC": t.path("gemrc"),
            "BUNDLE_USER_HOME": t.path("bundle-home", "x")[:-2], "BUNDLE_APP_CONFIG": t.path("bundle-config", "x")[:-2]}


GENERATED = Spec(body="text", skip_headers=("etag", "content-length"))
GZIP = Spec(body="gunzip", skip_headers=("etag", "content-length"))


def exchange_spec(method, path):
    name = path.split("?")[0].rstrip("/").rsplit("/", 1)[-1]
    if name in ("versions", "names") or "/info/" in path:
        return GENERATED
    return None


def keep_gems(method, path):
    return GEM in path or path.endswith(("/versions", "/names")) or "/api/v1/" in path


@scenario("rubygems", formats=["rubygems"], clients=["gem"])
def rubygems(t):
    """rubygems hosted and group: gem push, gem install, gem yank, bundler; compact index and legacy indexes."""
    p = t.prefix
    repo = t.create_repository("rubygems", "hosted", hosted(p + "gems-hosted"), step="create hosted")
    grp = t.create_repository("rubygems", "group", group(p + "gems-group", [repo]), step="create group")
    t.need("gem")
    env = gem_env(t)
    with open(env["GEMRC"], "w") as handle:
        handle.write("---\n:sources: []\n:update_sources: false\n")
    built = {}
    for version in ("1.0.0", "1.0.1", "2.0.0.pre"):
        root = gemspec(t, version)
        t.run("gem build " + version, ["gem", "build", "%s.gemspec" % GEM], env=dict(env, SOURCE_DATE_EPOCH="1577836800"),
              cwd=root, record=False)
        built[version] = os.path.join(root, "%s-%s.gem" % (GEM, version))
    user, password = t.client_user()
    key = "Basic " + base64.b64encode(("%s:%s" % (user, password)).encode()).decode()
    with t.recording("gem", spec=exchange_spec, keep=keep_gems) as proxied:
        host = "%s/repository/%s" % (proxied, repo)
        push_env = dict(env, GEM_HOST_API_KEY=key)
        for version, path in built.items():
            t.run("gem push " + version, ["gem", "push", path, "--host", host], env=push_env)
        t.run("gem push again", ["gem", "push", built["1.0.0"], "--host", host], env=push_env)
        source = proxied.replace("://", "://%s:%s@" % (user, password)) + "/repository/%s/" % grp
        t.run("gem install", ["gem", "install", GEM, "-v", "1.0.1", "--clear-sources", "--source", source,
                              "--no-document", "--install-dir", t.path("installed", "x")[:-2]], env=env)
        t.run("gem install missing", ["gem", "install", "sbs_nothing", "--clear-sources", "--source", source,
                                      "--no-document", "--install-dir", t.path("installed", "x")[:-2]], env=env)
        code, output = t.run("gem list remote", ["gem", "list", GEM, "--remote", "--all", "--clear-sources", "--source",
                                                 source], env=env, record=False)
        t.fact("gem list remote", {"code": code, "lines": sorted(l for l in output.splitlines() if GEM in l)})
        t.run("gem yank", ["gem", "yank", GEM, "-v", "1.0.0", "--host", host], env=push_env)
        app = t.path("app", "Gemfile")
        with open(app, "w") as handle:
            handle.write('source "%s"\ngem "%s", "~> 1.0"\n' % (source, GEM))
        if os.path.exists("/usr/local/bin/bundle") or os.path.exists("/usr/bin/bundle"):
            t.run("bundle lock", ["bundle", "lock"], env=dict(env, BUNDLE_GEMFILE=app), cwd=os.path.dirname(app))
            lock = os.path.join(os.path.dirname(app), "Gemfile.lock")
            t.fact("bundle lock resolved", any("%s (1.0.1)" % GEM in l for l in open(lock)) if os.path.exists(lock) else None)
    base = "/repository/%s/" % repo
    for label, path, spec in (("versions", "versions", GENERATED), ("names", "names", GENERATED),
                              ("info", "info/" + GEM, GENERATED), ("specs", "specs.4.8.gz", GZIP),
                              ("latest specs", "latest_specs.4.8.gz", GZIP),
                              ("prerelease specs", "prerelease_specs.4.8.gz", GZIP),
                              ("quick gemspec", "quick/Marshal.4.8/%s-1.0.1.gemspec.rz" % GEM, NONE),
                              ("gem file", "gems/%s-1.0.1.gem" % GEM, BYTES),
                              ("yanked gem file", "gems/%s-1.0.0.gem" % GEM, NONE),
                              ("dependencies", "api/v1/dependencies?gems=" + GEM, NONE),
                              ("dependencies json", "api/v1/dependencies.json?gems=" + GEM, Spec(body="json")),
                              ("missing info", "info/sbs_nothing", NONE),
                              ("api key", "api/v1/api_key", NONE)):
        t.step(label, "GET", base + path, spec=spec)
        t.step("group " + label, "GET", "/repository/%s/%s" % (grp, path), spec=spec)
    rest_listing(t, "rest", repo, derived=lambda path: not path.endswith(".gem") and "/quick/" not in path)
    settle_search(t, repo)
    t.json_step("search", "/service/rest/v1/search?repository=%s&name=%s" % (repo, GEM))
