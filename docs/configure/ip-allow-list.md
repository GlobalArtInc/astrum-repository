# IP allow list

The IP allow list limits which client addresses may reach the server, one list for each part of it. It is
global and allow-only: an address that is not in a list cannot reach that part, and an **empty list means
that part is open to every address**. All three lists are empty after installation.

Administrators edit the lists at **Settings → Security → IP allow list** (the page `admin/security/ip-allow`),
which needs the administrator right. Agents can read them through the [`get_ip_allow_list` tool](../use/mcp.md).

## The three lists

| List | Surface | What it covers |
|---|---|---|
| **Management** | `management` | Everything that is not in the two lists below: the web interface, the REST API, Prometheus metrics, the browse routes of the web interface, and `GET /service/rest/v1/search/assets/download` |
| **Content** | `content` | Repository downloads and uploads: `/repository/**`, the content API, Docker `/v2/**` and the dedicated Docker port, and repository domains. Also `POST` of components |
| **Agents** | `agents` | `/mcp` and the OAuth paths: `/oauth/**` and `/.well-known/oauth-*` |

A few edges matter when you choose the lists:

- The status endpoint is never restricted, so health probes keep working. It is `GET` or `HEAD` of
  `/service/rest/v1/status` and of `/service/rest/v1/status/writable`.
- The browse routes of the web interface serve file bytes but count as management. An address that is allowed
  only in the content list gets `403` there.
- A person who authorises an agent in the browser goes through the OAuth paths, so that address needs an entry
  in the agents list.
- The path is judged after percent-decoding it once. A path that does not decode counts as management.
- A request on a repository domain is judged by the path it resolves to on the server.

An entry is an IPv4 or IPv6 address (`10.1.1.1`, `fd00::1`) or a range in CIDR form (`10.0.0.0/8`, `fd00::/8`).
Each list takes at most 256 entries. A range that covers a whole address family (`0.0.0.0/0`, `::/0`) is refused:
clear the list to allow every address.

## Your address and trusted proxies

A request is judged by the address of its TCP peer. The only exception is a peer that is one of the trusted
proxies: then the address is taken from its `X-Forwarded-For` header.

The **Your address** panel on the page shows the address the server sees for your browser, the peer the
connection came from, and the trusted proxies. It shows a warning when a request carried a forwarded address
that was ignored, because its peer is not a trusted proxy.

The address is only as trustworthy as the proxy list. The server reads the header from the right and stops at
the first address that is not a trusted proxy. So a listed proxy must append the address it sees to the header,
or overwrite it. A proxy that forwards the header unchanged lets the client choose its own address. Put only
the proxies you run in the list. The setting is `ASTRUM_TRUSTED_PROXIES`, or `trusted_proxies` in the
configuration file; see [trusted proxies](reverse-proxy.md#trusted-proxies).

## Lockout protection

A management list that is not empty must contain the address you are using now. The server refuses to save
it otherwise, and the error is shown on the management list. The server also refuses a non-empty management
list when it does not know your address. An empty management list opens the management interface to every
address, so add your own address before you add anything else to it.

## Changes on other nodes

A saved list reaches every other node on the same database within about two seconds. No restart is needed,
and each node keeps the parsed lists in memory, so the check costs nothing per request.

## Break-glass

If you lock yourself out, for example because your address changed behind a NAT, stop the server and run:

```sh
astrum ip-allow reset
```

The command reads the normal configuration and the database directly, so it works even when the stored
value is corrupt. It clears all three lists, bumps the security revision and audits the change as made by
`*CLI`. The nodes open within two seconds, without a restart.

If the stored lists cannot be read at start, for example after a manual edit of the database, the server stops
and the log says:

```
the stored IP allow list is invalid: ...; stop the server and run `astrum ip-allow reset` to clear it
```

## Denials and audit

A denied request gets `403 Forbidden` with the message `Access from this address is not allowed`. The
checks run before sign-in, so a denied caller costs no password check, and a valid token or an OAuth grant
from a denied address is refused too.

- Every change of a list is audited as `security.ipAllow` / `updated`, with the lists before and after.
- A denial is audited as `security.ipAllow` / `denied`, with the address, the surface and the path. Each
  node audits at most one denial per address and surface every ten minutes.
- The Prometheus counter `astrum_ip_allow_denied_total{surface}` counts every denial, so use it for the rate.
  See [monitoring](../operate/monitoring.md).
