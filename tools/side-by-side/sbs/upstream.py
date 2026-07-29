import email.utils
import hashlib
import http.client
import http.server
import threading
import time
import urllib.parse

FIXED_TIME = email.utils.formatdate(1760000000, usegmt=True)


class _Server(http.server.ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True


class Upstream:
    def __init__(self, port):
        self.files = {}
        self.requests = []
        self.lock = threading.Lock()
        upstream = self

        class Handler(http.server.BaseHTTPRequestHandler):
            protocol_version = "HTTP/1.1"

            def log_message(self, *args):
                pass

            def do_GET(self):
                upstream.serve(self, body=True)

            def do_HEAD(self):
                upstream.serve(self, body=False)

        self.server = _Server(("127.0.0.1", port), Handler)
        self.port = port
        self.base = "http://127.0.0.1:%d" % port
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def add(self, path, content, content_type="application/octet-stream", headers=None):
        if isinstance(content, str):
            content = content.encode("utf-8")
        with self.lock:
            self.files[path] = (content, content_type, dict(headers or {}))

    def serve(self, handler, body):
        parsed = urllib.parse.urlsplit(handler.path)
        with self.lock:
            self.requests.append((handler.command, handler.path, dict(handler.headers.items())))
            entry = self.files.get(parsed.path)
        if entry is None:
            payload = b"not found\n"
            handler.send_response(404)
            handler.send_header("Content-Type", "text/plain")
            handler.send_header("Content-Length", str(len(payload)))
            handler.end_headers()
            if body:
                handler.wfile.write(payload)
            return
        content, ctype, extra = entry
        etag = '"%s"' % hashlib.sha1(content).hexdigest()
        if handler.headers.get("If-None-Match") == etag or handler.headers.get("If-Modified-Since") == FIXED_TIME:
            handler.send_response(304)
            handler.send_header("ETag", etag)
            handler.send_header("Content-Length", "0")
            handler.end_headers()
            return
        handler.send_response(200)
        handler.send_header("Content-Type", ctype)
        handler.send_header("Content-Length", str(len(content)))
        handler.send_header("ETag", etag)
        handler.send_header("Last-Modified", FIXED_TIME)
        for key, value in extra.items():
            handler.send_header(key, value)
        handler.end_headers()
        if body:
            handler.wfile.write(content)

    def count(self, path):
        with self.lock:
            return sum(1 for _, p, _ in self.requests if urllib.parse.urlsplit(p).path == path)

    def close(self):
        self.server.shutdown()
        self.server.server_close()


class Exchange:
    def __init__(self, method, path, request_headers, status, headers, body, started):
        self.method = method
        self.path = path
        self.request_headers = request_headers
        self.status = status
        self.headers = headers
        self.body = body
        self.started = started


class Recorder:
    def __init__(self, port, target, keep_body=1 << 20):
        self.target = urllib.parse.urlsplit(target)
        self.exchanges = []
        self.lock = threading.Lock()
        self.keep_body = keep_body
        recorder = self

        class Handler(http.server.BaseHTTPRequestHandler):
            protocol_version = "HTTP/1.1"

            def log_message(self, *args):
                pass

            def handle_one_request(self):
                try:
                    super().handle_one_request()
                except (ConnectionError, OSError):
                    self.close_connection = True

            def forward(self):
                recorder.forward(self)

            do_GET = do_HEAD = do_PUT = do_POST = do_DELETE = do_PATCH = do_OPTIONS = forward

        self.server = _Server(("127.0.0.1", port), Handler)
        self.port = port
        self.base = "http://127.0.0.1:%d" % port
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def forward(self, handler):
        started = time.monotonic()
        length = handler.headers.get("Content-Length")
        chunked = "chunked" in (handler.headers.get("Transfer-Encoding") or "").lower()
        if length:
            data = handler.rfile.read(int(length))
        elif chunked:
            data = read_chunked(handler.rfile)
        else:
            data = None
        headers = {}
        for key, value in handler.headers.items():
            if key.lower() in ("transfer-encoding", "connection", "proxy-connection", "keep-alive"):
                continue
            headers[key] = value
        if data is not None:
            headers["Content-Length"] = str(len(data))
        port = self.target.port or (443 if self.target.scheme == "https" else 80)
        if self.target.scheme == "https":
            connection = http.client.HTTPSConnection(self.target.hostname, port, timeout=600)
        else:
            connection = http.client.HTTPConnection(self.target.hostname, port, timeout=600)
        try:
            connection.putrequest(handler.command, handler.path, skip_host=True, skip_accept_encoding=True)
            for key, value in headers.items():
                connection.putheader(key, value)
            connection.endheaders()
            if data:
                connection.send(data)
            response = connection.getresponse()
            body = b"" if handler.command == "HEAD" else response.read()
            response_headers = response.getheaders()
            status = response.status
            reason = response.reason
        except (OSError, http.client.HTTPException) as error:
            status, reason, response_headers, body = 502, "Bad Gateway", [("Content-Type", "text/plain")], str(error).encode()
        finally:
            connection.close()
        handler.send_response_only(status, reason)
        sent_length = False
        for key, value in response_headers:
            lower = key.lower()
            if lower in ("transfer-encoding", "connection", "keep-alive"):
                continue
            if lower == "content-length":
                sent_length = True
            handler.send_header(key, value)
        if not sent_length and handler.command != "HEAD" and status not in (204, 304):
            handler.send_header("Content-Length", str(len(body)))
        handler.end_headers()
        if body and handler.command != "HEAD":
            handler.wfile.write(body)
        with self.lock:
            self.exchanges.append(Exchange(handler.command, handler.path, dict(headers), status, response_headers,
                                           body if len(body) <= self.keep_body else body[:self.keep_body], started))

    def close(self):
        self.server.shutdown()
        self.server.server_close()


def read_chunked(stream):
    out = b""
    while True:
        line = stream.readline().strip()
        size = int(line.split(b";")[0], 16)
        if size == 0:
            while stream.readline().strip():
                pass
            return out
        out += stream.read(size)
        stream.readline()
