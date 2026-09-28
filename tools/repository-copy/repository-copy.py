#!/usr/bin/env python3
import argparse
import base64
import concurrent.futures
import email.parser
import hashlib
import http.client
import io
import json
import os
import re
import secrets
import ssl
import sys
import tarfile
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile

PATH_FORMATS = {"maven2": "maven"}
HANDLED_FORMATS = ("maven2", "raw", "npm", "nuget", "helm", "pypi", "docker", "apt")
SKIPPED_USERS = ("admin", "anonymous")
INDEX_TYPES = (
    "application/vnd.oci.image.index.v1+json",
    "application/vnd.docker.distribution.manifest.list.v2+json",
)
MANIFEST_ACCEPT = ", ".join(INDEX_TYPES + (
    "application/vnd.oci.image.manifest.v1+json",
    "application/vnd.docker.distribution.manifest.v2+json",
    "application/vnd.docker.distribution.manifest.v1+prettyjws",
    "application/vnd.docker.distribution.manifest.v1+json",
))
CLEANUP_APIS = ("/service/rest/v1/cleanup-policies", "/service/rest/internal/cleanup-policies")
MAVEN_REBUILD_TASK = "repository.maven.rebuild-metadata"
CHECKSUMS = (".sha1", ".md5", ".sha256", ".sha512")
TEMPORARY_TAG = "repository-copy-untagged"
CHUNK = 1 << 20

_print_lock = threading.Lock()


def log(message=""):
    with _print_lock:
        print(message, flush=True)


class MigrationError(Exception):
    pass


class HttpError(MigrationError):
    def __init__(self, method, url, status, detail):
        self.status = status
        super().__init__("%s %s -> HTTP %s%s" % (method, url, status, (": " + detail) if detail else ""))


class FileBody:
    def __init__(self, path):
        self.path = path
        self.length = os.path.getsize(path)

    def open(self):
        return open(self.path, "rb")


