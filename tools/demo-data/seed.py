import base64
import hashlib
import io
import json
import os
import sys
import tarfile
import time
import urllib.error
import urllib.request
import uuid
import zipfile

BASE = os.environ.get("ASTRUM_URL", "http://127.0.0.1:8081").rstrip("/")
USER = os.environ.get("ASTRUM_USER", "admin")
PASSWORD = os.environ.get("ASTRUM_PASSWORD", "")
STORE = os.environ.get("ASTRUM_BLOB_STORE", "default")
AUTH = "Basic " + base64.b64encode(f"{USER}:{PASSWORD}".encode()).decode()
REPORT = []


def call(method, path, body=None, headers=None, raw=False):
    data = None
    sent = dict(headers or {})
    if body is not None and not raw:
        data = json.dumps(body).encode()
        sent.setdefault("Content-Type", "application/json")
    elif body is not None:
        data = body
    sent.setdefault("Authorization", AUTH)
    request = urllib.request.Request(BASE + path, data=data, method=method, headers=sent)
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            return response.status, response.read(), response.headers
    except urllib.error.HTTPError as error:
        return error.code, error.read(), error.headers


def note(kind, name, status, ok):
    REPORT.append((kind, name, status, ok))
    mark = "ok " if ok else "-- "
    print(f"{mark}{kind:<12} {name:<40} {status}")


def multipart(fields, files):
    boundary = "astrum-demo-" + uuid.uuid4().hex
    out = io.BytesIO()
    for key, value in fields.items():
        out.write(f"--{boundary}\r\nContent-Disposition: form-data; name=\"{key}\"\r\n\r\n{value}\r\n".encode())
    for key, (filename, content) in files.items():
        out.write(
            f"--{boundary}\r\nContent-Disposition: form-data; name=\"{key}\"; filename=\"{filename}\"\r\n"
            "Content-Type: application/octet-stream\r\n\r\n".encode()
        )
        out.write(content)
        out.write(b"\r\n")
    out.write(f"--{boundary}--\r\n".encode())
    return out.getvalue(), f"multipart/form-data; boundary={boundary}"


def storage(strict=True, policy="allow"):
    return {"blobStoreName": STORE, "strictContentTypeValidation": strict, "writePolicy": policy}


def proxy_parts(remote):
    return {
        "storage": {"blobStoreName": STORE, "strictContentTypeValidation": True},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": True, "timeToLive": 1440},
        "httpClient": {"blocked": False, "autoBlock": True},
    }


def repository(api, kind, name, extra):
    status, _, _ = call("GET", f"/service/rest/v1/repositories/{name}")
    if status == 200:
        note("repository", name, "exists", True)
        return
    body = {"name": name, "online": True}
    body.update(extra)
    status, text, _ = call("POST", f"/service/rest/v1/repositories/{api}/{kind}", body)
    note("repository", name, status if status != 201 else "created", status == 201)
    if status != 201:
        print("   ", text.decode(errors="replace")[:300])


def cleanup_policy(name, format_id, days, notes):
    status, _, _ = call("GET", f"/service/rest/v1/cleanup-policies/{name}")
    if status == 200:
        note("cleanup", name, "exists", True)
        return
    body = {"name": name, "format": format_id, "notes": notes, "criteriaLastDownloaded": days}
    status, text, _ = call("POST", "/service/rest/v1/cleanup-policies", body)
    note("cleanup", name, status, status in (200, 201, 204))
    if status not in (200, 201, 204):
        print("   ", text.decode(errors="replace")[:300])


def role(identifier, description, privileges):
    status, _, _ = call("GET", f"/service/rest/v1/security/roles/{identifier}")
    if status == 200:
        note("role", identifier, "exists", True)
        return
    body = {"id": identifier, "name": identifier, "description": description, "privileges": privileges, "roles": []}
    status, text, _ = call("POST", "/service/rest/v1/security/roles", body)
    note("role", identifier, status, status in (200, 201))
    if status not in (200, 201):
        print("   ", text.decode(errors="replace")[:300])


def upload(repository_name, label, fields, files):
    body, content_type = multipart(fields, files)
    status, text, _ = call(
        "POST",
        f"/service/rest/v1/components?repository={repository_name}",
        body,
        {"Content-Type": content_type},
        raw=True,
    )
    exists = status == 400 and b"redeploy" in text.lower() or status == 400 and b"already" in text.lower()
    note("upload", f"{repository_name}: {label}", "exists" if exists else status, status in (200, 204) or exists)
    if status not in (200, 204) and not exists:
        print("   ", text.decode(errors="replace")[:300])


