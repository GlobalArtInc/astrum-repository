import difflib
import gzip
import hashlib
import json

from . import normalize

DEFAULT_HEADERS = (
    "content-type", "content-length", "etag", "last-modified", "location", "docker-content-digest",
    "docker-distribution-api-version", "docker-upload-uuid", "www-authenticate", "content-range",
    "accept-ranges", "content-disposition", "allow", "range", "oci-subject", "oci-filters-applied",
    "x-error-code", "link",
)
PRESENCE_HEADERS = {"last-modified", "docker-upload-uuid", "accept-ranges"}
SET_HEADERS = {"allow"}


class Spec:
    def __init__(self, body="auto", headers=None, extra_headers=(), skip_headers=(), volatile=(), unordered=True,
                 transform=None, exact_headers=(), presence_headers=(), text_rules=None, max_lines=12,
                 error_body=False):
        self.body = body
        self.headers = tuple(h.lower() for h in (headers if headers is not None else DEFAULT_HEADERS)) + tuple(
            h.lower() for h in extra_headers)
        self.skip_headers = {h.lower() for h in skip_headers}
        self.volatile = volatile
        self.unordered = unordered
        self.transform = transform
        self.exact_headers = {h.lower() for h in exact_headers}
        self.presence_headers = {h.lower() for h in presence_headers}
        self.text_rules = text_rules
        self.max_lines = max_lines
        self.error_body = error_body


class Observation:
    def __init__(self, step, kind, data, spec, normalizer, note=None):
        self.step = step
        self.kind = kind
        self.data = data
        self.spec = spec
        self.normalizer = normalizer
        self.note = note

    def summary(self):
        if self.kind == "http":
            return "%s %s -> %s" % (self.data["method"], self.data["path"], self.data["status"])
        return "%s" % (self.data.get("value"),)


class Difference:
    def __init__(self, id, field, reference, candidate, detail=None):
        self.id = id
        self.field = field
        self.reference = reference
        self.candidate = candidate
        self.detail = detail
        self.allowed = None
        self.status = None

    def as_json(self):
        return {"id": self.id, "field": self.field, "reference": self.reference, "candidate": self.candidate,
                "detail": self.detail, "allowed": self.allowed, "status": self.status}


def media(value):
    if value is None:
        return None
    parts = [p.strip().lower() for p in value.split(";") if p.strip()]
    if not parts:
        return None
    params = sorted(p.replace(" ", "").replace('"', "") for p in parts[1:])
    return ";".join([parts[0]] + [p for p in params if p != "charset=utf-8"])


def challenge(value):
    if value is None:
        return None
    scheme, _, params = value.strip().partition(" ")
    scheme = scheme.lower()
    if scheme == "basic":
        return "basic"
    return scheme + " " + params


def body_kind(spec, data):
    if spec.body != "auto":
        return spec.body
    if data["method"] == "HEAD" or data["status"] in (204, 304) or data["status"] >= 300:
        return "none"
    ctype = media(data["headers"].get("content-type")) or ""
    if "json" in ctype:
        return "json"
    return "bytes"


def compare(step_id, ref, cand):
    if ref is None or cand is None:
        return [Difference(step_id, "exists", ref.summary() if ref else None, cand.summary() if cand else None)]
    if ref.kind != cand.kind:
        return [Difference(step_id, "kind", ref.kind, cand.kind)]
    if ref.kind == "fact":
        return compare_fact(step_id, ref, cand)
    return compare_http(step_id, ref, cand)


def compare_fact(step_id, ref, cand):
    spec = ref.spec
    a, b = ref.data.get("value"), cand.data.get("value")
    if spec.body == "json" or isinstance(a, (dict, list)):
        a = ref.normalizer.json(a, spec.volatile)
        b = cand.normalizer.json(b, spec.volatile)
        if spec.transform:
            a, b = spec.transform(a), spec.transform(b)
        if spec.body == "keys":
            a, b = keys(a), keys(b)
        return [Difference(step_id, "value" + path, x, y) for path, x, y in json_diff(a, b, spec.unordered)]
    if isinstance(a, str) and isinstance(b, str):
        a, b = ref.normalizer.text(a), cand.normalizer.text(b)
    if a != b:
        return [Difference(step_id, "value", a, b)]
    return []


