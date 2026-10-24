# Chromium extensions

Repositories of the format `chrome` serve extensions to Google Chrome, Chromium, Microsoft Edge, Brave,
Vivaldi and the other browsers built on Chromium. These browsers install and update extensions from an
*update URL* that speaks the update protocol of the Chrome Web Store: the browser asks
`<update URL>?response=updatecheck&prodversion=<browser version>&x=id%3D<id>%26v%3D<version>%26uc`, with one
`x` per extension, and the XML answer names the newest version of each extension, the address of its CRX
package, its SHA-256 and its size. A hosted repository keeps CRX3 packages that you upload; a proxy caches the
Chrome Web Store (remote URL `https://clients2.google.com/service/update2/crx`), Edge Add-ons
(`https://edge.microsoft.com/extensionwebstorebase/v1/crx`) or any other server that speaks the protocol,
such as a repository of another Astrum server; a group asks its members in order and takes each extension from
the first member that knows it.

The update URL of a repository is

```text
https://astrum.example.com/repository/chrome-group/updates.xml
```

and the repository itself answers the same query, so `https://astrum.example.com/repository/chrome-group` works
as an update URL too. Every repository serves:

- `updates.xml?response=updatecheck&…` (or the repository root with that query): the update answer. An
  extension the repository has is offered with its newest version, a `codebase` that points at this
  repository, `hash_sha256` and `size`; `status="noupdate"` answers a browser that already has that version or
  a newer one; an extension the repository does not know, or that the user may not read, gets
  `status="error-unknownApplication"`.
- `updates.xml?response=redirect&acceptformat=crx3&x=id%3D<id>%26uc`: `302 Found` to the package of the
  newest version, the form that direct download links use.
- `<id>/latest.crx`: `302 Found` to the package of the newest version.
- `<id>/<version>/<id>-<version>.crx`: the package itself (`application/x-chrome-extension`).

