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
repository; pick one per repository in its settings.

### Path-based routing (recommended)

Enable **Path-based routing** on the repository. The repository name becomes the first segment of the
image name, and no extra port or host is needed:

```shell
docker login astrum.example.com
docker pull astrum.example.com/docker-hosted/team/app:1.0
docker push astrum.example.com/docker-hosted/team/app:1.1
```

The reverse proxy only has to forward `/v2/` like any other path.

### Subdomain

Set **Subdomain** on the repository, for example `docker-a`, point `docker-a.astrum.example.com` at
the proxy (a wildcard DNS record and certificate cover all repositories) and forward the original
`Host`:

```shell
docker pull docker-a.astrum.example.com/team/app:1.0
```

### Dedicated port

Set an HTTP connector port on the repository (for example 8082), publish that port from the container
and give it its own proxy server block or host. This is only needed for clients that cannot use the
other two.

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