def tar_gz(entries):
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w:gz") as archive:
        for name, content in entries.items():
            data = content.encode() if isinstance(content, str) else content
            info = tarfile.TarInfo(name)
            info.size = len(data)
            info.mode = 0o644
            info.mtime = 1767225600
            archive.addfile(info, io.BytesIO(data))
    return out.getvalue()


def zip_bytes(entries):
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, content in entries.items():
            archive.writestr(zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0)), content)
    return out.getvalue()


def npm_package(name, version, description):
    manifest = {
        "name": name,
        "version": version,
        "description": description,
        "main": "index.js",
        "license": "MIT",
        "keywords": ["demo", "astrum"],
    }
    return tar_gz(
        {
            "package/package.json": json.dumps(manifest, indent=2) + "\n",
            "package/index.js": f"module.exports = () => 'Hello from {name} {version}';\n",
            "package/README.md": f"# {name}\n\n{description}\n",
        }
    )


def wheel(name, version, summary):
    module = name.replace("-", "_")
    dist = f"{module}-{version}.dist-info"
    files = {
        f"{module}/__init__.py": f"def hello():\n    return 'Hello from {name} {version}'\n",
        f"{dist}/METADATA": (
            f"Metadata-Version: 2.1\nName: {name}\nVersion: {version}\nSummary: {summary}\n"
            "License: MIT\nRequires-Python: >=3.8\n"
        ),
        f"{dist}/WHEEL": "Wheel-Version: 1.0\nGenerator: astrum-demo\nRoot-Is-Purelib: true\nTag: py3-none-any\n",
    }
    record = []
    for path, content in files.items():
        digest = base64.urlsafe_b64encode(hashlib.sha256(content.encode()).digest()).rstrip(b"=").decode()
        record.append(f"{path},sha256={digest},{len(content.encode())}")
    record.append(f"{dist}/RECORD,,")
    files[f"{dist}/RECORD"] = "\n".join(record) + "\n"
    return f"{module}-{version}-py3-none-any.whl", zip_bytes(files)


def helm_chart(name, version, description):
    return tar_gz(
        {
            f"{name}/Chart.yaml": (
                f"apiVersion: v2\nname: {name}\nversion: {version}\nappVersion: \"{version}\"\n"
                f"description: {description}\ntype: application\n"
            ),
            f"{name}/values.yaml": "replicaCount: 1\nimage:\n  repository: nginx\n  tag: stable\n",
            f"{name}/templates/configmap.yaml": (
                "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: {{ .Release.Name }}-demo\n"
                "data:\n  greeting: hello\n"
            ),
        }
    )


def nuget_package(identifier, version, description):
    nuspec = (
        '<?xml version="1.0" encoding="utf-8"?>\n'
        '<package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">\n'
        f"  <metadata>\n    <id>{identifier}</id>\n    <version>{version}</version>\n"
        f"    <authors>GlobalArt</authors>\n    <description>{description}</description>\n"
        "    <license type=\"expression\">MIT</license>\n  </metadata>\n</package>\n"
    )
    return zip_bytes(
        {
            f"{identifier}.nuspec": nuspec,
            "lib/netstandard2.0/readme.txt": f"{identifier} {version}\n",
        }
    )


def docker_image(repository_name, image, tag):
    prefix = f"/repository/{repository_name}/v2/{image}"
    layer = tar_gz({"hello.txt": f"Hello from {image}:{tag}\n"})
    config = json.dumps(
        {
            "architecture": "amd64",
            "os": "linux",
            "config": {"Cmd": ["cat", "/hello.txt"]},
            "rootfs": {"type": "layers", "diff_ids": ["sha256:" + hashlib.sha256(tar_bytes(image, tag)).hexdigest()]},
        }
    ).encode()
    for blob in (layer, config):
        digest = "sha256:" + hashlib.sha256(blob).hexdigest()
        status, _, _ = call("HEAD", f"{prefix}/blobs/{digest}")
        if status == 200:
            continue
        status, text, headers = call("POST", f"{prefix}/blobs/uploads/", b"", {"Content-Length": "0"}, raw=True)
        if status != 202:
            note("docker", f"{image}:{tag}", status, False)
            print("   ", text.decode(errors="replace")[:300])
            return
        location = headers.get("Location", "")
        if location.startswith("http"):
            location = location[len(BASE):] if location.startswith(BASE) else "/" + location.split("/", 3)[3]
        separator = "&" if "?" in location else "?"
        status, text, _ = call(
            "PUT",
            f"{location}{separator}digest={digest}",
            blob,
            {"Content-Type": "application/octet-stream"},
            raw=True,
        )
        if status != 201:
            note("docker", f"{image}:{tag}", status, False)
            print("   ", text.decode(errors="replace")[:300])
            return
    manifest = json.dumps(
        {
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "config": {
                "mediaType": "application/vnd.oci.image.config.v1+json",
                "digest": "sha256:" + hashlib.sha256(config).hexdigest(),
                "size": len(config),
            },
            "layers": [
                {
                    "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
                    "digest": "sha256:" + hashlib.sha256(layer).hexdigest(),
                    "size": len(layer),
                }
            ],
        }
    ).encode()
    status, text, _ = call(
        "PUT",
        f"{prefix}/manifests/{tag}",
        manifest,
        {"Content-Type": "application/vnd.oci.image.manifest.v1+json"},
        raw=True,
    )
    note("docker", f"{repository_name}: {image}:{tag}", status, status == 201)
    if status != 201:
        print("   ", text.decode(errors="replace")[:300])


