import fnmatch
import re

from .. import harness
from ..client import HarnessError
from . import access, docker, gitlfs, helm, maven, npm, nuget, pypi, raw, rest, rubygems, swift, terraform, yum

MODULES = (access, rest, raw, maven, npm, pypi, docker, helm, nuget, rubygems, gitlfs, yum, terraform, swift)

LOOPBACK = "127.0.0.1"


def ssrf(server):
    response = server.json("GET", "/service/rest/v1/security/ssrf-protection", ok=None)
    if response.status != 200:
        return None
    return response.json()


def loopback_allowed(settings):
    if settings is None:
        return True
    if not settings.get("enabled", True):
        return True
    return LOOPBACK in (settings.get("allowedIPs") or [])


def prepare_proxies(context):
    args = context.args
    context.ssrf_restore = {}
    if args.proxies == "off":
        context.proxies = False
        return
    allowed = True
    for side, server in context.servers.items():
        settings = ssrf(server)
        if loopback_allowed(settings):
            continue
        if side in args.allow_loopback:
            changed = dict(settings)
            changed["allowedIPs"] = list(settings.get("allowedIPs") or []) + [LOOPBACK]
            server.json("PUT", "/service/rest/v1/security/ssrf-protection", changed, ok=(200, 204))
            context.ssrf_restore[side] = settings
            context.log.info("   %s: 127.0.0.1 added to the SSRF allow list for the proxy scenarios" % side)
            continue
        allowed = False
        context.log.info("   %s does not allow proxies to 127.0.0.1 (SSRF protection); proxy scenarios are skipped "
                         "(--allow-loopback %s lifts it while they run)" % (side, side))
    if args.proxies == "on" and not allowed:
        raise HarnessError("--proxies on needs 127.0.0.1 in the SSRF allow list of both servers")
    context.proxies = allowed


def restore_proxies(context):
    for side, settings in getattr(context, "ssrf_restore", {}).items():
        try:
            context.servers[side].json("PUT", "/service/rest/v1/security/ssrf-protection", settings, ok=(200, 204))
            context.log.info("   %s: SSRF settings restored" % side)
        except Exception as error:
            context.log.info("   %s: RESTORING THE SSRF SETTINGS FAILED: %s" % (side, error))


def sync_settings(context):
    ref = context.servers["reference"]
    cand = context.servers["candidate"]
    context.settings_restore = []
    for path in ("/service/rest/v1/security/anonymous", "/service/rest/v1/security/realms/active"):
        wanted = ref.get_json(path)
        current = cand.get_json(path)
        if wanted != current:
            response = cand.json("PUT", path, wanted, ok=None)
            if response.status in (200, 204):
                context.settings_restore.append((path, current))
                context.log.info("   candidate: %s set like the reference for the run" % path.rsplit("/v1/", 1)[1])
            else:
                context.log.info("   candidate refused %s like the reference: %s" % (path, response.status))


def restore_settings(context):
    for path, value in reversed(getattr(context, "settings_restore", [])):
        try:
            context.servers["candidate"].json("PUT", path, value, ok=(200, 204))
        except Exception as error:
            context.log.info("   candidate: restoring %s failed: %s" % (path, error))


SERVED = ("docker", "gitlfs", "helm", "huggingface", "maven2", "npm", "nuget", "pypi", "raw", "rubygems", "swift",
          "terraform", "yum")
FORMAT_PRIVILEGE = re.compile(r"^as-repository-view-([a-z0-9]+)-\*-browse$")


def served_formats(context):
    found = {}
    for side, server in context.servers.items():
        response = server.json("GET", "/service/rest/internal/ui/state", ok=None)
        state = response.json() if response.status == 200 and "json" in (response.header("Content-Type") or "") else None
        if isinstance(state, dict) and isinstance(state.get("formats"), list):
            found[side] = {f.get("id") for f in state["formats"] if isinstance(f, dict)}
            continue
        names = [p.get("name", "") for p in server.get_json("/service/rest/v1/security/privileges")]
        formats = {m.group(1) for m in map(FORMAT_PRIVILEGE.match, names) if m}
        found[side] = formats or set(SERVED)
    context.formats = found
    context.common_formats = found["reference"] & found["candidate"]
    only = sorted(found["reference"] - found["candidate"])
    if only:
        context.log.info("   formats only the reference serves: %s" % ", ".join(only))


def admin_roles(context):
    context.admin_roles = {}
    for side, server in context.servers.items():
        roles = [r["id"] for r in server.get_json("/service/rest/v1/security/roles")]
        role = next((r for r in ("as-admin", "nx-admin") if r in roles), None)
        if not role:
            raise HarnessError("%s has no administrator role (as-admin)" % side)
        context.admin_roles[side] = role


def run(context, allow):
    args = context.args
    items = [i for i in harness.selected(args.scenario)
             if not any(fnmatch.fnmatchcase(i.name, p) for p in args.skip)]
    admin_roles(context)
    served_formats(context)
    try:
        sync_settings(context)
        prepare_proxies(context)
        harness.ensure_client_users(context)
        return harness.run_scenarios(context, items, allow, "run")
    finally:
        context.close()
        harness.drop_client_users(context)
        restore_proxies(context)
        restore_settings(context)
