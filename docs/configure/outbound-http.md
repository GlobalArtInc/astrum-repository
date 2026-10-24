# Outbound HTTP and proxy

Proxy repositories fetch content from their remotes over HTTP. **Settings → System → HTTP** (the page
**HTTP Settings**) sets the `User-Agent`, the timeout and the retries of those requests and the proxy
server they go through. The settings are stored in the database, so they need no restart: the server
that saves them applies them at once, the other servers on the same database within ten seconds.
Changes are recorded in the audit log. The page needs the privilege `as-settings-read` to read it and
`as-settings-update` to change it.

The server does not read the `HTTP_PROXY` and `HTTPS_PROXY` environment variables; see
[runtime configuration](runtime.md#outbound-http-proxy).

## Connection settings

| Setting | Default | Meaning |
|---|---|---|
| **User-Agent Suffix** | none | Text added to the `User-Agent` header, which always starts with `Astrum/<version>` |
| **Connection Timeout** | `60` | Seconds, 1 to 3600, to wait for a connection and for each read from the remote |
| **Connection Retries** | `2` | Further attempts, 0 to 10, after a connection fails or times out |

A request is retried only when the remote could not be reached or did not answer in time. An HTTP error
from the remote, such as `404` or `503`, is not retried.

Each proxy repository has the same three fields in its **HTTP** section, see
[proxy repositories](../repositories/overview.md#proxy-repositories). A repository's
**Connection Retries** and **Connection Timeout** replace the values of this page for that repository;
its **User-Agent Suffix** is added after the one of this page, so a repository with the suffix
`maven-central` on a server with the suffix `ExampleCorp` sends `Astrum/3.0.1 ExampleCorp maven-central`.

## Proxy server

The page has two proxies, **HTTP Proxy** and **HTTPS Proxy**. Each is turned on with
**Enable HTTP Proxy** or **Enable HTTPS Proxy** and then needs a host and a port:

| Setting | Meaning |
|---|---|
| **HTTP Proxy Host**, **HTTPS Proxy Host** | Host name or IP address of the proxy, without a scheme, port or path; an IPv6 address may be written with or without brackets |
| **HTTP Proxy Port**, **HTTPS Proxy Port** | Port of the proxy, 1 to 65535 |

Which proxy a request uses depends on the scheme of the remote URL:

| Request to | Goes through |
|---|---|
| `http://...` | The HTTP proxy, if it is enabled |
| `https://...` | The HTTPS proxy; when only the HTTP proxy is enabled, the HTTP proxy |

Both proxies are reached in plain HTTP at `http://<host>:<port>`. An `https://` request passes through
the proxy as a `CONNECT` tunnel, so TLS runs between Astrum and the remote. A proxy that inspects TLS
presents certificates signed by a CA of its own: add that CA under
[SSL certificates](ssl-certificates.md) and turn on **Use the Astrum Repository Truststore** in the
proxy repositories.

### Proxy authentication

Under **HTTP Proxy Authentication** or **HTTPS Proxy Authentication**, set **Authentication** to
**Username** and fill in **Username** and **Password**. Astrum sends them to the proxy with HTTP Basic
authentication. **NTLM Host** and **NTLM Domain** are saved but not used: NTLM authentication to the
proxy is not supported.

The password is stored sealed, see the [secrets key](runtime.md#secrets-key). Leave the field empty to
keep the stored password; turning authentication off forgets it.

### Hosts to exclude from the proxy

**Hosts to Exclude from Proxy** appears once a proxy is enabled. Requests to these hosts go straight to
the remote. Enter a host name or a pattern under **Host Pattern** and click **Add**.

- `*` stands for any text: `*.example.com` matches `repo.example.com` but not `example.com`, so add
  both when both are needed.
- The comparison ignores case and a trailing dot, and uses the host as it is written in the URL:
  `10.*` matches `http://10.1.2.3/`, not a host name that resolves to `10.1.2.3`.
- `*` alone sends every request past the proxy.
- A pattern contains no spaces, slashes or commas.

## Authentication to the remote

The proxy credentials of this page are sent only to the proxy. The **Authentication Type** in the
**HTTP** section of a proxy repository (**Username** or **Bearer token**) is for the remote itself: those
credentials are sent only to the scheme, host and port of the repository's **Remote Storage** URL, never
to the proxy and never to another host a redirect leads to.

All proxy repositories share the proxy of this page; a repository cannot have a proxy of its own. To let
one repository reach its remote directly, add the remote's host to **Hosts to Exclude from Proxy**.

## What uses these settings

Only proxy repositories use the connection and proxy settings of this page. The other connections the
server opens go straight to their target:

- OpenID Connect, LDAP and the email server;
- S3 blob stores and the agents of remote blob stores;
- notification channels. A channel's own **Connection** decides: **Through a proxy** uses the proxy
  entered in the channel, while **System HTTP settings** and **Direct** both connect directly in this
  version.

## SSRF protection

Requests of proxy repositories are subject to the SSRF protection, which refuses loopback, private and
other internal addresses unless they are allowed; see
[runtime configuration](runtime.md#outbound-http-proxy) for the page that lists the exceptions. With a
proxy:

- the proxy host is always allowed, even when it has a private address;
- Astrum still resolves the remote's host name itself and refuses the request when the name resolves to
  a blocked address; a name that Astrum cannot resolve at all is left to the proxy.

The same allowed addresses and domains govern Slack and webhook notification channels.

## REST API

`GET /service/rest/v1/http` returns the settings and `PUT /service/rest/v1/http` replaces them; a field
that is left out takes its default. Stored proxy passwords come back as
`#~ASTRUM~PLACEHOLDER~PASSWORD~#`; sending that value, an empty string or `null` keeps the stored
password.

```sh
curl -u "admin:$ASTRUM_PASSWORD" -X PUT -H "Content-Type: application/json" \
  https://astrum.example.com/service/rest/v1/http -d '{
    "userAgent": "ExampleCorp",
    "timeout": 30,
    "retries": 3,
    "httpProxy": {
      "enabled": true,
      "host": "proxy.example.com",
      "port": 3128,
      "authInfo": {"enabled": true, "username": "astrum", "password": "change-me"}
    },
    "httpsProxy": {"enabled": false},
    "nonProxyHosts": ["localhost", "*.example.com"]
  }'
```

Problems are answered with `400` and the field they concern, for example `PARAMETER timeout` or
`httpProxy.port`.