def tar_bytes(image, tag):
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w") as archive:
        data = f"Hello from {image}:{tag}\n".encode()
        info = tarfile.TarInfo("hello.txt")
        info.size = len(data)
        info.mode = 0o644
        info.mtime = 1767225600
        archive.addfile(info, io.BytesIO(data))
    return out.getvalue()


def cache_artifact(repository_name, path, content, headers):
    status, _, _ = call("PUT", f"/repository/{repository_name}{path}", content, headers, raw=True)
    note("cache", f"{repository_name}{path}", status, status in (200, 202, 409))


def warm(label, path):
    started = time.monotonic()
    status, body, _ = call("GET", path)
    note("proxy", label, f"{status} in {time.monotonic() - started:.1f}s, {len(body)} bytes", status == 200)


def main():
    if not PASSWORD:
        sys.exit("set ASTRUM_PASSWORD (and ASTRUM_URL, ASTRUM_USER)")
    status, _, _ = call("GET", "/service/rest/v1/repositories")
    if status != 200:
        sys.exit(f"{BASE} did not accept the credentials ({status})")

    cleanup_policy("caches-unused-30-days", "*", 30, "Task outputs nobody restored for 30 days")

    repository("raw", "hosted", "raw-hosted", {"storage": storage(strict=False)})
    repository("npm", "hosted", "npm-hosted", {"storage": storage()})
    repository("npm", "proxy", "npm-proxy", proxy_parts("https://registry.npmjs.org"))
    repository(
        "npm",
        "group",
        "npm-group",
        {"storage": {"blobStoreName": STORE, "strictContentTypeValidation": True}, "group": {"memberNames": ["npm-hosted", "npm-proxy"]}},
    )
    repository("pypi", "hosted", "pypi-hosted", {"storage": storage()})
    repository("pypi", "proxy", "pypi-proxy", proxy_parts("https://pypi.org"))
    repository(
        "pypi",
        "group",
        "pypi-group",
        {"storage": {"blobStoreName": STORE, "strictContentTypeValidation": True}, "group": {"memberNames": ["pypi-hosted", "pypi-proxy"]}},
    )
    maven_release = {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT"}
    repository("maven", "hosted", "maven-releases", {"storage": storage(policy="allow_once"), "maven": maven_release})
    repository(
        "maven",
        "hosted",
        "maven-snapshots",
        {"storage": storage(), "maven": {"versionPolicy": "SNAPSHOT", "layoutPolicy": "STRICT"}},
    )
    repository("maven", "proxy", "maven-central", dict(proxy_parts("https://repo1.maven.org/maven2/"), maven=maven_release))
    repository(
        "maven",
        "group",
        "maven-public",
        {
            "storage": {"blobStoreName": STORE, "strictContentTypeValidation": True},
            "group": {"memberNames": ["maven-releases", "maven-snapshots", "maven-central"]},
        },
    )
    repository("helm", "hosted", "helm-hosted", {"storage": storage()})
    repository("nuget", "hosted", "nuget-hosted", {"storage": storage()})
    repository(
        "nuget",
        "proxy",
        "nuget.org-proxy",
        dict(proxy_parts("https://api.nuget.org/v3/index.json"), nugetProxy={"queryCacheItemMaxAge": 3600, "nugetVersion": "V3"}),
    )
    repository(
        "docker",
        "hosted",
        "docker-hosted",
        {"storage": storage(), "docker": {"v1Enabled": False, "forceBasicAuth": True}},
    )
    repository(
        "docker",
        "proxy",
        "docker-hub",
        dict(
            proxy_parts("https://registry-1.docker.io"),
            docker={"v1Enabled": False, "forceBasicAuth": True},
            dockerProxy={"indexType": "HUB"},
        ),
    )
    repository("go", "proxy", "go-proxy", proxy_parts("https://proxy.golang.org"))
    repository("cargo", "proxy", "crates-io", proxy_parts("https://index.crates.io/"))
    cleanup = {"policyNames": ["caches-unused-30-days"]}
    repository("turbo", "hosted", "turbo-cache", {"storage": storage(strict=False), "cleanup": cleanup})
    repository("nx", "hosted", "nx-cache", {"storage": storage(strict=False), "cleanup": cleanup})

    readme = "Demo files of the Astrum Repository test server.\n"
    upload(
        "raw-hosted",
        "docs and a release",
        {
            "raw.directory": "/demo",
            "raw.asset1.filename": "docs/readme.txt",
            "raw.asset2.filename": "releases/1.0.0/hello-1.0.0.tar.gz",
            "raw.asset3.filename": "config/settings.json",
        },
        {
            "raw.asset1": ("readme.txt", readme.encode()),
            "raw.asset2": ("hello-1.0.0.tar.gz", tar_gz({"hello/README": readme})),
            "raw.asset3": ("settings.json", json.dumps({"greeting": "hello", "enabled": True}, indent=2).encode()),
        },
    )
    for name, version, text in [
        ("@demo/hello", "1.0.0", "Says hello"),
        ("@demo/hello", "1.1.0", "Says hello, a little louder"),
        ("demo-utils", "0.1.0", "Small helpers for the demo"),
    ]:
        upload("npm-hosted", f"{name}@{version}", {}, {"npm.asset": (f"{name.replace('/', '-')}-{version}.tgz", npm_package(name, version, text))})
    for name, version, text in [("demo-hello", "1.0.0", "Says hello"), ("demo-hello", "1.1.0", "Says hello twice")]:
        filename, content = wheel(name, version, text)
        upload("pypi-hosted", f"{name} {version}", {}, {"pypi.asset": (filename, content)})
    for version in ["1.0.0", "1.1.0"]:
        upload(
            "maven-releases",
            f"com.example:demo-lib:{version}",
            {
                "maven2.groupId": "com.example",
                "maven2.artifactId": "demo-lib",
                "maven2.version": version,
                "maven2.generate-pom": "true",
                "maven2.packaging": "jar",
                "maven2.asset1.extension": "jar",
            },
            {"maven2.asset1": (f"demo-lib-{version}.jar", zip_bytes({"META-INF/MANIFEST.MF": "Manifest-Version: 1.0\n", "com/example/Demo.txt": version}))},
        )
    for version in ["0.1.0", "0.2.0"]:
        upload("helm-hosted", f"demo-app {version}", {}, {"helm.asset": (f"demo-app-{version}.tgz", helm_chart("demo-app", version, "A demo chart"))})
    upload("nuget-hosted", "Demo.Hello 1.0.0", {}, {"nuget.asset": ("Demo.Hello.1.0.0.nupkg", nuget_package("Demo.Hello", "1.0.0", "Says hello"))})
    for tag in ["1.0.0", "latest"]:
        docker_image("docker-hosted", "demo/hello", tag)

    for hash_value, team in [("aa11bb22cc33dd44", "web"), ("ee55ff6600778899", "docs")]:
        cache_artifact(
            "turbo-cache",
            f"/v8/artifacts/{hash_value}?slug={team}",
            tar_gz({"apps/web/dist/index.html": "<h1>demo</h1>\n"}),
            {"Content-Type": "application/octet-stream", "x-artifact-duration": "4200"},
        )
    cache_artifact(
        "nx-cache",
        "/v1/cache/1234567890123456789",
        tar_gz({"dist/apps/demo/main.js": "console.log('demo')\n", "terminalOutput": "> nx run demo:build\n"}),
        {"Content-Type": "application/octet-stream"},
    )

    role(
        "developers",
        "Read everything and publish to the hosted demo repositories",
        [
            "as-repository-view-*-*-browse",
            "as-repository-view-*-*-read",
            "as-repository-view-npm-npm-hosted-add",
            "as-repository-view-pypi-pypi-hosted-add",
            "as-repository-view-maven2-maven-snapshots-add",
            "as-repository-view-maven2-maven-snapshots-edit",
            "as-repository-view-turbo-turbo-cache-add",
            "as-repository-view-nx-nx-cache-add",
        ],
    )

    warm("npm-group: lodash", "/repository/npm-group/lodash")
    warm("pypi-group: requests", "/repository/pypi-group/simple/requests/")
    warm("maven-public: junit 4.13.2 pom", "/repository/maven-public/junit/junit/4.13.2/junit-4.13.2.pom")
    warm("go-proxy: golang.org/x/text", "/repository/go-proxy/golang.org/x/text/@v/list")
    warm("nuget.org-proxy: index", "/repository/nuget.org-proxy/index.json")

    failed = [entry for entry in REPORT if not entry[3]]
    print(f"\n{len(REPORT) - len(failed)} done, {len(failed)} not done")


if __name__ == "__main__":
    main()