Browsers send no credentials with update checks and downloads, so let the anonymous user read the
repositories they use (see [anonymous access](../configure/authentication.md#anonymous-access)), or let only
your network reach them.

## Browser policy

Extensions from an update URL outside the stores are installed through browser policy. The
`ExtensionSettings` policy force-installs an extension and keeps it updated from the repository. On Linux,
save it as a JSON file in the managed policy directory, `/etc/opt/chrome/policies/managed/astrum.json` for
Chrome (`/etc/chromium/policies/managed/` for Chromium, `/etc/opt/edge/policies/managed/` for Edge,
`/etc/brave/policies/managed/` for Brave), readable by every user and writable only by root:

```json
{
  "ExtensionSettings": {
    "abcdefghijklmnopabcdefghijklmnop": {
      "installation_mode": "force_installed",
      "update_url": "https://astrum.example.com/repository/chrome-group/updates.xml",
      "override_update_url": true
    }
  }
}
```

Open `chrome://policy` (`edge://policy`, `brave://policy`) and click **Reload policies**, or restart the
browser. `installation_mode` is `force_installed` (installed, and the user cannot remove or disable it) or
`normal_installed` (installed, and the user can disable it). Without `override_update_url` the browser uses the
update URL of the policy only for the first installation and then follows the `update_url` of the extension's
manifest; every extension from the Chrome Web Store names the Web Store there, so a proxy of the Web Store in a
network without internet access needs `"override_update_url": true`. An entry
`"*": {"installation_mode": "blocked"}` in the same policy blocks every extension that the policy does not
list.

The older `ExtensionInstallForcelist` policy does the same with one string per extension, the id and the update
URL separated by a semicolon; it never overrides the update URL of the manifest:

```json
{
  "ExtensionInstallForcelist": [
    "abcdefghijklmnopabcdefghijklmnop;https://astrum.example.com/repository/chrome-group/updates.xml"
  ]
}
```

On Windows the policies live in the registry, set by Group Policy with the administrative templates of the
browser or directly. `ExtensionInstallForcelist` is a key with one string value per extension, named `1`, `2`
and so on; `ExtensionSettings` is a string value that holds the JSON on one line:

```shell
reg add HKLM\SOFTWARE\Policies\Google\Chrome\ExtensionInstallForcelist /v 1 /t REG_SZ /d "abcdefghijklmnopabcdefghijklmnop;https://astrum.example.com/repository/chrome-group/updates.xml" /f
```

Edge reads `HKLM\SOFTWARE\Policies\Microsoft\Edge`, Brave `HKLM\SOFTWARE\Policies\BraveSoftware\Brave` and
Chromium `HKLM\SOFTWARE\Policies\Chromium`. On macOS the policies come in a configuration profile installed by
MDM, with the payload type `com.google.Chrome` (`com.microsoft.Edge` for Edge, `com.brave.Browser` for Brave)
and `ExtensionSettings` as a dictionary:

```xml
<key>ExtensionSettings</key>
<dict>
  <key>abcdefghijklmnopabcdefghijklmnop</key>
  <dict>
    <key>installation_mode</key>
    <string>force_installed</string>
    <key>update_url</key>
    <string>https://astrum.example.com/repository/chrome-group/updates.xml</string>
    <key>override_update_url</key>
    <true/>
  </dict>
</dict>
```

On Windows and macOS, Chrome and Edge install extensions from an update URL outside their stores only on
machines that are joined to a domain, managed by MDM or enrolled in the cloud management of the browser;
Linux has no such condition. `chrome://extensions` with **Developer mode** on has an **Update** button that
checks for updates at once.

## Publishing extensions

A hosted repository takes CRX3 packages. Pack the extension with **Pack extension** on `chrome://extensions`
or on the command line, and keep the private key: the id of the extension comes from it, so every version has
to be packed with the same key.

```shell
chrome --pack-extension=acme-extension --pack-extension-key=acme-extension.pem
curl -i -X PUT -H "Authorization: Bearer $ASTRUM_TOKEN" -T acme-extension.crx \
  https://astrum.example.com/repository/chrome-hosted/acme-extension.crx
```

Any path that ends in `.crx` will do, and the form of the upload page and the components API take the file in
the field `chrome.asset`:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F chrome.asset=@acme-extension.crx \
  "https://astrum.example.com/service/rest/v1/components?repository=chrome-hosted"
```

The server reads the id from the CRX3 header (it has to belong to one of the RSA keys in the header) and the
version, name, description, `manifest_version` and `update_url` from `manifest.json`, resolving names such as
`__MSG_appName__` from `_locales/<default_locale>/messages.json`. It stores the package as
`<id>/<version>/<id>-<version>.crx`, whatever path it was put to, as a component named by the id, with the
attributes `chrome.id`, `chrome.name`, `chrome.version` and `chrome.manifestVersion`, and answers `201 Created`
with the id, name, version and address. Unless the write policy allows redeploying, a version that is already
there is refused with 409. CRX2 packages, which Chromium no longer installs, and files that are not CRX
packages are refused with 400. The server does not check the signatures of a package (the browser checks them
when it installs it) and never signs anything again. Search finds extensions by `chrome.name` and
`chrome.manifestVersion`, and by keywords in their name and description.

The `update_url` in the manifest is not used by the server. If you rely on `ExtensionInstallForcelist`, or on
`ExtensionSettings` without `override_update_url`, set it to the update URL of the repository so that updates
come from there.

## Proxies and groups

A proxy asks its remote for the newest version of each extension a browser checks (with `v=0.0.0.0` and
`acceptformat=crx3`, the `prodversion` of the browser or `156.0.0.0` when it sent none, and up to 50
extensions in a request) and keeps the answer of each extension as metadata, reused for the metadata maximum
age (24 hours by default). An extension the remote does not know is remembered for the time to live of the
negative cache. In the answer it gives, every `codebase` points at the proxy, at
`<id>/<version>/<id>-<version>.crx`, and the remote address is kept with the cached answer. The first
download of a version fetches that address (following redirects, such as the one of the Chrome Web Store to
its download host), checks it against the `hash_sha256` and `size` of the remote answer and checks that it is a
CRX3 package of that id and version, then caches it as a component; later downloads are served from the cache.
The addresses of the last 16 versions of an extension are kept, so a browser that is still offered an older
version can download it. When the remote cannot be reached or is blocked, the cached answers are served; an
update check that has nothing cached fails with 502. The SSRF protection and the HTTP settings of the
repository apply to the remote and to every download address it names.

A group asks its members in order with the extensions no member before has answered, so a hosted member wins
over a proxy for the same id; its answers and redirects point at the group, and a package comes from the first
member that has it. Proxies and groups take no uploads.

Extension versions are numbers only, so cleanup policies have no release type criterion for this format; the
other criteria work as for every format. When a component is deleted, by hand or by a cleanup policy, the next
update check offers the newest version that is left.
