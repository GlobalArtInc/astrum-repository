import json
import re
import time
import uuid
import urllib.parse

from ..compare import Spec
from ..harness import blob_store

JSON = {"Content-Type": "application/json", "Accept": "application/json"}


def only(prefix, key="name", keep=()):
    def transform(value):
        if isinstance(value, list):
            return [i for i in value if isinstance(i, dict) and (str(i.get(key, "")).startswith(prefix)
                                                                  or i.get(key) in keep)]
        return value
    return transform


def items_only(prefix, key="name"):
    def transform(value):
        if isinstance(value, dict) and isinstance(value.get("items"), list):
            value = dict(value)
            value["items"] = [i for i in value["items"] if str(i.get(key, "")).startswith(prefix)]
        return value
    return transform


def hosted(name, fmt_extra=None, write_policy="ALLOW", strict=True, cleanup=None, component=None):
    body = {"name": name, "online": True, "storage": dict(blob_store(), writePolicy=write_policy,
                                                          strictContentTypeValidation=strict)}
    if cleanup is not None:
        body["cleanup"] = cleanup
    if component is not None:
        body["component"] = component
    body.update(fmt_extra or {})
    return body


def proxy(name, remote, fmt_extra=None, strict=True):
    body = {"name": name, "online": True,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": strict},
            "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": True, "timeToLive": 1440},
            "httpClient": {"blocked": False, "autoBlock": False}}
    body.update(fmt_extra or {})
    return body


def group(name, members, fmt_extra=None, writable=None):
    body = {"name": name, "online": True,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": True},
            "group": {"memberNames": list(members)}}
    if writable:
        body["group"]["writableMember"] = writable
    body.update(fmt_extra or {})
    return body


def quote(value):
    return urllib.parse.quote(value, safe="")


def multipart(fields, files, boundary=None):
    boundary = boundary or "----side-by-side-" + uuid.uuid4().hex
    out = b""
    for name, value in fields:
        out += ("--%s\r\nContent-Disposition: form-data; name=\"%s\"\r\n\r\n" % (boundary, name)).encode("utf-8")
        out += (value if isinstance(value, bytes) else str(value).encode("utf-8")) + b"\r\n"
    for name, filename, content, ctype in files:
        out += ("--%s\r\nContent-Disposition: form-data; name=\"%s\"; filename=\"%s\"\r\nContent-Type: %s\r\n\r\n"
                % (boundary, name, filename, ctype or "application/octet-stream")).encode("utf-8")
        out += content + b"\r\n"
    out += ("--%s--\r\n" % boundary).encode("utf-8")
    return out, "multipart/form-data; boundary=" + boundary


def components(t, repository):
    return list(t.server.paginate("/service/rest/v1/components", repository=repository))


def assets(t, repository):
    return list(t.server.paginate("/service/rest/v1/assets", repository=repository))


COUNTERS = ("downloadCount", "lastDownloaded")
GENERATED = ("checksum", "fileSize", "blobCreated", "blobUpdated", "lastModified", "blobRef")


def generated_assets(predicate):
    def blank(asset):
        if isinstance(asset, dict) and predicate(str(asset.get("path", ""))):
            return {k: ("<generated>" if k in GENERATED else v) for k, v in asset.items()}
        return asset

    def transform(value):
        if isinstance(value, dict) and isinstance(value.get("items"), list):
            items = []
            for item in value["items"]:
                if isinstance(item, dict) and isinstance(item.get("assets"), list):
                    item = dict(item, assets=[blank(a) for a in item["assets"]])
                items.append(blank(item))
            return dict(value, items=items)
        return blank(value)
    return transform


def without_assets(predicate):
    def keep(asset):
        return not (isinstance(asset, dict) and predicate(str(asset.get("path", ""))))

    def transform(value):
        if isinstance(value, dict) and isinstance(value.get("items"), list):
            items = []
            for item in value["items"]:
                if not keep(item):
                    continue
                if isinstance(item, dict) and isinstance(item.get("assets"), list):
                    item = dict(item, assets=[a for a in item["assets"] if keep(a)])
                items.append(item)
            return dict(value, items=items)
        return value
    return transform


def rest_listing(t, step, repository, kinds=("components", "assets"), volatile=COUNTERS, generated=None, derived=None):
    for kind in kinds:
        transforms = [f for f in (without_assets(derived) if derived else None,
                                  generated_assets(generated) if generated else None) if f]
        t.json_step("%s %s" % (step, kind), "/service/rest/v1/%s?repository=%s" % (kind, quote(repository)),
                    volatile=volatile, transform=chain(*transforms) if transforms else None)
    if derived:
        paths = sorted(a.get("path") for a in assets(t, repository) if derived(str(a.get("path", ""))))
        t.fact("%s derived assets" % step, paths)


def settle_search(t, repository, timeout=20):
    expected = len(components(t, repository))
    deadline = time.monotonic() + timeout
    while True:
        found = len(list(t.server.paginate("/service/rest/v1/search", repository=repository)))
        if found == expected or time.monotonic() > deadline:
            return found
        time.sleep(0.5)


def search(t, step, **params):
    t.json_step(step, "/service/rest/v1/search?" + urllib.parse.urlencode(params))


def strip_keys(*keys):
    def transform(value):
        def walk(v):
            if isinstance(v, dict):
                return {k: walk(x) for k, x in v.items() if k not in keys}
            if isinstance(v, list):
                return [walk(x) for x in v]
            return v
        return walk(value)
    return transform


def chain(*fns):
    def transform(value):
        for fn in fns:
            value = fn(value)
        return value
    return transform


def text_sub(pattern, replacement):
    compiled = re.compile(pattern)

    def transform(value):
        return compiled.sub(replacement, value)
    return transform


def dumps(value):
    return json.dumps(value).encode("utf-8")


BYTES = Spec(body="bytes")
NONE = Spec(body="none")
JSON_SPEC = Spec(body="json")
TEXT = Spec(body="text")
