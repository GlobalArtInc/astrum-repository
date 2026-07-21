import secrets

from ..compare import Spec
from ..harness import scenario
from .common import JSON, dumps, hosted

NONE = Spec(body="none")


@scenario("access")
def access(t):
    """Anonymous access, wrong credentials, missing users and paths, a user with read rights on one repository."""
    p = t.prefix
    repo = t.create_repository("raw", "hosted", hosted(p + "access-raw"), step="create raw hosted")
    other = t.create_repository("raw", "hosted", hosted(p + "access-other"), step="create other raw hosted")
    t.step("put file", "PUT", "/repository/%s/a/file.txt" % repo, body=b"access\n",
           headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("put other file", "PUT", "/repository/%s/a/file.txt" % other, body=b"other\n",
           headers={"Content-Type": "text/plain"}, spec=NONE)

    suffix = t.shared.setdefault("suffix", secrets.token_hex(3))
    victim = p + "victim-" + suffix
    t.request("DELETE", "/service/rest/v1/security/users/" + victim)
    t.step("victim user", "POST", "/service/rest/v1/security/users", headers=JSON, spec=NONE,
           body=dumps({"userId": victim, "firstName": "Wrong", "lastName": "Password",
                       "emailAddress": victim + "@example.invalid", "password": "Victim-pass-1", "status": "active",
                       "roles": []}))
    t.defer(t.request, "DELETE", "/service/rest/v1/security/users/" + victim)
    t.step("wrong password repositories", "GET", "/service/rest/v1/repositories", auth=(victim, "wrong-password"),
           headers={"Accept": "application/json"}, spec=NONE)
    t.step("wrong password get file", "GET", "/repository/%s/a/file.txt" % repo, auth=(victim, "wrong-password"), spec=NONE)
    t.step("wrong password put file", "PUT", "/repository/%s/a/new.txt" % repo, body=b"x", auth=(victim, "wrong-password"),
           headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("right password after failures", "GET", "/service/rest/v1/status", auth=(victim, "Victim-pass-1"), spec=NONE)
    for label, auth in (("anonymous", None), ("unknown user", (p + "nobody-" + suffix, "x")), ("malformed basic", "Basic !!!"),
                        ("unknown bearer", "Bearer astrum_" + t.shared.setdefault("token", secrets.token_hex(20)))):
        t.step(label + " repositories", "GET", "/service/rest/v1/repositories", auth=auth, headers={"Accept": "application/json"},
               spec=Spec(body="auto"))
        t.step(label + " status", "GET", "/service/rest/v1/status", auth=auth, spec=NONE)
        t.step(label + " get file", "GET", "/repository/%s/a/file.txt" % repo, auth=auth, spec=NONE)
        t.step(label + " head file", "HEAD", "/repository/%s/a/file.txt" % repo, auth=auth, spec=NONE)
        t.step(label + " put file", "PUT", "/repository/%s/a/new.txt" % repo, body=b"x", auth=auth,
               headers={"Content-Type": "text/plain"}, spec=NONE)
        t.step(label + " missing file", "GET", "/repository/%s/a/missing.txt" % repo, auth=auth, spec=NONE)
        t.step(label + " missing repository", "GET", "/repository/%smissing/a.txt" % p, auth=auth, spec=NONE)
        t.step(label + " users", "GET", "/service/rest/v1/security/users", auth=auth, spec=NONE)
        t.step(label + " browse", "GET", "/service/rest/repository/browse/%s/" % repo, auth=auth, spec=NONE)
        t.step(label + " search", "GET", "/service/rest/v1/search?repository=" + repo, auth=auth,
               headers={"Accept": "application/json"}, spec=Spec(body="json"))

    role = p + "access-reader"
    user = p + "reader-" + suffix
    t.request("DELETE", "/service/rest/v1/security/users/" + user)
    t.request("DELETE", "/service/rest/v1/security/roles/" + role)
    t.step("reader role", "POST", "/service/rest/v1/security/roles", headers=JSON, spec=NONE,
           body=dumps({"id": role, "name": role, "description": "side by side",
                       "privileges": ["as-repository-view-raw-%s-read" % repo, "as-repository-view-raw-%s-browse" % repo],
                       "roles": []}))
    t.defer(t.request, "DELETE", "/service/rest/v1/security/roles/" + role)
    password = "Reader-pass-1"
    t.step("reader user", "POST", "/service/rest/v1/security/users", headers=JSON, spec=NONE,
           body=dumps({"userId": user, "firstName": "Read", "lastName": "Only", "emailAddress": user + "@example.invalid",
                       "password": password, "status": "active", "roles": [role]}))
    t.defer(t.request, "DELETE", "/service/rest/v1/security/users/" + user)
    reader = (user, password)
    t.step("reader get file", "GET", "/repository/%s/a/file.txt" % repo, auth=reader, spec=Spec(body="bytes"))
    t.step("reader head file", "HEAD", "/repository/%s/a/file.txt" % repo, auth=reader, spec=NONE)
    t.step("reader put file", "PUT", "/repository/%s/a/new.txt" % repo, body=b"x", auth=reader,
           headers={"Content-Type": "text/plain"}, spec=NONE)
    t.step("reader delete file", "DELETE", "/repository/%s/a/file.txt" % repo, auth=reader, spec=NONE)
    t.step("reader other repository", "GET", "/repository/%s/a/file.txt" % other, auth=reader, spec=NONE)
    t.step("reader missing file", "GET", "/repository/%s/a/missing.txt" % repo, auth=reader, spec=NONE)
    t.step("reader repositories", "GET", "/service/rest/v1/repositories", auth=reader,
           headers={"Accept": "application/json"}, spec=Spec(body="json", transform=lambda v: [
               i for i in v if i.get("name", "").startswith(p)] if isinstance(v, list) else v))
    t.step("reader components", "GET", "/service/rest/v1/components?repository=" + repo, auth=reader,
           headers={"Accept": "application/json"}, spec=Spec(body="json"))
    t.step("reader components other", "GET", "/service/rest/v1/components?repository=" + other, auth=reader,
           headers={"Accept": "application/json"}, spec=NONE)
    t.step("reader search", "GET", "/service/rest/v1/search?repository=" + other, auth=reader,
           headers={"Accept": "application/json"}, spec=Spec(body="json"))
    t.step("reader users", "GET", "/service/rest/v1/security/users", auth=reader, spec=NONE)
    t.step("reader create repository", "POST", "/service/rest/v1/repositories/raw/hosted", auth=reader, headers=JSON,
           body=dumps(hosted(p + "access-denied")), spec=NONE)
    t.step("reader browse", "GET", "/service/rest/repository/browse/%s/" % repo, auth=reader, spec=NONE)
    t.step("reader browse other", "GET", "/service/rest/repository/browse/%s/" % other, auth=reader, spec=NONE)

    t.step("disabled user setup", "PUT", "/service/rest/v1/security/users/" + user, headers=JSON, spec=NONE,
           body=dumps({"userId": user, "firstName": "Read", "lastName": "Only", "emailAddress": user + "@example.invalid",
                       "status": "disabled", "roles": [role], "source": "default", "readOnly": False}))
    t.step("disabled user get file", "GET", "/repository/%s/a/file.txt" % repo, auth=reader, spec=NONE)
    for attempt in range(1, 7):
        t.step("repeated wrong password %d" % attempt, "GET", "/service/rest/v1/status", auth=(victim, "wrong"), spec=NONE)
    t.step("right password after repeated failures", "GET", "/service/rest/v1/status", auth=(victim, "Victim-pass-1"),
           spec=NONE)

    t.step("root", "GET", "/repository/", spec=NONE)
    t.step("repository root", "GET", "/repository/%s/" % repo, spec=NONE)
    t.step("traversal", "GET", "/repository/%s/a/../../%s/a/file.txt" % (repo, other), spec=NONE)
    t.step("encoded traversal", "GET", "/repository/%s/a/%%2e%%2e/%%2e%%2e/%s/a/file.txt" % (repo, other), spec=NONE)
    t.step("unknown service path", "GET", "/service/%snothing" % p, spec=NONE)
    t.step("method not allowed", "PATCH", "/repository/%s/a/file.txt" % repo, body=b"x", spec=NONE)
