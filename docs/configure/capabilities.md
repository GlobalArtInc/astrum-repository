# Capabilities

A capability is an optional feature with settings of its own, turned on and off without a restart. They
are managed under **Settings → System → Capabilities**, or with the REST API under
`/service/rest/v1/capabilities`. A server has at most one capability of each type. Capabilities are
stored in the database and apply to every server on it.

## Capability types

| Type | ID | Category | What it does |
|---|---|---|---|
| **Base URL** | `baseurl` | Core | Sets the address that links written by the server start with; see [base URL](runtime.md#base-url) |
| **UI: Settings** | `rapture.settings` | UI | Sets the title of the browser tab and the session timeout |
| **UI: Branding** | `rapture.branding` | UI | Shows your own HTML above and below every page of the web interface |
| **Metrics: Prometheus** | `metrics.prometheus` | Monitoring | Serves metrics at `/service/metrics/prometheus`; see [Prometheus metrics](../operate/monitoring.md#prometheus-metrics) |
| **Blob Stores: Space Alert** | `blobstores.space` | Monitoring | Sends free space, quota and reachability problems of blob stores to notification channels; see [blob store space alerts](../operate/monitoring.md#blob-store-space-alerts) |

These are all the types this server has. Other optional features have pages of their own: the role
every signed-in user gets is set under **Settings → Security → Default Role**, notifications under
**Settings → System → Notifications**, the mail server under **Settings → System → Email Server**.

### Base URL

**URL** is the address of the server as users see it, such as `https://astrum.example.com`: an `http` or
`https` URL without user name, password, query or fragment. A trailing slash is removed. While the
capability is disabled or absent, links are built from each request.

### UI: Settings

This capability exists from the start with its defaults; open it to change them rather than creating it.

| Setting | Default | Meaning |
|---|---|---|
| **Title** | `Astrum Repository` | The title of the browser tab, at most 200 characters |
| **Session timeout** | `0` | Minutes of inactivity after which a web session ends; `0` keeps sessions open, see [sessions](runtime.md#sessions) |

**Debug allowed**, the two status intervals and the two request timeouts are stored but have no effect
in this version. Disabling the capability brings back the default title; the session timeout stays in
force.

### UI: Branding

| Setting | Meaning |
|---|---|
| **Enable header**, **Header HTML snippet** | HTML shown above every page |
| **Enable footer**, **Footer HTML snippet** | HTML shown below every page |

Each snippet may be up to 64 KiB, and `$baseUrl` in it is replaced by the base URL of the server, for
example to load an image from a raw repository. The browser cleans the snippet before showing it:
scripts, style sheets, frames, forms and SVG are removed, and what stays are text formatting, headings,
lists, tables, `div` and `span`, links and images with `http`, `https` or `mailto` addresses (images also
as `data:` PNG, GIF, JPEG or WebP), and the attributes `class`, `style`, `title`, `alt`, `align`, `width`,
`height`, `target` and `rel`.

## Adding and changing a capability

1. Click **Create Capability** and pick a type under **Select a Type**. Types that exist already are marked
   **Already set up**.
2. Click **Continue**, fill in the settings, leave **Enable this capability** checked or clear it, and add
   **Notes** if you like.
3. Click **Create**.

The list shows **Type**, **Description** and **Notes** of every capability and can be filtered by
**State** (**Active** or **Disabled**) and **Category**. Open a capability to change it on the
**Settings** tab; **Summary** shows its description, notes and category, **Status** what it reports (the
metrics address, the last space check) and **About** what the type does.

**Disable** turns a capability off and keeps its settings; **Enable** turns it on again. **Delete** removes
it with its settings. The privileges are `as-capabilities-read`, `as-capabilities-create`,
`as-capabilities-update` and `as-capabilities-delete`; `as-admin` has them all.

## REST API

| Request | Does |
|---|---|
| `GET /service/rest/v1/capabilities` | Lists the capabilities that exist |
| `GET /service/rest/v1/capabilities/types` | Lists the types with their settings |
| `POST /service/rest/v1/capabilities` | Creates a capability; refused with `400` when one of the type exists |
| `GET`, `PUT`, `DELETE /service/rest/v1/capabilities/<ID>` | Reads, replaces or deletes the capability of that type |

A capability has `type`, `enabled`, `notes` and `properties`, whose values are strings. `PUT` replaces the
whole capability: send every property, `enabled` (true when left out) and `notes` (empty when left out),
or the missing ones fall back to their defaults.

```sh
curl -u "admin:$ASTRUM_PASSWORD" -X POST -H "Content-Type: application/json" \
  https://astrum.example.com/service/rest/v1/capabilities -d '{
    "type": "baseurl",
    "enabled": true,
    "properties": {"url": "https://astrum.example.com"}
  }'

curl -u "admin:$ASTRUM_PASSWORD" -X PUT -H "Content-Type: application/json" \
  https://astrum.example.com/service/rest/v1/capabilities/metrics.prometheus -d '{
    "enabled": false,
    "notes": "scraped by the new monitoring only",
    "properties": {"repositories": "true"}
  }'
```
