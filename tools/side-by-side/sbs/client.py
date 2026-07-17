import base64
import hashlib
import http.client
import json
import socket
import ssl
import time
import urllib.parse

DEFAULT = object()


class HarnessError(Exception):
    pass


class Response:
    def __init__(self, method, url, status, reason, headers, body, elapsed):
        self.method = method
        self.url = url
        self.status = status
        self.reason = reason
        self.header_list = headers
        self.body = body
        self.elapsed = elapsed

    def header(self, name, default=None):
        name = name.lower()
        values = [v for k, v in self.header_list if k.lower() == name]
        if not values:
            return default
        return ", ".join(values)

    def headers(self):
        out = {}
        for key, value in self.header_list:
            key = key.lower()
            out[key] = out[key] + ", " + value if key in out else value
        return out

    def json(self):
        if not self.body:
            return None
        return json.loads(self.body.decode("utf-8"))

    def text(self):
        return self.body.decode("utf-8", "replace")

    def __repr__(self):
        return "<%s %s -> %s>" % (self.method, self.url, self.status)


def basic(user, password):
    return "Basic " + base64.b64encode(("%s:%s" % (user, password)).encode("utf-8")).decode("ascii")


class Server:
    def __init__(self, name, url, user, password, timeout=120, insecure=False):
        self.name = name
        self.base = url.rstrip("/")
        parts = urllib.parse.urlsplit(self.base)
        if parts.scheme not in ("http", "https") or not parts.hostname:
            raise HarnessError("%s: not an http(s) URL: %s" % (name, url))
        self.scheme = parts.scheme
        self.host = parts.hostname
        self.port = parts.port or (443 if parts.scheme == "https" else 80)
        self.netloc = parts.netloc
        self.user = user
        self.password = password
        self.timeout = timeout
        self.insecure = insecure

    def authorization(self):
        return basic(self.user, self.password)

    def with_base(self, url):
        return Server(self.name, url, self.user, self.password, self.timeout, self.insecure)

    def url(self, path):
        if path.startswith(("http://", "https://")):
            return path
        return self.base + path

    def connection(self, scheme, host, port):
        if scheme == "https":
            context = ssl.create_default_context()
            if self.insecure:
                context.check_hostname = False
                context.verify_mode = ssl.CERT_NONE
            return http.client.HTTPSConnection(host, port, timeout=self.timeout, context=context)
        return http.client.HTTPConnection(host, port, timeout=self.timeout)

    def request(self, method, path, body=None, headers=None, auth=DEFAULT, retries=2, digest=False):
        url = self.url(path)
        parts = urllib.parse.urlsplit(url)
        target = parts.path or "/"
        if parts.query:
            target += "?" + parts.query
        send = {}
        for key, value in (headers or {}).items():
            if value is not None:
                send[key] = value
        if auth is DEFAULT:
            send.setdefault("Authorization", self.authorization())
        elif isinstance(auth, tuple):
            send["Authorization"] = basic(*auth)
        elif isinstance(auth, str):
            send["Authorization"] = auth
        if isinstance(body, str):
            body = body.encode("utf-8")
        if body is not None:
            send.setdefault("Content-Length", str(len(body)))
        attempt = 0
        while True:
            started = time.monotonic()
            connection = self.connection(parts.scheme, parts.hostname, parts.port or (443 if parts.scheme == "https" else 80))
            try:
                connection.putrequest(method, target, skip_accept_encoding=True)
                for key, value in send.items():
                    connection.putheader(key, value)
                connection.endheaders()
                if body:
                    connection.send(body)
                raw = connection.getresponse()
                if digest and method != "HEAD":
                    hasher = hashlib.sha256()
                    size = 0
                    while True:
                        chunk = raw.read(1 << 20)
                        if not chunk:
                            break
                        hasher.update(chunk)
                        size += len(chunk)
                    data = ("%d bytes sha256 %s" % (size, hasher.hexdigest())).encode()
                else:
                    data = b"" if method == "HEAD" else raw.read()
                return Response(method, url, raw.status, raw.reason, raw.getheaders(), data, time.monotonic() - started)
            except (OSError, http.client.HTTPException, socket.timeout) as error:
                if attempt < retries:
                    attempt += 1
                    time.sleep(attempt)
                    continue
                raise HarnessError("%s %s failed: %s" % (method, url, error))
            finally:
                connection.close()

    def json(self, method, path, payload=None, ok=(200, 201, 204), headers=None):
        send = {"Accept": "application/json"}
        send.update(headers or {})
        body = None
        if payload is not None:
            body = json.dumps(payload).encode("utf-8")
            send["Content-Type"] = "application/json"
        response = self.request(method, path, body=body, headers=send)
        if ok is not None and response.status not in ok:
            raise HarnessError("%s: %s %s -> %s %s" % (self.name, method, path, response.status, response.text()[:300]))
        return response

    def get_json(self, path):
        return self.json("GET", path, ok=(200,)).json()

    def paginate(self, path, **params):
        token = None
        while True:
            query = dict(params)
            if token:
                query["continuationToken"] = token
            separator = "&" if "?" in path else "?"
            data = self.get_json(path + separator + urllib.parse.urlencode(query)) if query else self.get_json(path)
            for item in data.get("items") or []:
                yield item
            token = data.get("continuationToken")
            if not token:
                return


def read_password(path, side):
    if not path:
        raise HarnessError("--%s-password-file is required" % side)
    try:
        with open(path, "r", encoding="utf-8") as handle:
            value = handle.read().rstrip("\r\n")
    except OSError as error:
        raise HarnessError("cannot read the %s password file: %s" % (side, error.strerror))
    if not value:
        raise HarnessError("the %s password file is empty" % side)
    return value