def compare_http(step_id, ref, cand):
    spec = ref.spec
    out = []
    ra, ca = ref.data, cand.data
    if ra["status"] != ca["status"]:
        out.append(Difference(step_id, "status", ra["status"], ca["status"],
                              "reference: %s | candidate: %s" % (snippet(ra), snippet(ca))))
    kind = body_kind(spec, ra)
    if body_kind(spec, ca) != kind and spec.body == "auto" and ra["status"] == ca["status"]:
        kind = "bytes" if "bytes" in (kind, body_kind(spec, ca)) else kind
    same_bytes = ra["body"] == ca["body"]
    errors = ra["status"] >= 400 and ca["status"] >= 400
    if errors and not spec.error_body:
        kind = "none"
    quiet_error = errors and kind == "none"
    empty = not ra["body"] and not ca["body"] and ra["method"] != "HEAD"
    for name in spec.headers:
        if name in spec.skip_headers:
            continue
        if name in ("content-type", "content-length") and quiet_error and name not in spec.exact_headers:
            continue
        if name == "content-type" and empty and name not in spec.exact_headers:
            continue
        x = ra["headers"].get(name)
        y = ca["headers"].get(name)
        if x is None and y is None:
            continue
        if name in spec.presence_headers or (name in PRESENCE_HEADERS and name not in spec.exact_headers):
            if (x is None) != (y is None):
                out.append(Difference(step_id, "header." + name, x, y))
            continue
        if name == "content-type":
            x, y = media(x), media(y)
        elif name == "www-authenticate":
            x = challenge(ref.normalizer.text(x)) if x is not None else None
            y = challenge(cand.normalizer.text(y)) if y is not None else None
        elif name in ("content-length", "etag", "docker-content-digest") and name not in spec.exact_headers:
            exact = (ra["method"] == "HEAD" and spec.body in ("auto", "bytes")) or (
                kind == "bytes" and same_bytes and ra["status"] == ca["status"])
            if name == "docker-content-digest":
                exact = exact or kind != "none"
            if name == "content-length" and not exact and (kind != "bytes" or ra["status"] >= 300):
                continue
            if not exact:
                if (x is None) != (y is None):
                    out.append(Difference(step_id, "header." + name, x, y))
                continue
        elif name in SET_HEADERS:
            x = ",".join(sorted(p.strip().upper() for p in x.split(","))) if x else x
            y = ",".join(sorted(p.strip().upper() for p in y.split(","))) if y else y
        else:
            x = ref.normalizer.text(x) if x is not None else None
            y = cand.normalizer.text(y) if y is not None else None
        if x != y:
            out.append(Difference(step_id, "header." + name, x, y))
    if kind != "none" and ra["status"] == ca["status"]:
        out.extend(compare_body(step_id, kind, ref, cand))
    return out


def snippet(data, limit=160):
    body = data.get("body") or b""
    text = body[:limit].decode("utf-8", "replace")
    return " ".join(text.split())


def compare_body(step_id, kind, ref, cand):
    spec = ref.spec
    a, b = ref.data["body"], cand.data["body"]
    if kind == "bytes":
        if a == b:
            return []
        detail = "reference %d bytes sha256 %s, candidate %d bytes sha256 %s" % (
            len(a), hashlib.sha256(a).hexdigest()[:16], len(b), hashlib.sha256(b).hexdigest()[:16])
        if is_text(a) and is_text(b):
            detail += "\n" + text_diff(ref.normalizer.text(a.decode("utf-8", "replace")),
                                       cand.normalizer.text(b.decode("utf-8", "replace")), spec.max_lines)
        return [Difference(step_id, "body", "%d bytes" % len(a), "%d bytes" % len(b), detail)]
    if kind == "gunzip":
        try:
            a, b = gzip.decompress(a), gzip.decompress(b)
        except (OSError, EOFError) as error:
            return [Difference(step_id, "body", "gzip", "gzip", "cannot decompress: %s" % error)]
        if a == b:
            return []
        return [Difference(step_id, "body", "%d bytes unzipped" % len(a), "%d bytes unzipped" % len(b))]
    if kind == "text":
        x = ref.normalizer.text(a.decode("utf-8", "replace"))
        y = cand.normalizer.text(b.decode("utf-8", "replace"))
        if spec.transform:
            x, y = spec.transform(x), spec.transform(y)
        if x == y:
            return []
        return [Difference(step_id, "body", "text", "text", text_diff(x, y, spec.max_lines))]
    if kind in ("json", "shape", "keys"):
        if a and b and not valid_json(a) and not valid_json(b):
            return compare_body(step_id, "text", ref, cand)
        try:
            x = json.loads(a.decode("utf-8")) if a else None
        except ValueError:
            return [Difference(step_id, "body", "not JSON", "JSON" if valid_json(b) else "not JSON", snippet(ref.data))]
        try:
            y = json.loads(b.decode("utf-8")) if b else None
        except ValueError:
            return [Difference(step_id, "body", "JSON", "not JSON", snippet(cand.data))]
        x = ref.normalizer.json(x, spec.volatile)
        y = cand.normalizer.json(y, spec.volatile)
        if spec.transform:
            x, y = spec.transform(x), spec.transform(y)
        if kind == "shape":
            x, y = normalize.shape(x), normalize.shape(y)
        if kind == "keys":
            x, y = keys(x), keys(y)
        return [Difference(step_id, "body" + path, p, q) for path, p, q in json_diff(x, y, spec.unordered)]
    return []


