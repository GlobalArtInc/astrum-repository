# Reverse proxy and TLS

Run the server on plain HTTP behind a reverse proxy that terminates TLS. The proxy must:

- pass the original `Host` header and set `X-Forwarded-Proto: https`, so generated links, Docker auth
  challenges and repository indexes point at the public address;
- accept request bodies of any size and stream them instead of buffering them: Docker layers, Maven
  bundles and Helm charts can be several gigabytes;
- allow long requests: uploads and the first download of a large artifact through a proxy repository
  can take minutes.

If the proxy cannot forward the host and scheme, set the base URL explicitly; see
[base URL](runtime.md#base-url).

## nginx

```nginx
server {
    listen 443 ssl;
    http2 on;
    server_name astrum.example.com;

    ssl_certificate     /etc/ssl/astrum/fullchain.pem;
    ssl_certificate_key /etc/ssl/astrum/privkey.pem;

    client_max_body_size 0;
    proxy_request_buffering off;
    proxy_buffering off;
    proxy_read_timeout 600s;
    proxy_send_timeout 600s;

    location / {
        proxy_pass http://127.0.0.1:8081;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name astrum.example.com;
    return 301 https://$host$request_uri;
}
```

## Kubernetes Gateway API

When the server runs on a VM next to a cluster, expose it to the gateway through a Service without a
selector and an EndpointSlice, then route to it:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: astrum
  namespace: astrum
spec:
  ports:
    - name: http
      port: 8081
      targetPort: 8081
---
apiVersion: discovery.k8s.io/v1
kind: EndpointSlice
metadata:
  name: astrum
  namespace: astrum
  labels:
    kubernetes.io/service-name: astrum
addressType: IPv4
ports:
  - name: http
    port: 8081
    protocol: TCP
endpoints:
  - addresses: ["10.0.10.20"]
---
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: astrum
  namespace: astrum
spec:
  hostnames: ["astrum.example.com"]
  parentRefs:
    - name: public-gateway
      namespace: gateway-system
      sectionName: https
  rules:
    - matches:
        - path: {type: PathPrefix, value: /}
      timeouts:
        request: 600s
      filters:
        - type: RequestHeaderModifier
          requestHeaderModifier:
            set:
              - name: X-Forwarded-Proto
                value: https
      backendRefs:
        - name: astrum
          port: 8081
```

Check the gateway's own limits on request body size and idle timeouts as well; Envoy, for example,
has a default stream idle timeout that a slow upload can hit.

## Docker repositories

Docker clients always talk to `/v2/` at the root of a host. There are three ways to reach a Docker
repository; pick one per repository in its settings (**Routing Mode**).

### Path routing (recommended)

Choose **Use path routing**. The repository name becomes the first segment of the image name, and no
extra port or host is needed:

```shell
docker login astrum.example.com
docker pull astrum.example.com/docker-hosted/team/app:1.0
docker push astrum.example.com/docker-hosted/team/app:1.1
```

The reverse proxy only has to forward `/v2/` like any other path.

### Subdomain

Choose **Use connectors** and set **Subdomain** on the repository, for example `docker-a`. Point
`docker-a.astrum.example.com` at the proxy (a wildcard DNS record and certificate cover all
repositories) and forward the original `Host` (or send it as `X-Forwarded-Host`): the server routes `/v2/`
on a host that starts with the subdomain to that repository.

```shell
docker pull docker-a.astrum.example.com/team/app:1.0
```

### Dedicated port

Choose **Use connectors** and set the **HTTP Connector** port on the repository (for example 8082),
publish that port from the container and give it its own proxy server block or host. The server opens
a listener on that port within ten seconds of the change, on every server of an installation. This is
only needed for clients that cannot use the other two.

An **HTTPS Connector** port serves TLS itself, for installations without a reverse proxy. It needs a
certificate and key in PEM format, given to the server with `connectors.tls_certificate` and
`connectors.tls_key` (see [runtime configuration](runtime.md#configuration-file)); one pair serves all
HTTPS connectors.

## A domain of its own for a repository

Any repository can also be reached on a domain of its own, set under **Domain** in its settings (or as
the `domain` section of the repository over the REST API). Its address under `/repository/<name>/` on
the server keeps working. The domain must point to the proxy, and the proxy must send its requests to
the server with the original `Host` (or `X-Forwarded-Host`) and `X-Forwarded-Proto`, as in the examples
above; the server picks the repository from the host. While
a request comes in on a repository domain, the links the server writes (Docker token realms, npm
tarballs, index pages) use that domain.

There are two ways to use a domain:

- **Whole domain** (`ROOT`): the domain belongs to one repository and serves it from its root.
  `docker-prem.example.com` for the Docker repository `docker-onprem`:

  ```shell
  docker login docker-prem.example.com
  docker pull docker-prem.example.com/team/app:1.0
  ```

  For other formats the domain root is the repository URL: `https://files.example.com/tools/1.0/install.sh`
  is `/repository/files/tools/1.0/install.sh`.

- **Path on a shared domain** (`PATH`): several repositories share the domain, each under its own first
  path segment, which is the repository name unless **Path** says otherwise. With `images.example.com`,
  `docker-onprem` under the path `prod` and `docker-dev` without a path:

  ```shell
  docker pull images.example.com/prod/team/app:1.0
  docker pull images.example.com/docker-dev/team/tool:2.0
  ```

  The catalogue of the domain (`/v2/_catalog`) lists the images of its Docker repositories under their
  paths.

The settings refuse a domain that another repository uses as a whole domain, a whole domain on a
domain that other repositories share, a path that is already taken on the same domain, the server's own
host, and paths `v2`, `repository` and `service`. Before saving, the form looks up the A record of the
domain and says whether it points to this server; the repository is saved either way, so a domain can be
set up before its DNS. On a repository domain, `/service/` is still the REST API, and other repositories
are not reachable.

Over REST the section reads:

```json
"domain": { "host": "images.example.com", "routing": "PATH", "path": "prod" }
```

## Serving a repository under another host or path

A proxy can expose a single repository under a product-specific address by rewriting the path, for
example `https://downloads.example.com/tool/` to `/repository/tool-releases/`:

```nginx
location /tool/ {
    proxy_pass http://127.0.0.1:8081/repository/tool-releases/;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

This suits raw downloads and formats whose indexes use relative links. Metadata that carries absolute
links, such as npm package documents, keeps pointing at `/repository/<name>/` on the server's own
address, so test a client against the rewritten address before publishing it.