class Base64FileBody:
    def __init__(self, path):
        self.path = path
        self.length = 4 * ((os.path.getsize(path) + 2) // 3)

    def open(self):
        return Base64Reader(open(self.path, "rb"))


class Base64Reader:
    def __init__(self, handle):
        self.handle = handle
        self.buffer = b""

    def read(self, size=-1):
        while size < 0 or len(self.buffer) < size:
            raw = self.handle.read(3 * 65536)
            if not raw:
                break
            self.buffer += base64.b64encode(raw)
        if size < 0:
            out, self.buffer = self.buffer, b""
        else:
            out, self.buffer = self.buffer[:size], self.buffer[size:]
        return out

    def close(self):
        self.handle.close()


class CompositeBody:
    def __init__(self, pieces):
        self.pieces = pieces
        self.length = sum(len(p) if isinstance(p, bytes) else p.length for p in pieces)

    def open(self):
        return ChainReader([io.BytesIO(p) if isinstance(p, bytes) else p.open() for p in self.pieces])


class ChainReader:
    def __init__(self, readers):
        self.readers = readers
        self.index = 0

    def read(self, size=-1):
        out = []
        remaining = size
        while self.index < len(self.readers) and (size < 0 or remaining > 0):
            chunk = self.readers[self.index].read(remaining if size >= 0 else -1)
            if not chunk:
                self.index += 1
                continue
            out.append(chunk)
            if size >= 0:
                remaining -= len(chunk)
        return b"".join(out)

    def close(self):
        for reader in self.readers:
            reader.close()


class Response:
    def __init__(self, status, headers, raw, url):
        self.status = status
        self.headers = headers
        self.raw = raw
        self.url = url

    def read(self):
        try:
            return self.raw.read()
        finally:
            self.raw.close()

    def json(self):
        data = self.read()
        return json.loads(data) if data else None

    def text(self, limit=400):
        try:
            data = self.read()
        except Exception:
            return ""
        return " ".join(data.decode("utf-8", "replace").split())[:limit]

    def close(self):
        self.raw.close()


class Client:
    def __init__(self, name, url, user, password, timeout, insecure):
        self.name = name
        self.base = url.rstrip("/")
        self.basic = "Basic " + base64.b64encode(("%s:%s" % (user, password)).encode("utf-8")).decode("ascii")
        self.timeout = timeout
        handlers = []
        if insecure:
            context = ssl.create_default_context()
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE
            handlers.append(urllib.request.HTTPSHandler(context=context))
        self.opener = urllib.request.build_opener(*handlers)
        self.tokens = {}
        self.lock = threading.Lock()

    def url(self, path):
        if path.startswith(("http://", "https://")):
            return path
        return self.base + path

    def open(self, method, path, body=None, headers=None, token_key=None, retries=3):
        url = self.url(path)
        attempt = 0
        redirects = 0
        challenged = False
        while True:
            request_headers = dict(headers or {})
            with self.lock:
                token = self.tokens.get(token_key) if token_key else None
            request_headers["Authorization"] = ("Bearer " + token) if token else self.basic
            reader = None
            data = body
            if body is not None and not isinstance(body, bytes):
                reader = body.open()
                data = reader
                request_headers["Content-Length"] = str(body.length)
            request = urllib.request.Request(url, data=data, method=method, headers=request_headers)
            try:
                response = self.opener.open(request, timeout=self.timeout)
                return Response(response.status, response.headers, response, url)
            except urllib.error.HTTPError as error:
                challenge = error.headers.get("WWW-Authenticate") or ""
                if error.code == 401 and not challenged and challenge.lower().startswith("bearer "):
                    error.close()
                    challenged = True
                    token_key = token_key or url
                    token = self.fetch_token(challenge)
                    with self.lock:
                        self.tokens[token_key] = token
                    continue
                location = urllib.parse.urljoin(url, error.headers.get("Location") or "")
                if error.code in (307, 308) and redirects < 5 and self.same_origin(location):
                    error.close()
                    redirects += 1
                    url = location
                    continue
                if error.code in (502, 503, 504) and attempt < retries:
                    error.close()
                    attempt += 1
                    time.sleep(2 ** attempt)
                    continue
                return Response(error.code, error.headers, error, url)
            except (urllib.error.URLError, http.client.HTTPException, OSError) as error:
                if attempt < retries:
                    attempt += 1
                    time.sleep(2 ** attempt)
                    continue
                reason = getattr(error, "reason", None) or error
                raise MigrationError("%s %s failed: %s" % (method, url, reason))
            finally:
                if reader is not None:
                    reader.close()

    def same_origin(self, url):
        return urllib.parse.urlsplit(url)[:2] == urllib.parse.urlsplit(self.base)[:2]

    def fetch_token(self, challenge):
        params = dict(re.findall(r'(\w+)="([^"]*)"', challenge))
        realm = params.get("realm")
        if not realm:
            raise MigrationError("%s sent a Bearer challenge without a realm" % self.name)
        query = []
        if params.get("service"):
            query.append(("service", params["service"]))
        for scope in (params.get("scope") or "").split():
            query.append(("scope", scope))
        url = realm + (("&" if "?" in realm else "?") + urllib.parse.urlencode(query) if query else "")
        request = urllib.request.Request(url, method="GET", headers={"Authorization": self.basic})
        try:
            with self.opener.open(request, timeout=self.timeout) as response:
                data = json.loads(response.read() or b"{}")
        except urllib.error.HTTPError as error:
            error.close()
            raise MigrationError("%s token request to %s failed with HTTP %s" % (self.name, realm, error.code))
        except (urllib.error.URLError, OSError) as error:
            raise MigrationError("%s token request to %s failed: %s" % (self.name, realm, error))
        token = data.get("token") or data.get("access_token")
        if not token:
            raise MigrationError("%s token endpoint %s returned no token" % (self.name, realm))
        return token

    def check(self, method, path, ok=(200,), **kwargs):
        response = self.open(method, path, **kwargs)
        if response.status not in ok:
            raise HttpError(method, response.url, response.status, response.text())
        return response

    def get_json(self, path, **kwargs):
        return self.check("GET", path, headers={"Accept": "application/json"}, **kwargs).json()

    def send_json(self, method, path, payload, ok=(200, 201, 204)):
        headers = {"Accept": "application/json"}
        body = b""
        if payload is not None:
            body = json.dumps(payload).encode("utf-8")
            headers["Content-Type"] = "application/json"
        response = self.open(method, path, body=body, headers=headers, retries=0)
        if response.status not in ok:
            raise HttpError(method, response.url, response.status, response.text())
        return response.read()

    def paginate(self, path, **params):
        token = None
        while True:
            query = dict(params)
            if token:
                query["continuationToken"] = token
            data = self.get_json(path + "?" + urllib.parse.urlencode(query))
            for item in data.get("items") or []:
                yield item
            token = data.get("continuationToken")
            if not token:
                return

    def try_json(self, path):
        response = self.open("GET", path, headers={"Accept": "application/json"}, retries=1)
        if response.status != 200:
            response.close()
            return None
        try:
            return response.json()
        except ValueError:
            return None


class Downloaded:
    def __init__(self, path, size, sha1, sha256, md5, content_type):
        self.path = path
        self.size = size
        self.sha1 = sha1
        self.sha256 = sha256
        self.md5 = md5
        self.content_type = content_type

    def remove(self):
        try:
            os.unlink(self.path)
        except OSError:
            pass


def download(client, path, temp_dir, headers=None, token_key=None, attempts=3):
    last = None
    for attempt in range(attempts):
        response = client.open("GET", path, headers=headers, token_key=token_key)
        if response.status != 200:
            raise HttpError("GET", response.url, response.status, response.text(200))
        handle = tempfile.NamedTemporaryFile(prefix="repository-copy-", dir=temp_dir, delete=False)
        sha1, sha256, md5 = hashlib.sha1(), hashlib.sha256(), hashlib.md5()
        size = 0
        try:
            with handle:
                while True:
                    chunk = response.raw.read(CHUNK)
                    if not chunk:
                        break
                    handle.write(chunk)
                    sha1.update(chunk)
                    sha256.update(chunk)
                    md5.update(chunk)
                    size += len(chunk)
            response.close()
            expected = response.headers.get("Content-Length")
            if expected is not None and expected.isdigit() and int(expected) != size:
                raise MigrationError("GET %s: got %d of %s bytes" % (response.url, size, expected))
            return Downloaded(handle.name, size, sha1.hexdigest(), sha256.hexdigest(), md5.hexdigest(),
                              response.headers.get("Content-Type"))
        except (MigrationError, OSError, http.client.HTTPException) as error:
            response.close()
            try:
                os.unlink(handle.name)
            except OSError:
                pass
            last = error
            time.sleep(2 ** attempt)
    raise MigrationError(str(last))


def multipart(fields, files):
    boundary = "----repository-copy-" + secrets.token_hex(12)
    pieces = []
    for name, value in fields:
        pieces.append(("--%s\r\nContent-Disposition: form-data; name=\"%s\"\r\n\r\n" % (boundary, name)).encode("utf-8")
                      + value.encode("utf-8") + b"\r\n")
    for name, filename, body in files:
        safe = filename.replace('"', "%22")
        pieces.append(("--%s\r\nContent-Disposition: form-data; name=\"%s\"; filename=\"%s\"\r\n"
                       "Content-Type: application/octet-stream\r\n\r\n" % (boundary, name, safe)).encode("utf-8"))
        pieces.append(body)
        pieces.append(b"\r\n")
    pieces.append(("--%s--\r\n" % boundary).encode("utf-8"))
    return CompositeBody(pieces), "multipart/form-data; boundary=" + boundary


def human(size):
    value = float(size or 0)
    for unit in ("B", "KiB", "MiB", "GiB", "TiB"):
        if value < 1024 or unit == "TiB":
            return ("%d %s" % (value, unit)) if unit == "B" else ("%.1f %s" % (value, unit))
        value /= 1024


def quote_name(name):
    return urllib.parse.quote(name, safe="")


def basename(path):
    return path.rstrip("/").rsplit("/", 1)[-1]


def read_password(path, side):
    if not path:
        raise MigrationError("--%s-password-file is required" % side)
    try:
        with open(path, "r", encoding="utf-8") as handle:
            value = handle.read().rstrip("\r\n")
    except OSError as error:
        raise MigrationError("cannot read the %s password file: %s" % (side, error.strerror))
    if not value:
        raise MigrationError("the %s password file is empty" % side)
    return value


def make_client(args, side, required=True):
    url = getattr(args, side + "_url")
    if not url:
        if required:
            raise MigrationError("--%s-url is required" % side)
        return None
    password = read_password(getattr(args, side + "_password_file"), side)
    client = Client(side, url, getattr(args, side + "_user"), password, args.timeout, args.insecure)
    response = client.open("GET", "/service/rest/v1/repositories", headers={"Accept": "application/json"}, retries=1)
    status = response.status
    response.close()
    if status in (401, 403):
        raise MigrationError("%s rejected the credentials of %s (HTTP %s)" % (url, getattr(args, side + "_user"), status))
    if status != 200:
        raise MigrationError("%s does not answer the repositories API (HTTP %s)" % (url, status))
    return client


def map_name(name, available):
    return name if name in available else None


def list_repositories(client):
    return client.get_json("/service/rest/v1/repositories")


def repository_settings(client, repo):
    path = "/service/rest/v1/repositories/%s/%s/%s" % (PATH_FORMATS.get(repo["format"], repo["format"]), repo["type"], quote_name(repo["name"]))
    return client.try_json(path)


def list_assets(client, repo):
    return list(client.paginate("/service/rest/v1/assets", repository=repo))


def list_components(client, repo):
    return list(client.paginate("/service/rest/v1/components", repository=repo))


def cleanup_api(client):
    for path in CLEANUP_APIS:
        data = client.try_json(path)
        if isinstance(data, list):
            return path, data
    return None, None


def built_in_all(privileges):
    for privilege in privileges:
        name = privilege.get("name") or ""
        if privilege.get("readOnly") and privilege.get("type") == "wildcard" and re.fullmatch(r"[a-z]{2}-all", name):
            return privilege
    return None


def wildcard_prefix(privileges):
    found = built_in_all(privileges)
    if found and found.get("pattern"):
        return found["pattern"].split(":", 1)[0]
    return None


def built_in_id_prefix(privileges):
    found = built_in_all(privileges)
    return found["name"][:3] if found else None


def renamed_id(name, prefixes):
    source, target = prefixes
    if not source or not target or not name.startswith(source):
        return name
    return target + name[len(source):]


def target_id(name, target_ids, prefixes):
    if name in target_ids:
        return name
    renamed = renamed_id(name, prefixes)
    return renamed if renamed in target_ids else name


def cmd_inventory(args):
    source = make_client(args, "source")
    log("Reading %s" % source.base)
    repositories = list_repositories(source)
    settings = {}
    bulk = source.try_json("/service/rest/v1/repositorySettings")
    if isinstance(bulk, list):
        settings = {item.get("name"): item for item in bulk}

    def describe(repo):
        item = {"name": repo["name"], "format": repo["format"], "type": repo["type"], "online": repo.get("online")}
        body = settings.get(repo["name"]) or repository_settings(source, repo) or {}
        item["blobStore"] = (body.get("storage") or {}).get("blobStoreName")
        if repo["type"] == "group":
            item["members"] = (body.get("group") or {}).get("memberNames")
        if repo["type"] == "proxy":
            item["remoteUrl"] = (body.get("proxy") or {}).get("remoteUrl")
        if repo["format"] == "docker":
            docker = body.get("docker") or {}
            item["docker"] = {k: docker.get(k) for k in ("httpPort", "httpsPort", "subdomain", "pathEnabled", "forceBasicAuth", "v1Enabled") if k in docker}
        if repo["type"] == "hosted":
            assets = 0
            size = 0
            for asset in source.paginate("/service/rest/v1/assets", repository=repo["name"]):
                assets += 1
                size += asset.get("fileSize") or 0
            item["components"] = sum(1 for _ in source.paginate("/service/rest/v1/components", repository=repo["name"]))
            item["assets"] = assets
            item["totalSize"] = size
            item["contentCopy"] = repo["format"] in HANDLED_FORMATS
        return item

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:
        described = list(pool.map(describe, repositories))

    blob_stores = []
    for store in source.get_json("/service/rest/v1/blobstores"):
        entry = dict(store)
        if (store.get("type") or "").lower() == "file":
            detail = source.try_json("/service/rest/v1/blobstores/file/" + quote_name(store["name"])) or {}
            entry["path"] = detail.get("path")
        blob_stores.append(entry)

    users = [u for u in source.get_json("/service/rest/v1/security/users?source=default")]
    roles = [r for r in source.get_json("/service/rest/v1/security/roles?source=default") if not r.get("readOnly")]
    privileges = [p for p in source.get_json("/service/rest/v1/security/privileges") if not p.get("readOnly")]
    selectors = source.get_json("/service/rest/v1/security/content-selectors")
    realms = source.get_json("/service/rest/v1/security/realms/active")
    anonymous = source.get_json("/service/rest/v1/security/anonymous")
    cleanup_path, cleanup = cleanup_api(source)
    routing = source.try_json("/service/rest/v1/routing-rules")

    inventory = {
        "source": source.base,
        "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "repositories": described,
        "blobStores": blob_stores,
        "users": users,
        "roles": roles,
        "privileges": privileges,
        "contentSelectors": selectors,
        "activeRealms": realms,
        "anonymous": anonymous,
        "cleanupPolicies": cleanup,
        "cleanupPoliciesApi": cleanup_path,
        "routingRules": routing,
    }
    if args.output:
        with open(args.output, "w", encoding="utf-8") as handle:
            json.dump(inventory, handle, indent=2)
        log("Inventory written to %s" % args.output)

    log("")
    log("Repositories (%d)" % len(described))
    header = "  %-32s %-10s %-7s %-14s %10s %10s %12s" % ("name", "format", "type", "blob store", "components", "assets", "size")
    log(header)
    for item in sorted(described, key=lambda r: ({"hosted": 0, "proxy": 1, "group": 2}.get(r["type"], 3), r["name"])):
        if item["type"] == "hosted":
            counts = "%10d %10d %12s" % (item["components"], item["assets"], human(item["totalSize"]))
            if not item["contentCopy"]:
                counts += "  (content copy not handled)"
        else:
            counts = "%10s %10s %12s" % ("-", "-", "-")
        log("  %-32s %-10s %-7s %-14s %s" % (item["name"], item["format"], item["type"], item.get("blobStore") or "-", counts))
    hosted = [r for r in described if r["type"] == "hosted"]
    log("  hosted total: %d components, %d assets, %s" % (
        sum(r["components"] for r in hosted), sum(r["assets"] for r in hosted), human(sum(r["totalSize"] for r in hosted))))
    log("")
    log("Blob stores: " + ", ".join("%s (%s, %s)" % (b["name"], b.get("type"), human(b.get("totalSizeInBytes"))) for b in blob_stores))
    log("Local users (%d): %s" % (len(users), ", ".join(u["userId"] for u in users)))
    log("Custom roles (%d): %s" % (len(roles), ", ".join(r["id"] for r in roles) or "-"))
    log("Custom privileges (%d): %s" % (len(privileges), ", ".join(p["name"] for p in privileges) or "-"))
    log("Content selectors (%d): %s" % (len(selectors), ", ".join(s["name"] for s in selectors) or "-"))
    log("Active realms: %s" % ", ".join(realms))
    log("Anonymous access: %s (user %s, realm %s)" % ("enabled" if anonymous.get("enabled") else "disabled", anonymous.get("userId"), anonymous.get("realmName")))
    if cleanup is None:
        log("Cleanup policies: API not available")
    else:
        log("Cleanup policies (%d): %s" % (len(cleanup), ", ".join(c["name"] for c in cleanup) or "-"))
    if routing is not None:
        log("Routing rules (%d): %s" % (len(routing), ", ".join(r["name"] for r in routing) or "-"))
    return 0


class ConfigReport:
    def __init__(self, dry_run):
        self.dry_run = dry_run
        self.counts = {}
        self.failures = 0
        self.warnings = []

    def add(self, kind, name, action, detail=""):
        if self.dry_run and action == "created":
            action = "would create"
        if self.dry_run and action == "updated":
            action = "would update"
        self.counts[action] = self.counts.get(action, 0) + 1
        if action == "failed":
            self.failures += 1
        log("  %-13s %-17s %s%s" % (action, kind, name, (" - " + detail) if detail else ""))

    def warn(self, message):
        self.warnings.append(message)
        log("  %-13s %s" % ("warning", message))


def ordered_groups(groups):
    pending = list(groups)
    names = {repo["name"] for repo in pending}
    ordered = []
    placed = set()
    while pending:
        progress = False
        for repo in list(pending):
            members = set(((repo.get("_body") or {}).get("group") or {}).get("memberNames") or [])
            if not (members & names) - placed:
                ordered.append(repo)
                placed.add(repo["name"])
                pending.remove(repo)
                progress = True
        if not progress:
            ordered.extend(pending)
            break
    return ordered


def ordered_roles(roles):
    pending = list(roles)
    ids = {role["id"] for role in pending}
    ordered = []
    placed = set()
    while pending:
        progress = False
        for role in list(pending):
            if not (set(role.get("roles") or []) & ids) - placed:
                ordered.append(role)
                placed.add(role["id"])
                pending.remove(role)
                progress = True
        if not progress:
            ordered.extend(pending)
            break
    return ordered


def prepare_repository(body, repo, args, target_state, report):
    for key in ("url", "format", "type"):
        body.pop(key, None)
    notes = []
    storage = body.get("storage") or {}
    store = storage.get("blobStoreName")
    if store and store not in target_state["blobStores"]:
        notes.append("blob store %s does not exist on the target" % store)
    cleanup = body.get("cleanup")
    if isinstance(cleanup, dict) and cleanup.get("policyNames"):
        keep = [name for name in cleanup["policyNames"] if name in target_state["cleanupPolicies"]]
        dropped = [name for name in cleanup["policyNames"] if name not in target_state["cleanupPolicies"]]
        cleanup["policyNames"] = keep
        if dropped:
            report.warn("repository %s: cleanup policies %s are not on the target and were not attached" % (repo["name"], ", ".join(dropped)))
    if repo["format"] == "docker":
        docker = body.get("docker")
        if not isinstance(docker, dict):
            docker = {}
            body["docker"] = docker
        docker["pathEnabled"] = True
        if not args.keep_docker_ports:
            for key in ("httpPort", "httpsPort"):
                if docker.get(key):
                    report.warn("repository %s: dropped docker %s %s, served at /v2/%s/ instead" % (repo["name"], key, docker[key], repo["name"]))
                    docker[key] = None
    if "routingRuleName" in body:
        body["routingRule"] = body.pop("routingRuleName")
    signing = body.get("aptSigning")
    if isinstance(signing, dict) and signing.get("keypair") and not signing.get("passphrase"):
        report.warn("repository %s: the source API does not export the passphrase of the apt signing key; if the key has one, set it on the target by hand" % repo["name"])
    client = body.get("httpClient")
    if isinstance(client, dict):
        authentication = client.get("authentication")
        if isinstance(authentication, dict) and authentication.get("type") in ("username", "ntlm") and not authentication.get("password"):
            client["authentication"] = None
            report.warn("repository %s: the source API does not export the remote password; set the %s credentials of %s on the target by hand"
                        % (repo["name"], authentication.get("type"), authentication.get("username")))
        elif isinstance(authentication, dict) and authentication.get("type") == "bearerToken" and not authentication.get("bearerToken"):
            client["authentication"] = None
            report.warn("repository %s: the source API does not export the remote bearer token; set it on the target by hand"
                        % repo["name"])
    return notes


def attach_cleanup(target, repo, existing, state, args, report):
    label = "%s %s" % (repo["format"], repo["type"])
    wanted = [p for p in ((repo.get("_body") or {}).get("cleanup") or {}).get("policyNames") or [] if p in state["cleanupPolicies"]]
    if not wanted:
        report.add("repository", repo["name"], "exists", label)
        return
    current_body = repository_settings(target, existing)
    if current_body is None:
        report.add("repository", repo["name"], "exists", label)
        return
    cleanup = current_body.get("cleanup") if isinstance(current_body.get("cleanup"), dict) else {}
    current = cleanup.get("policyNames") or []
    missing = [p for p in wanted if p not in current]
    if not missing:
        report.add("repository", repo["name"], "exists", label)
        return
    for key in ("url", "format", "type"):
        current_body.pop(key, None)
    if "routingRuleName" in current_body:
        current_body["routingRule"] = current_body.pop("routingRuleName")
    current_body["cleanup"] = {"policyNames": current + missing}
    path = "/service/rest/v1/repositories/%s/%s/%s" % (PATH_FORMATS.get(repo["format"], repo["format"]), repo["type"], quote_name(repo["name"]))
    try:
        if not args.dry_run:
            target.send_json("PUT", path, current_body)
        report.add("repository", repo["name"], "updated", "attached cleanup policies %s" % ", ".join(missing))
    except MigrationError as error:
        report.add("repository", repo["name"], "failed", "cannot attach cleanup policies: %s" % error)


def cmd_config(args):
    source = make_client(args, "source")
    target = make_client(args, "target")
    report = ConfigReport(args.dry_run)
    log("Configuration %s -> %s%s" % (source.base, target.base, " (dry run)" if args.dry_run else ""))

    log("")
    log("Blob stores")
    target_stores = {s["name"] for s in target.get_json("/service/rest/v1/blobstores")}
    for store in source.get_json("/service/rest/v1/blobstores"):
        name = store["name"]
        if name in target_stores:
            report.add("blob store", name, "exists")
            continue
        if (store.get("type") or "").lower() != "file":
            report.add("blob store", name, "failed", "type %s is not migrated, create it on the target by hand" % store.get("type"))
            continue
        detail = source.try_json("/service/rest/v1/blobstores/file/" + quote_name(name)) or {}
        path = detail.get("path") if args.keep_blob_store_paths and detail.get("path") else name
        payload = {"name": name, "path": path}
        if detail.get("softQuota"):
            payload["softQuota"] = detail["softQuota"]
        try:
            if not args.dry_run:
                target.send_json("POST", "/service/rest/v1/blobstores/file", payload)
            target_stores.add(name)
            report.add("blob store", name, "created", "path %s" % path)
        except MigrationError as error:
            report.add("blob store", name, "failed", str(error))

    log("")
    log("Content selectors")
    target_selectors = {s["name"] for s in target.get_json("/service/rest/v1/security/content-selectors")}
    for selector in source.get_json("/service/rest/v1/security/content-selectors"):
        name = selector["name"]
        if name in target_selectors:
            report.add("content selector", name, "exists")
            continue
        try:
            if not args.dry_run:
                target.send_json("POST", "/service/rest/v1/security/content-selectors",
                                 {"name": name, "description": selector.get("description") or "", "expression": selector["expression"]})
            target_selectors.add(name)
            report.add("content selector", name, "created")
        except MigrationError as error:
            report.add("content selector", name, "failed", str(error))

    log("")
    log("Cleanup policies")
    source_cleanup_path, source_cleanup = cleanup_api(source)
    target_cleanup_path, target_cleanup = cleanup_api(target)
    target_policies = {p["name"] for p in target_cleanup or []}
    if source_cleanup is None or target_cleanup is None:
        report.add("cleanup policies", "*", "skipped", "API not available on the %s" % ("source" if source_cleanup is None else "target"))
    else:
        for policy in source_cleanup:
            name = policy["name"]
            if name in target_policies:
                report.add("cleanup policy", name, "exists")
                continue
            payload = {k: v for k, v in policy.items() if k not in ("inUseCount", "repositories")}
            try:
                if not args.dry_run:
                    target.send_json("POST", target_cleanup_path, payload)
                target_policies.add(name)
                report.add("cleanup policy", name, "created")
            except MigrationError as error:
                report.add("cleanup policy", name, "failed", str(error))

    log("")
    log("Routing rules")
    source_rules = source.try_json("/service/rest/v1/routing-rules")
    target_rules = target.try_json("/service/rest/v1/routing-rules")
    target_rule_names = {r["name"] for r in target_rules or []}
    if source_rules is None or target_rules is None:
        report.add("routing rules", "*", "skipped", "API not available")
    else:
        for rule in source_rules:
            if rule["name"] in target_rule_names:
                report.add("routing rule", rule["name"], "exists")
                continue
            try:
                if not args.dry_run:
                    target.send_json("POST", "/service/rest/v1/routing-rules", rule)
                target_rule_names.add(rule["name"])
                report.add("routing rule", rule["name"], "created")
            except MigrationError as error:
                report.add("routing rule", rule["name"], "failed", str(error))

    log("")
    log("Repositories")
    target_repos = {r["name"]: r for r in list_repositories(target)}
    source_repos = list_repositories(source)
    state = {"blobStores": target_stores, "cleanupPolicies": target_policies}
    for repo in source_repos:
        repo["_body"] = repository_settings(source, repo)
    hosted = [r for r in source_repos if r["type"] == "hosted"]
    proxies = [r for r in source_repos if r["type"] == "proxy"]
    groups = ordered_groups([r for r in source_repos if r["type"] == "group"])
    others = [r for r in source_repos if r["type"] not in ("hosted", "proxy", "group")]
    for repo in hosted + proxies + groups + others:
        name = repo["name"]
        label = "%s %s" % (repo["format"], repo["type"])
        existing = target_repos.get(name)
        if existing:
            if existing["format"] != repo["format"] or existing["type"] != repo["type"]:
                report.add("repository", name, "failed", "exists on the target as %s %s, source is %s" % (existing["format"], existing["type"], label))
            else:
                attach_cleanup(target, repo, existing, state, args, report)
            continue
        body = repo.pop("_body", None)
        if body is None:
            report.add("repository", name, "failed", "cannot read its settings from the source")
            continue
        notes = prepare_repository(body, repo, args, state, report)
        if repo["type"] == "group":
            members = (body.get("group") or {}).get("memberNames") or []
            missing = [m for m in members if m not in target_repos]
            if missing:
                notes.append("members missing on the target: %s" % ", ".join(missing))
        if notes and not args.dry_run:
            report.add("repository", name, "failed", "; ".join(notes))
            continue
        path = "/service/rest/v1/repositories/%s/%s" % (PATH_FORMATS.get(repo["format"], repo["format"]), repo["type"])
        try:
            if not args.dry_run:
                target.send_json("POST", path, body)
            target_repos[name] = {"name": name, "format": repo["format"], "type": repo["type"]}
            report.add("repository", name, "created", label + ("" if not notes else " (" + "; ".join(notes) + ")"))
        except MigrationError as error:
            if isinstance(error, HttpError) and error.status in (404, 405):
                report.add("repository", name, "failed", "the target has no %s %s recipe (%s)" % (repo["format"], repo["type"], error))
            else:
                report.add("repository", name, "failed", str(error))

    log("")
    log("Privileges")
    source_privileges = source.get_json("/service/rest/v1/security/privileges")
    target_privileges_list = target.get_json("/service/rest/v1/security/privileges")
    target_privileges = {p["name"] for p in target_privileges_list}
    source_prefix = wildcard_prefix(source_privileges)
    target_prefix = wildcard_prefix(target_privileges_list)
    id_prefixes = (built_in_id_prefix(source_privileges), built_in_id_prefix(target_privileges_list))
    if args.dry_run:
        for privilege in source_privileges:
            if privilege.get("readOnly") and privilege.get("repository") in target_repos:
                target_privileges.add(renamed_id(privilege["name"], id_prefixes))
    for privilege in source_privileges:
        if privilege.get("readOnly"):
            continue
        name = privilege["name"]
        if name in target_privileges:
            report.add("privilege", name, "exists")
            continue
        payload = {k: v for k, v in privilege.items() if k not in ("readOnly", "type")}
        kind = privilege["type"]
        if kind == "wildcard" and source_prefix and target_prefix and source_prefix != target_prefix:
            pattern = payload.get("pattern") or ""
            if pattern.startswith(source_prefix + ":"):
                payload["pattern"] = target_prefix + pattern[len(source_prefix):]
        try:
            if not args.dry_run:
                target.send_json("POST", "/service/rest/v1/security/privileges/" + kind, payload)
            target_privileges.add(name)
            report.add("privilege", name, "created", kind)
        except MigrationError as error:
            report.add("privilege", name, "failed", str(error))

    log("")
    log("Roles")
    target_roles = {r["id"] for r in target.get_json("/service/rest/v1/security/roles")}
    source_roles = [r for r in source.get_json("/service/rest/v1/security/roles?source=default") if not r.get("readOnly")]
    for role in ordered_roles(source_roles):
        if role["id"] in target_roles:
            report.add("role", role["id"], "exists")
            continue
        role_privileges = [target_id(p, target_privileges, id_prefixes) for p in role.get("privileges") or []]
        role_roles = [target_id(r, target_roles, id_prefixes) for r in role.get("roles") or []]
        privileges = [p for p in role_privileges if p in target_privileges]
        dropped = [p for p in role_privileges if p not in target_privileges]
        roles = [r for r in role_roles if r in target_roles]
        dropped_roles = [r for r in role_roles if r not in target_roles]
        if dropped or dropped_roles:
            report.warn("role %s: not on the target, left out: %s" % (role["id"], ", ".join(dropped + dropped_roles)))
        payload = {"id": role["id"], "name": role.get("name") or role["id"], "description": role.get("description") or "",
                   "privileges": privileges, "roles": roles}
        try:
            if not args.dry_run:
                target.send_json("POST", "/service/rest/v1/security/roles", payload)
            target_roles.add(role["id"])
            report.add("role", role["id"], "created")
        except MigrationError as error:
            report.add("role", role["id"], "failed", str(error))

    log("")
    log("Users")
    target_users = {u["userId"] for u in target.get_json("/service/rest/v1/security/users?source=default")}
    source_users = [u for u in source.get_json("/service/rest/v1/security/users?source=default") if u["userId"] not in SKIPPED_USERS]
    to_create = [u for u in source_users if u["userId"] not in target_users]
    if to_create and not args.dry_run and not args.new_passwords_file:
        raise MigrationError("%d users need to be created; pass --new-passwords-file to receive their new passwords" % len(to_create))
    password_fd = None
    try:
        for user in source_users:
            user_id = user["userId"]
            if user_id in target_users:
                report.add("user", user_id, "exists")
                continue
            user_roles = [target_id(r, target_roles, id_prefixes) for r in user.get("roles") or []]
            roles = [r for r in user_roles if r in target_roles]
            dropped = [r for r in user_roles if r not in target_roles]
            if dropped:
                report.warn("user %s: roles not on the target were left out: %s" % (user_id, ", ".join(dropped)))
            if args.dry_run:
                report.add("user", user_id, "created", "roles %s" % ", ".join(roles))
                continue
            password = secrets.token_urlsafe(18)
            payload = {"userId": user_id, "firstName": user.get("firstName") or user_id, "lastName": user.get("lastName") or user_id,
                       "emailAddress": user.get("emailAddress") or "%s@example.org" % user_id, "password": password,
                       "status": user.get("status") or "active", "roles": roles}
            try:
                target.send_json("POST", "/service/rest/v1/security/users", payload)
            except MigrationError as error:
                report.add("user", user_id, "failed", str(error))
                continue
            if password_fd is None:
                password_fd = os.open(args.new_passwords_file, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
                os.fchmod(password_fd, 0o600)
            os.write(password_fd, ("%s\t%s\n" % (user_id, password)).encode("utf-8"))
            report.add("user", user_id, "created", "roles %s" % ", ".join(roles))
    finally:
        if password_fd is not None:
            os.close(password_fd)
            log("  new passwords appended to %s" % args.new_passwords_file)

    log("")
    log("Anonymous access")
    target_realms_available = {r["id"] for r in target.get_json("/service/rest/v1/security/realms/available")}
    anonymous = source.get_json("/service/rest/v1/security/anonymous")
    current = target.get_json("/service/rest/v1/security/anonymous")
    desired = {"enabled": bool(anonymous.get("enabled")), "userId": anonymous.get("userId") or "anonymous",
               "realmName": anonymous.get("realmName")}
    realm = desired["realmName"] or ""
    if not realm:
        desired["realmName"] = current.get("realmName")
    if all(current.get(k) == v for k, v in desired.items()):
        report.add("anonymous", "settings", "exists", "enabled" if desired["enabled"] else "disabled")
    else:
        try:
            if not args.dry_run:
                target.send_json("PUT", "/service/rest/v1/security/anonymous", desired)
            report.add("anonymous", "settings", "updated", "%s, user %s, realm %s" % ("enabled" if desired["enabled"] else "disabled", desired["userId"], desired["realmName"]))
        except MigrationError as error:
            report.add("anonymous", "settings", "failed", str(error))

    log("")
    log("Realms")
    source_active = source.get_json("/service/rest/v1/security/realms/active")
    target_active = target.get_json("/service/rest/v1/security/realms/active")
    wanted = []
    for name in source_active:
        mapped = map_name(name, target_realms_available)
        if mapped is None:
            report.add("realm", name, "dropped", "not available on the target")
        elif mapped not in wanted:
            wanted.append(mapped)
            if mapped != name:
                log("  %-13s %-17s %s -> %s" % ("mapped", "realm", name, mapped))
    for name in target_active:
        if name not in wanted:
            wanted.append(name)
            log("  %-13s %-17s %s - active on the target only" % ("kept", "realm", name))
    if wanted == target_active:
        report.add("realms", ", ".join(wanted), "exists")
    else:
        try:
            if not args.dry_run:
                target.send_json("PUT", "/service/rest/v1/security/realms/active", wanted)
            report.add("realms", ", ".join(wanted), "updated")
        except MigrationError as error:
            report.add("realms", ", ".join(wanted), "failed", str(error))

    log("")
    log("Summary: " + ", ".join("%s %d" % (k, v) for k, v in sorted(report.counts.items())))
    if report.warnings:
        log("Warnings to act on:")
        for warning in report.warnings:
            log("  - " + warning)
    return 1 if report.failures else 0


class RepoResult:
    def __init__(self, name, fmt):
        self.name = name
        self.format = fmt
        self.copied = 0
        self.skipped = 0
        self.planned = 0
        self.failed = 0
        self.bytes = 0
        self.notes = []
        self.handled = True
        self.source_counts = None
        self.target_counts = None
        self.lock = threading.Lock()

    def add(self, field, amount=1):
        with self.lock:
            setattr(self, field, getattr(self, field) + amount)

    def fail(self, message):
        with self.lock:
            self.failed += 1
        log("  [%s] FAILED %s" % (self.name, message))

    def note(self, message):
        with self.lock:
            self.notes.append(message)
        log("  [%s] %s" % (self.name, message))


class ContentContext:
    def __init__(self, args, source, target):
        self.args = args
        self.source = source
        self.target = target
        self.workers = max(1, args.workers)
        self.temp_dir = args.temp_dir
        self.dry_run = args.dry_run
        self.verbose = args.verbose

    def source_path(self, repo, asset):
        marker = "/repository/%s/" % repo
        path = asset["path"].lstrip("/")
        url = asset.get("downloadUrl") or ""
        index = url.find(marker)
        if index >= 0:
            suffix = url[index + len(marker):]
            if suffix != path and urllib.parse.unquote(suffix) != path:
                path = urllib.parse.unquote(suffix)
        return marker + urllib.parse.quote(path, safe="/")


def run_parallel(ctx, result, items, action, label):
    if not items:
        return
    done = [0]
    started = time.time()
    last = [started]

    def guarded(item):
        try:
            action(item)
        except Exception as error:
            result.fail("%s: %s" % (label(item), error))
        with result.lock:
            done[0] += 1
            count = done[0]
        now = time.time()
        if count == len(items) or now - last[0] > 30:
            last[0] = now
            log("  [%s] %d/%d processed" % (result.name, count, len(items)))

    with concurrent.futures.ThreadPoolExecutor(max_workers=ctx.workers) as pool:
        list(pool.map(guarded, items))


def same_content(source_asset, target_asset):
    source_sha1 = (source_asset.get("checksum") or {}).get("sha1")
    target_sha1 = (target_asset.get("checksum") or {}).get("sha1")
    if source_sha1 and target_sha1:
        return source_sha1 == target_sha1
    return True


def verify_sha1(asset, downloaded):
    expected = (asset.get("checksum") or {}).get("sha1")
    if expected and expected != downloaded.sha1:
        raise MigrationError("sha1 mismatch: source API says %s, downloaded %s" % (expected, downloaded.sha1))


def expect(response, ok, method):
    if response.status not in ok:
        raise HttpError(method, response.url, response.status, response.text())
    response.close()


def is_maven_metadata(path):
    return basename(path).startswith("maven-metadata.xml")


def is_derived(path):
    name = basename(path)
    return is_maven_metadata(name) or name.startswith("archetype-catalog.xml") or name.endswith(CHECKSUMS)


def copy_by_path(ctx, repo, result, source_assets):
    target_assets = {a["path"].lstrip("/"): a for a in list_assets(ctx.target, repo)}
    main, metadata = [], []
    for asset in source_assets:
        existing = target_assets.get(asset["path"].lstrip("/"))
        if existing is not None and same_content(asset, existing):
            result.add("skipped")
            continue
        if existing is not None:
            result.note("%s differs on the target and will be uploaded again" % asset["path"])
        if existing is None and result.format == "maven2" and is_derived(asset["path"]) \
                and served(ctx.target, ctx.source_path(repo, asset)):
            result.add("skipped")
            continue
        if result.format == "maven2" and is_maven_metadata(asset["path"]):
            metadata.append(asset)
        else:
            main.append(asset)
    if ctx.dry_run:
        result.add("planned", len(main) + len(metadata))
        return

    def put(asset):
        downloaded = download(ctx.source, ctx.source_path(repo, asset), ctx.temp_dir)
        try:
            verify_sha1(asset, downloaded)
            content_type = asset.get("contentType") or downloaded.content_type or "application/octet-stream"
            response = ctx.target.open("PUT", ctx.source_path(repo, asset), body=FileBody(downloaded.path),
                                       headers={"Content-Type": content_type})
            expect(response, (200, 201, 204), "PUT")
            result.add("copied")
            result.add("bytes", downloaded.size)
            if ctx.verbose:
                log("  [%s] copied %s" % (repo, asset["path"]))
        finally:
            downloaded.remove()

    run_parallel(ctx, result, main, put, lambda a: a["path"])
    run_parallel(ctx, result, metadata, put, lambda a: a["path"])
    if result.format == "maven2" and maven_rebuild_if_needed(ctx, repo, result, source_assets):
        rebuilt = {a["path"].lstrip("/"): a for a in list_assets(ctx.target, repo)}
        restore = [a for a in source_assets if is_maven_metadata(a["path"])
                   and not same_content(a, rebuilt.get(a["path"].lstrip("/"), {"checksum": {"sha1": "-"}}))]
        if restore:
            copied = result.copied
            run_parallel(ctx, result, restore, put, lambda a: a["path"])
            result.copied = copied
            result.note("restored %d source metadata files that the rebuild had regenerated" % len(restore))


def served(client, path):
    response = client.open("GET", path, retries=1)
    status = response.status
    response.close()
    return status == 200


def maven_rebuild_if_needed(ctx, repo, result, source_assets):
    paths = {a["path"].lstrip("/") for a in list_assets(ctx.target, repo)}
    artifacts = set()
    for asset in source_assets:
        parts = asset["path"].lstrip("/").split("/")
        if len(parts) >= 4 and parts[-1].startswith(parts[-3] + "-") and not is_maven_metadata(parts[-1]):
            artifacts.add("/".join(parts[:-2]))
    missing = sorted(a for a in artifacts if a + "/maven-metadata.xml" not in paths)
    missing = [a for a in missing if not served(ctx.target, "/repository/%s/%s/maven-metadata.xml" % (repo, a))]
    if not missing:
        return False
    result.note("%d artifacts have no maven-metadata.xml on the target (e.g. %s); rebuilding Maven metadata" % (len(missing), missing[0]))
    name = "repository-copy: rebuild Maven metadata of %s" % repo
    tasks = ctx.target.get_json("/service/rest/v1/tasks?type=" + MAVEN_REBUILD_TASK).get("items") or []
    task = next((t for t in tasks if t.get("name") == name), None)
    if task is None:
        request = {"action": "coreui_Task", "method": "create", "type": "rpc", "tid": 1, "data": [{
            "id": "NX.coreui.model.Task-1", "enabled": True, "name": name, "typeId": MAVEN_REBUILD_TASK,
            "notificationCondition": "FAILURE", "schedule": "manual", "timeZoneOffset": "+00:00",
            "properties": {"repositoryName": repo, "groupId": "", "artifactId": "", "baseVersion": "",
                           "rebuildChecksums": "true", "cascadeRebuild": "true"}}]}
        reply = json.loads(ctx.target.send_json("POST", "/service/extdirect", request) or b"{}")
        if not (reply.get("result") or {}).get("success"):
            result.fail("cannot create the Maven metadata rebuild task: %s" % json.dumps(reply.get("result") or reply)[:300])
            return False
        tasks = ctx.target.get_json("/service/rest/v1/tasks?type=" + MAVEN_REBUILD_TASK).get("items") or []
        task = next((t for t in tasks if t.get("name") == name), None)
        if task is None:
            result.fail("the Maven metadata rebuild task was created but cannot be found")
            return False
    previous = ctx.target.get_json("/service/rest/v1/tasks/" + task["id"]).get("lastRun")
    ctx.target.send_json("POST", "/service/rest/v1/tasks/%s/run" % task["id"], None)
    for _ in range(120):
        time.sleep(2)
        state = ctx.target.get_json("/service/rest/v1/tasks/" + task["id"])
        if state.get("currentState") == "WAITING" and state.get("lastRun") != previous and state.get("lastRunResult"):
            if state["lastRunResult"] != "OK":
                result.fail("Maven metadata rebuild ended with %s" % state["lastRunResult"])
                return False
            result.note("Maven metadata rebuild finished")
            return True
    result.note("Maven metadata rebuild is still running (task %s)" % task["id"])
    return False


def npm_path(repo, name):
    if name.startswith("@"):
        return "/repository/%s/%s" % (repo, urllib.parse.quote(name, safe="@").replace("/", "%2f"))
    return "/repository/%s/%s" % (repo, urllib.parse.quote(name, safe=""))


def copy_npm(ctx, repo, result, source_assets):
    by_path = {a["path"].lstrip("/"): a for a in source_assets}
    packages = []
    for path in by_path:
        if "/-/" in path and path.endswith(".tgz"):
            name = path.split("/-/", 1)[0]
            if name not in packages:
                packages.append(name)
    marker = "@@repository-copy-%s@@" % secrets.token_hex(8)

    def copy_package(name):
        document = ctx.source.get_json(npm_path(repo, name))
        current = ctx.target.open("GET", npm_path(repo, name), headers={"Accept": "application/json"})
        if current.status == 200:
            target_versions = set((current.json() or {}).get("versions") or {})
        else:
            current.close()
            target_versions = set()
        times = document.get("time") or {}
        versions = document.get("versions") or {}
        tags = document.get("dist-tags") or {}
        for version in sorted(versions, key=lambda v: (times.get(v) or "", v)):
            manifest = versions[version]
            tarball = basename(((manifest.get("dist") or {}).get("tarball")) or "%s-%s.tgz" % (name.split("/")[-1], version))
            asset = by_path.get("%s/-/%s" % (name, tarball))
            if asset is None:
                result.note("%s@%s has no tarball in the source repository, skipped" % (name, version))
                continue
            if version in target_versions:
                result.add("skipped")
                continue
            if ctx.dry_run:
                result.add("planned")
                continue
            downloaded = None
            try:
                downloaded = download(ctx.source, ctx.source_path(repo, asset), ctx.temp_dir)
                shasum = (manifest.get("dist") or {}).get("shasum")
                if shasum and shasum != downloaded.sha1:
                    raise MigrationError("%s@%s: tarball sha1 %s does not match dist.shasum %s" % (name, version, downloaded.sha1, shasum))
                publish = {
                    "_id": name,
                    "name": name,
                    "description": document.get("description") or manifest.get("description") or "",
                    "dist-tags": {tag: value for tag, value in tags.items() if value == version} or {TEMPORARY_TAG: version},
                    "versions": {version: manifest},
                    "readme": manifest.get("readme") or document.get("readme") or "",
                    "_attachments": {tarball: {"content_type": "application/octet-stream", "data": marker, "length": downloaded.size}},
                }
                original_times = {key: times[key] for key in ("created", version) if times.get(key)}
                if original_times:
                    publish["time"] = original_times
                text = json.dumps(publish)
                head, tail = text.split('"%s"' % marker, 1)
                body = CompositeBody([(head + '"').encode("utf-8"), Base64FileBody(downloaded.path), ('"' + tail).encode("utf-8")])
                response = ctx.target.open("PUT", npm_path(repo, name), body=body, headers={"Content-Type": "application/json"})
                expect(response, (200, 201), "PUT")
                if TEMPORARY_TAG in publish["dist-tags"]:
                    removal = ctx.target.open("DELETE", "/repository/%s/-/package/%s/dist-tags/%s" % (repo, urllib.parse.quote(name, safe="@/"), TEMPORARY_TAG))
                    removal.close()
                    if removal.status not in (200, 201, 204):
                        result.note("%s: cannot remove the temporary dist-tag %s (HTTP %s)" % (name, TEMPORARY_TAG, removal.status))
                result.add("copied")
                result.add("bytes", downloaded.size)
                if ctx.verbose:
                    log("  [%s] published %s@%s" % (repo, name, version))
            except MigrationError as error:
                result.fail("%s@%s: %s" % (name, version, error))
            finally:
                if downloaded is not None:
                    downloaded.remove()
        if not ctx.dry_run:
            final = ctx.target.try_json(npm_path(repo, name)) or {}
            target_tags = final.get("dist-tags") or {}
            for tag, value in tags.items():
                if target_tags.get(tag) != value and value in (final.get("versions") or {}):
                    result.note("%s: dist-tag %s is %s on the target, %s on the source" % (name, tag, target_tags.get(tag), value))

    run_parallel(ctx, result, packages, copy_package, lambda n: n)


def nuget_version(version):
    value = version.split("+", 1)[0].lower()
    head, dash, tail = value.partition("-")
    parts = head.split(".")
    if all(p.isdigit() for p in parts):
        numbers = [str(int(p)) for p in parts]
        while len(numbers) < 3:
            numbers.append("0")
        if len(numbers) == 4 and numbers[3] == "0":
            numbers = numbers[:3]
        head = ".".join(numbers)
    return head + dash + tail


def pick_package_asset(component, suffixes):
    assets = component.get("assets") or []
    for asset in assets:
        if asset["path"].lower().endswith(suffixes):
            return asset
    for asset in assets:
        if (asset.get("contentType") or "") in ("application/zip", "application/octet-stream") and not asset["path"].lower().endswith(".nuspec"):
            return asset
    return assets[0] if len(assets) == 1 else None


def copy_nuget(ctx, repo, result, source_assets):
    target_keys = {(c["name"].lower(), nuget_version(c.get("version") or "")) for c in list_components(ctx.target, repo)}
    plan = []
    for component in list_components(ctx.source, repo):
        key = (component["name"].lower(), nuget_version(component.get("version") or ""))
        if key in target_keys:
            result.add("skipped")
            continue
        asset = pick_package_asset(component, (".nupkg",))
        if asset is None:
            result.fail("%s %s: no .nupkg asset found" % (component["name"], component.get("version")))
            continue
        plan.append((component, asset))
    if ctx.dry_run:
        result.add("planned", len(plan))
        return

    def push(item):
        component, asset = item
        downloaded = download(ctx.source, ctx.source_path(repo, asset), ctx.temp_dir)
        try:
            verify_sha1(asset, downloaded)
            filename = "%s.%s.nupkg" % (component["name"], component.get("version"))
            body, content_type = multipart([], [("package", filename, FileBody(downloaded.path))])
            response = ctx.target.open("PUT", "/repository/%s/" % repo, body=body, headers={"Content-Type": content_type})
            if response.status == 409:
                response.close()
                result.add("skipped")
                return
            expect(response, (200, 201, 202, 204), "PUT")
            result.add("copied")
            result.add("bytes", downloaded.size)
            if ctx.verbose:
                log("  [%s] pushed %s %s" % (repo, component["name"], component.get("version")))
        finally:
            downloaded.remove()

    run_parallel(ctx, result, plan, push, lambda item: "%s %s" % (item[0]["name"], item[0].get("version")))


def copy_helm(ctx, repo, result, source_assets):
    target_names = {basename(a["path"]) for a in list_assets(ctx.target, repo)}
    charts, provenance = [], []
    for asset in source_assets:
        name = basename(asset["path"])
        if not (name.endswith(".tgz") or name.endswith(".tgz.prov")):
            continue
        if name in target_names:
            result.add("skipped")
            continue
        (provenance if name.endswith(".prov") else charts).append(asset)
    if ctx.dry_run:
        result.add("planned", len(charts) + len(provenance))
        return

    def upload(asset):
        downloaded = download(ctx.source, ctx.source_path(repo, asset), ctx.temp_dir)
        try:
            verify_sha1(asset, downloaded)
            path = "/repository/%s/%s" % (repo, urllib.parse.quote(basename(asset["path"])))
            response = ctx.target.open("PUT", path, body=FileBody(downloaded.path), headers={"Content-Type": "application/x-compressed"})
            expect(response, (200, 201, 204), "PUT")
            result.add("copied")
            result.add("bytes", downloaded.size)
            if ctx.verbose:
                log("  [%s] uploaded %s" % (repo, asset["path"]))
        finally:
            downloaded.remove()

    run_parallel(ctx, result, charts, upload, lambda a: a["path"])
    run_parallel(ctx, result, provenance, upload, lambda a: a["path"])


PYPI_MULTI = {
    "classifier": "classifiers",
    "provides-extra": "provides_extras",
    "home-page": "home_page",
}


def python_metadata(path, filename):
    lower = filename.lower()
    raw = None
    try:
        if lower.endswith((".whl", ".egg")):
            with zipfile.ZipFile(path) as archive:
                for member in archive.namelist():
                    if member.endswith(".dist-info/METADATA") or member == "EGG-INFO/PKG-INFO":
                        raw = archive.read(member)
                        break
        elif lower.endswith(".zip"):
            with zipfile.ZipFile(path) as archive:
                for member in archive.namelist():
                    if member.count("/") == 1 and member.endswith("/PKG-INFO"):
                        raw = archive.read(member)
                        break
        elif lower.endswith((".tar.gz", ".tgz", ".tar.bz2", ".tar.xz", ".tar")):
            with tarfile.open(path) as archive:
                for member in archive:
                    if member.isfile() and member.name.count("/") == 1 and member.name.endswith("/PKG-INFO"):
                        raw = archive.extractfile(member).read()
                        break
    except (zipfile.BadZipFile, tarfile.TarError, OSError):
        return None
    if raw is None:
        return None
    message = email.parser.Parser().parsestr(raw.decode("utf-8", "replace"))
    fields = []
    for key in dict.fromkeys(k.lower() for k in message.keys()):
        name = PYPI_MULTI.get(key, key.replace("-", "_"))
        if name == "description":
            continue
        for value in message.get_all(key) or []:
            fields.append((name, str(value)))
    description = message.get_payload()
    if isinstance(description, str) and description.strip():
        fields.append(("description", description))
    elif message.get("Description"):
        fields.append(("description", str(message.get("Description"))))
    return fields


def python_filetype(filename):
    lower = filename.lower()
    if lower.endswith(".whl"):
        parts = filename[:-4].split("-")
        return "bdist_wheel", parts[-3] if len(parts) >= 5 else "any"
    if lower.endswith(".egg"):
        match = re.search(r"-py(\d+(?:\.\d+)?)", filename)
        return "bdist_egg", match.group(1) if match else "any"
    if lower.endswith((".tar.gz", ".tgz", ".zip", ".tar.bz2", ".tar.xz", ".tar")):
        return "sdist", "source"
    return "bdist_dumb", "any"


def copy_pypi(ctx, repo, result, source_assets):
    target_names = {basename(a["path"]) for a in list_assets(ctx.target, repo)}
    signatures = {a["path"]: a for a in source_assets if a["path"].endswith(".asc")}
    plan = []
    for asset in source_assets:
        path = asset["path"].lstrip("/")
        if not path.startswith("packages/") or path.endswith(".asc"):
            continue
        if basename(path) in target_names:
            result.add("skipped")
            continue
        plan.append(asset)
    if ctx.dry_run:
        result.add("planned", len(plan))
        return

    def upload(asset):
        path = asset["path"].lstrip("/")
        filename = basename(path)
        downloaded = download(ctx.source, ctx.source_path(repo, asset), ctx.temp_dir)
        signature = None
        try:
            verify_sha1(asset, downloaded)
            fields = python_metadata(downloaded.path, filename)
            if fields is None:
                parts = path.split("/")
                if len(parts) < 4:
                    raise MigrationError("cannot read package metadata from %s" % filename)
                fields = [("name", parts[1]), ("version", parts[2]), ("metadata_version", "1.0")]
                result.note("%s: no PKG-INFO/METADATA inside, uploaded with name and version from its path" % filename)
            filetype, pyversion = python_filetype(filename)
            fields = [(":action", "file_upload"), ("protocol_version", "1")] + fields + [
                ("filetype", filetype), ("pyversion", pyversion),
                ("md5_digest", downloaded.md5), ("sha256_digest", downloaded.sha256)]
            files = [("content", filename, FileBody(downloaded.path))]
            signature_asset = signatures.get(asset["path"] + ".asc")
            if signature_asset is not None:
                signature = download(ctx.source, ctx.source_path(repo, signature_asset), ctx.temp_dir)
                files.append(("gpg_signature", filename + ".asc", FileBody(signature.path)))
            body, content_type = multipart(fields, files)
            response = ctx.target.open("POST", "/repository/%s/" % repo, body=body, headers={"Content-Type": content_type})
            expect(response, (200, 201, 204), "POST")
            result.add("copied")
            result.add("bytes", downloaded.size)
            if ctx.verbose:
                log("  [%s] uploaded %s" % (repo, filename))
        finally:
            downloaded.remove()
            if signature is not None:
                signature.remove()

    run_parallel(ctx, result, plan, upload, lambda a: a["path"])


def copy_apt(ctx, repo, result, source_assets):
    target_names = {basename(a["path"]) for a in list_assets(ctx.target, repo)}
    plan = []
    for asset in source_assets:
        name = basename(asset["path"])
        if not name.endswith(".deb"):
            continue
        if name in target_names:
            result.add("skipped")
            continue
        plan.append(asset)
    if ctx.dry_run:
        result.add("planned", len(plan))
        return

    def upload(asset):
        downloaded = download(ctx.source, ctx.source_path(repo, asset), ctx.temp_dir)
        try:
            verify_sha1(asset, downloaded)
            response = ctx.target.open("POST", "/repository/%s/" % repo, body=FileBody(downloaded.path),
                                       headers={"Content-Type": "application/vnd.debian.binary-package"})
            expect(response, (200, 201, 204), "POST")
            result.add("copied")
            result.add("bytes", downloaded.size)
            if ctx.verbose:
                log("  [%s] uploaded %s" % (repo, asset["path"]))
        finally:
            downloaded.remove()

    run_parallel(ctx, result, plan, upload, lambda a: a["path"])


class DockerCopier:
    def __init__(self, ctx, repo, result):
        self.ctx = ctx
        self.repo = repo
        self.result = result
        self.source_base = "/repository/%s/v2" % repo
        self.target_base = "/v2/%s" % repo
        self.blob_locks = {}
        self.done_blobs = set()
        self.lock = threading.Lock()

    def check_target(self):
        response = self.ctx.target.open("GET", self.target_base + "/", token_key="target:" + self.repo)
        status = response.status
        detail = response.text(200)
        if status != 200:
            raise MigrationError("the target does not serve %s at /v2/%s/ (HTTP %s %s); enable path-based routing (docker.pathEnabled)"
                                 % (self.repo, self.repo, status, detail))

    def fetch_manifest(self, image, reference, accept=MANIFEST_ACCEPT):
        response = self.ctx.source.open("GET", "%s/%s/manifests/%s" % (self.source_base, image, reference),
                                        headers={"Accept": accept}, token_key="source:%s/%s" % (self.repo, image))
        if response.status != 200:
            raise HttpError("GET", response.url, response.status, response.text(200))
        body = response.read()
        digest = "sha256:" + hashlib.sha256(body).hexdigest()
        announced = response.headers.get("Docker-Content-Digest")
        if announced and announced.startswith("sha256:") and announced != digest:
            raise MigrationError("manifest %s:%s digest %s does not match its content %s" % (image, reference, announced, digest))
        content_type = (response.headers.get("Content-Type") or "").split(";")[0].strip()
        try:
            document = json.loads(body)
        except ValueError:
            raise MigrationError("manifest %s:%s is not JSON" % (image, reference))
        media_type = document.get("mediaType") or content_type
        if not media_type:
            media_type = "application/vnd.docker.distribution.manifest.v1+prettyjws" if document.get("schemaVersion") == 1 else "application/vnd.oci.image.manifest.v1+json"
        return body, media_type, digest, document

    def target_digest(self, image, reference):
        response = self.ctx.target.open("HEAD", "%s/%s/manifests/%s" % (self.target_base, image, reference),
                                        headers={"Accept": MANIFEST_ACCEPT}, token_key="target:%s/%s" % (self.repo, image))
        response.close()
        if response.status != 200:
            return None
        return response.headers.get("Docker-Content-Digest")

    def copy_tag(self, image, tag):
        body, media_type, digest, document = self.fetch_manifest(image, tag)
        if self.target_digest(image, tag) == digest:
            self.result.add("skipped")
            return
        if self.ctx.dry_run:
            self.result.add("planned")
            return
        self.copy_references(image, document)
        self.put_manifest(image, tag, body, media_type)
        self.result.add("copied")
        if self.ctx.verbose:
            log("  [%s] copied %s:%s (%s)" % (self.repo, image, tag, digest))

    def copy_references(self, image, document):
        if document.get("manifests"):
            for child in document["manifests"]:
                digest = child["digest"]
                if self.target_digest(image, digest) == digest:
                    continue
                accept = child.get("mediaType") or MANIFEST_ACCEPT
                body, media_type, actual, child_document = self.fetch_manifest(image, digest, accept)
                if actual != digest:
                    raise MigrationError("child manifest %s of %s came back as %s" % (digest, image, actual))
                self.copy_references(image, child_document)
                self.put_manifest(image, digest, body, media_type)
            return
        blobs = []
        if isinstance(document.get("config"), dict):
            blobs.append(document["config"])
        blobs.extend(document.get("layers") or [])
        blobs.extend({"digest": layer["blobSum"]} for layer in document.get("fsLayers") or [])
        for blob in blobs:
            if blob.get("urls") or "foreign" in (blob.get("mediaType") or ""):
                continue
            self.copy_blob(image, blob["digest"])

    def copy_blob(self, image, digest):
        with self.lock:
            lock = self.blob_locks.setdefault(digest, threading.Lock())
        with lock:
            if digest in self.done_blobs:
                return
            target_key = "target:%s/%s" % (self.repo, image)
            head = self.ctx.target.open("HEAD", "%s/%s/blobs/%s" % (self.target_base, image, digest), token_key=target_key)
            head.close()
            if head.status == 200:
                with self.lock:
                    self.done_blobs.add(digest)
                return
            downloaded = download(self.ctx.source, "%s/%s/blobs/%s" % (self.source_base, image, digest), self.ctx.temp_dir,
                                  token_key="source:%s/%s" % (self.repo, image))
            try:
                if digest.startswith("sha256:") and digest != "sha256:" + downloaded.sha256:
                    raise MigrationError("blob %s downloaded with digest sha256:%s" % (digest, downloaded.sha256))
                start = self.ctx.target.open("POST", "%s/%s/blobs/uploads/" % (self.target_base, image), body=b"",
                                             headers={"Content-Length": "0"}, token_key=target_key)
                if start.status != 202 or not start.headers.get("Location"):
                    raise HttpError("POST", start.url, start.status, start.text(200))
                start.close()
                location = urllib.parse.urljoin(self.ctx.target.base + "/", start.headers["Location"])
                url = location + ("&" if "?" in location else "?") + "digest=" + urllib.parse.quote(digest)
                response = self.ctx.target.open("PUT", url, body=FileBody(downloaded.path),
                                                headers={"Content-Type": "application/octet-stream"}, token_key=target_key)
                expect(response, (201, 204), "PUT")
                self.result.add("bytes", downloaded.size)
                with self.lock:
                    self.done_blobs.add(digest)
            finally:
                downloaded.remove()

    def put_manifest(self, image, reference, body, media_type):
        response = self.ctx.target.open("PUT", "%s/%s/manifests/%s" % (self.target_base, image, reference), body=body,
                                        headers={"Content-Type": media_type}, token_key="target:%s/%s" % (self.repo, image))
        expect(response, (200, 201, 202), "PUT")


def copy_docker(ctx, repo, result, source_assets):
    copier = DockerCopier(ctx, repo, result)
    tags = []
    for component in list_components(ctx.source, repo):
        if component.get("name") and component.get("version"):
            tags.append((component["name"], component["version"]))
    if not tags:
        return
    copier.check_target()
    run_parallel(ctx, result, tags, lambda item: copier.copy_tag(*item), lambda item: "%s:%s" % item)


HANDLERS = {
    "maven2": copy_by_path,
    "raw": copy_by_path,
    "npm": copy_npm,
    "nuget": copy_nuget,
    "helm": copy_helm,
    "pypi": copy_pypi,
    "docker": copy_docker,
    "apt": copy_apt,
}


def counts(client, repo, assets=None):
    if assets is None:
        assets = list_assets(client, repo)
    components = sum(1 for _ in client.paginate("/service/rest/v1/components", repository=repo))
    return components, len(assets), sum(a.get("fileSize") or 0 for a in assets)


def cmd_content(args):
    source = make_client(args, "source")
    target = make_client(args, "target")
    ctx = ContentContext(args, source, target)
    if args.temp_dir and not os.path.isdir(args.temp_dir):
        raise MigrationError("--temp-dir %s does not exist" % args.temp_dir)
    source_repos = [r for r in list_repositories(source) if r["type"] == "hosted"]
    if args.repository:
        known = {r["name"] for r in source_repos}
        unknown = [n for n in args.repository if n not in known]
        if unknown:
            raise MigrationError("not hosted repositories on the source: %s" % ", ".join(unknown))
        source_repos = [r for r in source_repos if r["name"] in args.repository]
    target_repos = {r["name"]: r for r in list_repositories(target)}
    log("Content %s -> %s%s, %d workers" % (source.base, target.base, " (dry run)" if args.dry_run else "", ctx.workers))
    results = []
    for repo in source_repos:
        name = repo["name"]
        result = RepoResult(name, repo["format"])
        results.append(result)
        log("")
        log("Repository %s (%s)" % (name, repo["format"]))
        try:
            source_assets = list_assets(source, name)
            result.source_counts = counts(source, name, source_assets)
            handler = HANDLERS.get(repo["format"])
            if handler is None:
                result.handled = False
                result.note("format %s is not handled: %d components, %d assets, %s left on the source"
                            % (repo["format"], result.source_counts[0], result.source_counts[1], human(result.source_counts[2])))
                continue
            existing = target_repos.get(name)
            if existing is None:
                result.fail("the repository does not exist on the target, run config first")
                continue
            if existing["format"] != repo["format"] or existing["type"] != "hosted":
                result.fail("the target repository is %s %s" % (existing["format"], existing["type"]))
                continue
            handler(ctx, name, result, source_assets)
            result.target_counts = counts(target, name)
        except MigrationError as error:
            result.fail(str(error))
        log("  [%s] copied %d, skipped %d, %s%sfailed %d" % (
            name, result.copied, result.skipped, ("to copy %d, " % result.planned) if args.dry_run else "",
            ("%s transferred, " % human(result.bytes)) if result.bytes else "", result.failed))

    log("")
    log("Summary")
    log("  %-30s %-7s %13s %13s %8s %8s %8s %8s" % ("repository", "format", "components", "assets", "copied", "skipped", "to copy" if args.dry_run else "", "failed"))
    for result in results:
        source_counts = result.source_counts or ("?", "?", 0)
        target_counts = result.target_counts or ("?", "?", 0)
        components = "%s/%s" % (source_counts[0], target_counts[0])
        assets = "%s/%s" % (source_counts[1], target_counts[1])
        status = "" if result.handled else "  not handled"
        log("  %-30s %-7s %13s %13s %8d %8d %8s %8d%s" % (
            result.name, result.format, components, assets, result.copied, result.skipped,
            result.planned if args.dry_run else "", result.failed, status))
    log("  (components and assets are source/target)")
    unhandled = [r for r in results if not r.handled]
    if unhandled:
        log("Not handled: " + ", ".join("%s (%s)" % (r.name, r.format) for r in unhandled))
    failed = sum(r.failed for r in results)
    return 1 if failed else 0


def build_parser():
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--source-url", help="Source server base URL, e.g. https://old.example.com")
    common.add_argument("--source-user", default="admin")
    common.add_argument("--source-password-file", help="file holding the source password")
    common.add_argument("--target-url", help="Target Astrum base URL")
    common.add_argument("--target-user", default="admin")
    common.add_argument("--target-password-file", help="file holding the target password")
    common.add_argument("--timeout", type=float, default=300, help="socket timeout in seconds (default 300)")
    common.add_argument("--insecure", action="store_true", help="do not verify TLS certificates")

    parser = argparse.ArgumentParser(prog="repository-copy", description="Copy configuration and hosted content from one repository server to an Astrum Repository server over the REST API.")
    commands = parser.add_subparsers(dest="command", metavar="command")
    commands.required = True

    inventory = commands.add_parser("inventory", parents=[common], help="describe the source server")
    inventory.add_argument("--output", help="write the inventory as JSON to this file")
    inventory.add_argument("--workers", type=int, default=4, help="repositories counted in parallel (default 4)")
    inventory.set_defaults(func=cmd_inventory)

    config = commands.add_parser("config", parents=[common], help="recreate blob stores, repositories and security on the target")
    config.add_argument("--dry-run", action="store_true", help="only show what would be done")
    config.add_argument("--keep-docker-ports", action="store_true", help="keep the HTTP/HTTPS connector ports of docker repositories")
    config.add_argument("--keep-blob-store-paths", action="store_true", help="create file blob stores with the source paths instead of paths named after them")
    config.add_argument("--new-passwords-file", help="file (mode 0600) that receives the generated passwords of created users")
    config.set_defaults(func=cmd_config)

    content = commands.add_parser("content", parents=[common], help="copy hosted repository content to the target")
    content.add_argument("--repository", action="append", metavar="NAME", help="copy only this hosted repository (repeatable)")
    content.add_argument("--workers", type=int, default=4, help="parallel transfers (default 4)")
    content.add_argument("--temp-dir", help="directory for transfer buffers (default: system temp)")
    content.add_argument("--dry-run", action="store_true", help="only count what would be copied")
    content.add_argument("--verbose", action="store_true", help="print every copied item")
    content.set_defaults(func=cmd_content)
    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    try:
        return args.func(args)
    except MigrationError as error:
        log("error: %s" % error)
        return 2
    except KeyboardInterrupt:
        log("interrupted")
        return 130


if __name__ == "__main__":
    sys.exit(main())