def keys(value):
    if isinstance(value, dict):
        return {k: keys(v) for k, v in value.items()}
    if isinstance(value, list):
        merged = {}
        for item in value:
            if isinstance(item, dict):
                for k, v in keys(item).items():
                    if merged.get(k) in (None, "*"):
                        merged[k] = v
        if merged:
            return [merged]
        return "*"
    return "*"


def valid_json(data):
    try:
        json.loads(data.decode("utf-8"))
        return True
    except ValueError:
        return False


def is_text(data):
    if b"\x00" in data[:4096]:
        return False
    try:
        data[:65536].decode("utf-8")
        return True
    except UnicodeDecodeError:
        return False


def text_diff(a, b, limit):
    lines = list(difflib.unified_diff(a.splitlines(), b.splitlines(), "reference", "candidate", lineterm="", n=1))
    if len(lines) > limit + 2:
        lines = lines[:limit + 2] + ["... %d more lines" % (len(lines) - limit - 2)]
    return "\n".join(lines)


def key_preview(item):
    keys = [k for k in ("name", "path", "version", "id", "format") if k in item]
    return keys or list(item)[:4]


def short(value, limit=200):
    text = normalize.canonical(value) if not isinstance(value, str) else json.dumps(value, ensure_ascii=False)
    return text if len(text) <= limit else text[:limit] + "..."


SUPERSET = [True]


def json_diff(a, b, unordered, path=""):
    if isinstance(a, dict) and isinstance(b, dict):
        out = []
        for key in sorted(set(a) | set(b), key=str):
            sub = "%s.%s" % (path, key)
            if key not in a:
                if SUPERSET[0]:
                    continue
                out.append((sub, "<missing>", short(b[key])))
            elif key not in b:
                out.append((sub, short(a[key]), "<missing>"))
            else:
                out.extend(json_diff(a[key], b[key], unordered, sub))
        return out
    if isinstance(a, list) and isinstance(b, list):
        key = normalize.item_key(a) if a else normalize.item_key(b)
        if key and normalize.item_key(b if a else a) in (key, None) and all(isinstance(i, dict) and key in i for i in a + b):
            left = {str(i[key]): i for i in a}
            right = {str(i[key]): i for i in b}
            out = []
            for k in sorted(set(left) | set(right)):
                sub = "%s[%s=%s]" % (path, key, k)
                if k not in left:
                    out.append((sub, "<missing>", short(right[k]) if not isinstance(right[k], dict) else
                                short({x: right[k][x] for x in key_preview(right[k])})))
                elif k not in right:
                    out.append((sub, short(left[k]), "<missing>"))
                else:
                    out.extend(json_diff(left[k], right[k], unordered, sub))
            if not unordered and [str(i[key]) for i in a] != [str(i[key]) for i in b] and set(left) == set(right):
                out.append((path + "(order)", short([i[key] for i in a]), short([i[key] for i in b])))
            return out
        if unordered:
            a = sorted(a, key=normalize.canonical)
            b = sorted(b, key=normalize.canonical)
        if len(a) != len(b):
            if all(not isinstance(i, (dict, list)) for i in a + b):
                missing = [i for i in a if i not in b]
                extra = [i for i in b if i not in a]
                return [(path, "only here: " + short(missing), "only here: " + short(extra))]
            return [(path + "(length)", "%d items" % len(a), "%d items" % len(b))] + [
                (path, short(a), short(b))]
        out = []
        for index, (x, y) in enumerate(zip(a, b)):
            out.extend(json_diff(x, y, unordered, "%s[%d]" % (path, index)))
        return out
    if a != b:
        return [(path or "$", short(a), short(b))]
    return []
