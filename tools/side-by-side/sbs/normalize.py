import json
import re

UUID = re.compile(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
ISO_TIME = re.compile(r"\d{4}-\d\d-\d\d[T ]\d\d:\d\d(:\d\d(\.\d+)?)?(Z|[+-]\d\d:?\d\d)?")
HTTP_TIME = re.compile(r"(Mon|Tue|Wed|Thu|Fri|Sat|Sun), \d\d (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) \d{4} \d\d:\d\d:\d\d GMT")
MAVEN_SNAPSHOT = re.compile(r"(?<![0-9A-Za-z])20\d{6}\.\d{6}(?![0-9A-Za-z])")
MAVEN_STAMP = re.compile(r"(?<![0-9A-Za-z])20\d{12}(?![0-9A-Za-z])")
EPOCH_MILLIS = re.compile(r"(?<![0-9A-Za-z])1[789]\d{11}(?![0-9A-Za-z])")

TIME_KEYS = {
    "lastModified", "lastDownloaded", "blobCreated", "lastUpdated", "created", "updated", "modified",
    "createdAt", "updatedAt", "lastRun", "nextRun", "lastRunResult", "timestamp", "lastVerified",
    "expires", "expiresAt", "lastUsed", "date", "time", "blob_created", "last_modified", "last_updated",
}
OPAQUE_KEYS = {"continuationToken", "id", "componentId", "assetId", "sessionId", "etag", "_rev", "token"}


class Normalizer:
    def __init__(self, bases, extra=None, volatile=()):
        self.volatile = set(volatile)
        self.rules = []
        seen = set()
        for base in sorted(bases, key=len, reverse=True):
            base = base.rstrip("/")
            for variant in (base, base.split("://", 1)[-1]):
                if variant and variant not in seen:
                    seen.add(variant)
                    self.rules.append((re.compile(re.escape(variant)), "<base>" if "://" in variant else "<host>"))
        for pattern, replacement in extra or []:
            self.rules.append((re.compile(pattern), replacement))

    def text(self, value):
        for pattern, replacement in self.rules:
            value = pattern.sub(replacement, value)
        value = UUID.sub("<uuid>", value)
        value = ISO_TIME.sub("<time>", value)
        value = HTTP_TIME.sub("<time>", value)
        value = MAVEN_SNAPSHOT.sub("<snapshot-ts>", value)
        value = MAVEN_STAMP.sub("<stamp>", value)
        value = EPOCH_MILLIS.sub("<millis>", value)
        return value

    def url(self, value):
        return self.text(value)

    def json(self, value, volatile=(), opaque=True, times=True):
        volatile = set(volatile) | self.volatile
        return self._json(value, volatile, opaque, times, None)

    def _json(self, value, volatile, opaque, times, key):
        if isinstance(value, dict):
            out = {}
            for k, v in value.items():
                out[self.text(k)] = self._json(v, volatile, opaque, times, k)
            return out
        if isinstance(value, list):
            return [self._json(v, volatile, opaque, times, key) for v in value]
        if key is not None and key in volatile:
            return "<volatile>"
        if key is not None and value is not None:
            if times and key in TIME_KEYS:
                return "<time>"
            if opaque and key in OPAQUE_KEYS and isinstance(value, str) and looks_opaque(value):
                return "<opaque>"
        if isinstance(value, str):
            return self.text(value)
        return value


def looks_opaque(value):
    if UUID.fullmatch(value):
        return True
    return len(value) >= 12 and re.fullmatch(r"[A-Za-z0-9+/=_]+", value) is not None


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False)


KEY_FIELDS = ("name", "path", "userId", "id", "@id", "format", "repository", "type", "key", "version", "typeId", "alias", "realm")


def item_key(items):
    if not items or not all(isinstance(i, dict) for i in items):
        return None
    for field in KEY_FIELDS:
        values = [i.get(field) for i in items]
        if all(isinstance(v, (str, int)) for v in values) and len(set(values)) == len(values):
            return field
    return None


def sort_lists(value):
    if isinstance(value, dict):
        return {k: sort_lists(v) for k, v in value.items()}
    if isinstance(value, list):
        items = [sort_lists(v) for v in value]
        return sorted(items, key=canonical)
    return value


def shape(value):
    if isinstance(value, dict):
        return {k: shape(v) for k, v in sorted(value.items())}
    if isinstance(value, list):
        shapes = []
        for item in value:
            s = shape(item)
            if s not in shapes:
                shapes.append(s)
        return sorted(shapes, key=canonical)
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, (int, float)):
        return "number"
    return "string"
