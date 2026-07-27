import gzip

from ..compare import Spec
from ..harness import scenario
from .common import JSON, dumps, hosted, multipart, only, quote, settle_search, strip_keys

BUILTIN_ROLES = ("as-admin", "as-anonymous")
def shared_privileges(prefix, formats):
    def keep(item):
        if not isinstance(item, dict):
            return False
        if item.get("name", "").startswith(prefix):
            return True
        if not item.get("readOnly"):
            return False
        fmt = item.get("format")
        if fmt and fmt != "*" and fmt not in formats:
            return False
        repository = item.get("repository")
        return repository in (None, "*", "*-" + str(fmt)) or str(repository).startswith(prefix) or \
            str(repository).startswith("*-")

    def transform(value):
        return [i for i in value if keep(i)] if isinstance(value, list) else value
    return transform


@scenario("rest")
def rest(t):
    """Public REST API: repositories, components, assets, search, blob stores, security, tasks, settings."""
    p = t.prefix
    repo = t.create_repository("raw", "hosted", hosted(p + "rest-raw", {"raw": {"contentDisposition": "ATTACHMENT"}}),
                               step="create raw hosted")
    for path, content, ctype in (("docs/readme.txt", b"side by side\n", "text/plain"),
                                 ("docs/data.json", b'{"a": 1}\n', "application/json"),
                                 ("bin/blob.gz", gzip.compress(bytes(range(256)) * 8, mtime=0), "application/gzip")):
        t.step("put " + path, "PUT", "/repository/%s/%s" % (repo, path), body=content,
               headers={"Content-Type": ctype}, spec=Spec(body="none"))
    body, ctype = multipart([("raw.directory", "upload"), ("raw.asset1.filename", "form.txt")],
                            [("raw.asset1", "form.txt", b"uploaded through the components API\n", "text/plain")])
    t.step("components upload", "POST", "/service/rest/v1/components?repository=" + repo, body=body,
           headers={"Content-Type": ctype}, spec=Spec(body="none"))

    t.json_step("repositories", "/service/rest/v1/repositories", transform=only(p))
    t.json_step("repository", "/service/rest/v1/repositories/" + repo)
    t.json_step("repository raw hosted", "/service/rest/v1/repositories/raw/hosted/" + repo)
    t.json_step("repository wrong type", "/service/rest/v1/repositories/raw/proxy/" + repo, body="none")
    t.json_step("repositorySettings", "/service/rest/v1/repositorySettings", transform=only(p))
    t.json_step("repository missing", "/service/rest/v1/repositories/%smissing" % p, body="none")

    t.json_step("components", "/service/rest/v1/components?repository=" + repo)
    t.json_step("assets", "/service/rest/v1/assets?repository=" + repo)
    t.json_step("components without repository", "/service/rest/v1/components", body="none")
    t.json_step("components unknown repository", "/service/rest/v1/components?repository=%smissing" % p, body="none")
    listed = t.server.get_json("/service/rest/v1/components?repository=" + repo)["items"]
    readme = next((c for c in listed if c.get("name", "").endswith("readme.txt")), None)
    if readme:
        t.json_step("component by id", "/service/rest/v1/components/" + quote(readme["id"]))
        asset = readme["assets"][0]
        t.json_step("asset by id", "/service/rest/v1/assets/" + quote(asset["id"]))
    t.json_step("component bad id", "/service/rest/v1/components/not-an-id", body="none")
    t.json_step("asset bad id", "/service/rest/v1/assets/not-an-id", body="none")

    settle_search(t, repo)
    t.json_step("search by repository", "/service/rest/v1/search?repository=" + repo)
    t.json_step("search by name", "/service/rest/v1/search?repository=%s&name=docs/readme.txt" % repo)
    t.json_step("search by wildcard", "/service/rest/v1/search?repository=%s&name=docs/*" % repo)
    t.json_step("search q", "/service/rest/v1/search?q=readme&repository=" + repo)
    t.json_step("search format", "/service/rest/v1/search?format=raw&repository=" + repo)
    t.json_step("search sha1", "/service/rest/v1/search?sha1=0000000000000000000000000000000000000000")
    t.json_step("search assets", "/service/rest/v1/search/assets?repository=%s&name=docs/*" % repo)
    t.json_step("search assets download one", "/service/rest/v1/search/assets/download?repository=%s&name=docs/readme.txt"
                % repo, body="none")
    t.json_step("search assets download many", "/service/rest/v1/search/assets/download?repository=%s&name=docs/*"
                % repo, body="none")
    t.json_step("search assets download none", "/service/rest/v1/search/assets/download?repository=%s&name=nothing"
                % repo, body="none")
    t.json_step("search sort", "/service/rest/v1/search?repository=%s&sort=name&direction=desc" % repo,
                spec=Spec(body="json", unordered=False))

    counts = ("blobCount", "totalSizeInBytes", "availableSpaceInBytes")
    t.json_step("blobstores", "/service/rest/v1/blobstores", transform=only("default"), volatile=counts)
    t.json_step("blobstore file default", "/service/rest/v1/blobstores/file/default")
    t.json_step("blobstore quota status", "/service/rest/v1/blobstores/default/quota-status")
    t.json_step("blobstore missing", "/service/rest/v1/blobstores/file/%smissing" % p, body="none")

    common = t.context.common_formats
    t.fact("served formats", sorted(t.context.formats[t.side]))
    t.json_step("privileges", "/service/rest/v1/security/privileges", transform=shared_privileges(p, common))
    t.json_step("privilege builtin", "/service/rest/v1/security/privileges/as-all")
    t.json_step("privilege repository view", "/service/rest/v1/security/privileges/as-repository-view-raw-%s-read" % repo)
    t.json_step("privilege missing", "/service/rest/v1/security/privileges/%smissing" % p, body="none")
    t.json_step("roles", "/service/rest/v1/security/roles", transform=only(p, key="id", keep=BUILTIN_ROLES))
    t.json_step("role admin", "/service/rest/v1/security/roles/as-admin")
    t.json_step("role anonymous", "/service/rest/v1/security/roles/as-anonymous")
    t.json_step("role missing", "/service/rest/v1/security/roles/%smissing" % p, body="none")
    t.json_step("users by id", "/service/rest/v1/security/users?userId=%sclient" % p, transform=only(p, key="userId"))
    t.json_step("users admin", "/service/rest/v1/security/users?userId=admin", transform=only("admin", key="userId"))
    t.json_step("users anonymous", "/service/rest/v1/security/users?userId=anonymous",
                transform=only("anonymous", key="userId"))
    t.json_step("user sources", "/service/rest/v1/security/user-sources")
    t.json_step("realms available", "/service/rest/v1/security/realms/available")
    t.json_step("realms active", "/service/rest/v1/security/realms/active", spec=Spec(body="json", unordered=False))
    t.json_step("anonymous", "/service/rest/v1/security/anonymous")
    t.json_step("ssrf", "/service/rest/v1/security/ssrf-protection")

    selector = p + "selector"
    t.request("DELETE", "/service/rest/v1/security/content-selectors/" + selector)
    t.step("selector create", "POST", "/service/rest/v1/security/content-selectors",
           body=dumps({"name": selector, "description": "side by side", "expression": 'format == "raw" and path =^ "/docs/"'}),
           headers=JSON, spec=Spec(body="none"))
    t.defer(t.request, "DELETE", "/service/rest/v1/security/content-selectors/" + selector)
    t.json_step("selectors", "/service/rest/v1/security/content-selectors", transform=only(p))
    t.json_step("selector", "/service/rest/v1/security/content-selectors/" + selector)
    t.step("selector invalid", "POST", "/service/rest/v1/security/content-selectors",
           body=dumps({"name": p + "bad", "expression": "format =="}), headers=JSON, spec=Spec(body="none"))
    t.request("DELETE", "/service/rest/v1/security/content-selectors/" + p + "bad")

    privilege = p + "priv"
    t.request("DELETE", "/service/rest/v1/security/privileges/" + privilege)
    t.step("privilege create", "POST", "/service/rest/v1/security/privileges/repository-view",
           body=dumps({"name": privilege, "description": "side by side", "actions": ["READ", "BROWSE"],
                       "format": "raw", "repository": repo}), headers=JSON, spec=Spec(body="none"))
    t.defer(t.request, "DELETE", "/service/rest/v1/security/privileges/" + privilege)
    t.json_step("privilege custom", "/service/rest/v1/security/privileges/" + privilege)
    role = p + "role"
    t.request("DELETE", "/service/rest/v1/security/roles/" + role)
    t.step("role create", "POST", "/service/rest/v1/security/roles",
           body=dumps({"id": role, "name": role, "description": "side by side", "privileges": [privilege],
                       "roles": ["as-anonymous"]}), headers=JSON, spec=Spec(body="json"))
    t.defer(t.request, "DELETE", "/service/rest/v1/security/roles/" + role)
    t.json_step("role custom", "/service/rest/v1/security/roles/" + role)
    t.step("role duplicate", "POST", "/service/rest/v1/security/roles",
           body=dumps({"id": role, "name": role, "privileges": [], "roles": []}), headers=JSON, spec=Spec(body="none"))
    user = p + "user"
    t.request("DELETE", "/service/rest/v1/security/users/" + user)
    t.step("user create", "POST", "/service/rest/v1/security/users",
           body=dumps({"userId": user, "firstName": "Side", "lastName": "User", "emailAddress": user + "@example.invalid",
                       "password": "Side-by-side-1", "status": "active", "roles": [role]}), headers=JSON,
           spec=Spec(body="json"))
    t.defer(t.request, "DELETE", "/service/rest/v1/security/users/" + user)
    t.step("user update", "PUT", "/service/rest/v1/security/users/" + user,
           body=dumps({"userId": user, "firstName": "Side", "lastName": "Changed", "emailAddress": user + "@example.invalid",
                       "status": "active", "roles": [role], "source": "default", "readOnly": False}), headers=JSON,
           spec=Spec(body="none"))
    t.json_step("user after update", "/service/rest/v1/security/users?userId=" + user, transform=only(user, key="userId"))
    t.step("user change password", "PUT", "/service/rest/v1/security/users/%s/change-password" % user,
           body=b"Side-by-side-2", headers={"Content-Type": "text/plain"}, spec=Spec(body="none"))
    t.step("user new password works", "GET", "/service/rest/v1/status", auth=(user, "Side-by-side-2"),
           spec=Spec(body="none"))
    t.step("user duplicate", "POST", "/service/rest/v1/security/users",
           body=dumps({"userId": user, "firstName": "a", "lastName": "b", "emailAddress": "c@example.invalid",
                       "password": "x", "status": "active", "roles": []}), headers=JSON, spec=Spec(body="none"))

    rule = p + "rule"
    t.request("DELETE", "/service/rest/v1/routing-rules/" + rule)
    t.step("routing rule create", "POST", "/service/rest/v1/routing-rules",
           body=dumps({"name": rule, "description": "side by side", "mode": "BLOCK", "matchers": ["^/secret/.*"]}),
           headers=JSON, spec=Spec(body="none"))
    t.defer(t.request, "DELETE", "/service/rest/v1/routing-rules/" + rule)
    t.json_step("routing rules", "/service/rest/v1/routing-rules", transform=only(p))
    t.json_step("routing rule", "/service/rest/v1/routing-rules/" + rule)

    t.json_step("tasks", "/service/rest/v1/tasks", body="keys", transform=strip_keys("properties"))
    t.json_step("tasks by type", "/service/rest/v1/tasks?type=blobstore.compact", body="keys",
                transform=strip_keys("properties"))
    t.json_step("task templates", "/service/rest/v1/tasks/templates", body="keys",
                transform=strip_keys("properties", "formFields"))
    templates = t.server.get_json("/service/rest/v1/tasks/templates")
    t.fact("task types", sorted(i.get("type") for i in templates if isinstance(i, dict)))
    t.json_step("task missing", "/service/rest/v1/tasks/%smissing" % p, body="none")
    t.json_step("status", "/service/rest/v1/status", body="none")
    t.json_step("status writable", "/service/rest/v1/status/writable", body="none")
    t.json_step("status check", "/service/rest/v1/status/check", body="keys")
    t.json_step("upload specs raw", "/service/rest/v1/formats/raw/upload-specs")
    t.json_step("upload specs", "/service/rest/v1/formats/upload-specs",
                transform=lambda v: [i for i in v if i.get("format") in common] if isinstance(v, list) else v)
    t.json_step("http settings", "/service/rest/v1/http", body="keys")
    t.json_step("email settings", "/service/rest/v1/email", body="keys")
    t.json_step("capabilities", "/service/rest/v1/capabilities", body="keys")
    t.json_step("capability types", "/service/rest/v1/capabilities/types", body="keys")
    t.json_step("lifecycle phase", "/service/rest/v1/lifecycle/phase", body="none")
    t.json_step("truststore", "/service/rest/v1/security/ssl/truststore", body="keys")
    t.json_step("script", "/service/rest/v1/script", body="none")
    t.json_step("unknown endpoint", "/service/rest/v1/%snothing" % p, body="none")

    bad = dict(hosted(p + "rest-bad"))
    del bad["storage"]
    t.step("create invalid", "POST", "/service/rest/v1/repositories/raw/hosted", body=dumps(bad), headers=JSON,
           spec=Spec(body="none"))
    t.request("DELETE", "/service/rest/v1/repositories/" + p + "rest-bad")
    t.step("create duplicate", "POST", "/service/rest/v1/repositories/raw/hosted", body=dumps(hosted(repo)),
           headers=JSON, spec=Spec(body="none"))
    t.step("create wrong content type", "POST", "/service/rest/v1/repositories/raw/hosted",
           body=dumps(hosted(p + "rest-ct")), headers={"Content-Type": "text/plain"}, spec=Spec(body="none"))
    t.request("DELETE", "/service/rest/v1/repositories/" + p + "rest-ct")
    t.step("delete missing", "DELETE", "/service/rest/v1/repositories/%smissing" % p, spec=Spec(body="none"))
    changed = hosted(repo, {"raw": {"contentDisposition": "INLINE"}}, write_policy="ALLOW_ONCE")
    t.step("update repository", "PUT", "/service/rest/v1/repositories/raw/hosted/" + repo, body=dumps(changed),
           headers=JSON, spec=Spec(body="none"))
    t.json_step("repository after update", "/service/rest/v1/repositories/raw/hosted/" + repo)
    t.step("invalidate cache hosted", "POST", "/service/rest/v1/repositories/%s/invalidate-cache" % repo,
           spec=Spec(body="none"))
    t.step("rebuild index", "POST", "/service/rest/v1/repositories/%s/rebuild-index" % repo, spec=Spec(body="none"))
    if readme:
        listed = t.server.get_json("/service/rest/v1/components?repository=" + repo)["items"]
        target = next((c for c in listed if c.get("name", "").endswith("data.json")), None)
        if target:
            t.step("delete component", "DELETE", "/service/rest/v1/components/" + quote(target["id"]),
                   spec=Spec(body="none"))
            t.step("delete component again", "DELETE", "/service/rest/v1/components/" + quote(target["id"]),
                   spec=Spec(body="none"))
    t.json_step("assets after delete", "/service/rest/v1/assets?repository=" + repo)
    settle_search(t, repo)
    t.json_step("search after delete", "/service/rest/v1/search?repository=" + repo)
