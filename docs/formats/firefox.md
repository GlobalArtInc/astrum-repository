# Firefox add-ons

Repositories of the format `firefox` keep Firefox add-ons (`.xpi` files: zip archives with a `manifest.json`)
and serve them to the Firefox installations an administrator manages with enterprise policies. A hosted
repository holds your own add-ons and writes the update manifests Firefox asks for; a proxy caches add-ons
of addons.mozilla.org through its API (`https://addons.mozilla.org` is the remote URL); a group merges its
members, each add-on coming from the first member that knows it.

Release and Beta builds of Firefox install only add-ons signed by Mozilla, that is listed on
addons.mozilla.org or signed there as unlisted for distribution of your own, and this holds for add-ons
installed through policies too. ESR, Developer Edition and Nightly builds can turn the check off with the
preference `xpinstall.signatures.required` set to `false`. The server neither signs nor re-signs: it stores
each file as it is, takes signed and unsigned files alike, and records in the attribute `firefox.signed`
whether a file carries a signature (`META-INF/mozilla.rsa` or `META-INF/cose.sig`). Sign an add-on of your
own as unlisted (`web-ext sign --channel=unlisted`) before you upload it for Release builds.

Put an `.xpi` to any path of a hosted repository that ends in `.xpi`, with an
[access token](../configure/authentication.md#access-tokens) or a password, or send it through the upload
form or the components API in the field `firefox.asset`:

```shell
curl -i -X PUT -H "Authorization: Bearer $ASTRUM_TOKEN" -T web-ext-artifacts/tool-1.2.3.xpi \
  https://astrum.example.com/repository/firefox-hosted/tool-1.2.3.xpi
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F firefox.asset=@tool-1.2.3.xpi \
  "https://astrum.example.com/service/rest/v1/components?repository=firefox-hosted"
```

The server reads `manifest.json` for the id (`browser_specific_settings.gecko.id`, or `applications.gecko.id`
in older add-ons; in a signed file that has neither, the id its signing certificate names), the version,
the name (a `__MSG_<key>__` name is looked up in `_locales/<default_locale>/messages.json`) and
`strict_min_version` and `strict_max_version`. Ids look like `tool@example.org` or a UUID in braces,
`{11111111-2222-3333-4444-555555555555}`. The file is stored as `<id>/<version>/<name>-<version>.xpi`,
where `<name>` is the part of the id before `@` or the UUID without braces, whatever path it was put to.
Putting the same file again changes nothing; another file for a version that is already there is refused
with 409 unless the write policy allows redeploying. Cleanup policies count versions with letters, such
as `1.0b2` or `2.0pre1`, as pre-releases.

Firefox reads the policies from `policies.json`:

- Linux: `/etc/firefox/policies/policies.json`, or `distribution/policies.json` in the installation
  directory, such as `/usr/lib/firefox/distribution/policies.json` (distribution packages may install
  Firefox elsewhere);
- Windows: `distribution\policies.json` in the installation directory, such as
  `C:\Program Files\Mozilla Firefox\distribution\policies.json`; Group Policy with the Firefox ADMX
  templates sets the same policies;
- macOS: `/Applications/Firefox.app/Contents/Resources/distribution/policies.json`; a configuration
  profile sets the same policies.

`about:policies` shows what Firefox has read. The `ExtensionSettings` policy installs an add-on from the
repository; `<id>/latest.xpi` downloads its newest version, and `<id>/<version>.xpi` a given one:

```json
{
  "policies": {
    "ExtensionSettings": {
      "tool@example.org": {
        "installation_mode": "force_installed",
        "install_url": "https://astrum.example.com/repository/firefox-group/tool@example.org/latest.xpi"
      }
    }
  }
}
```

In addresses the braces of a UUID id are written as `%7B` and `%7D`
(`.../firefox-group/%7B11111111-2222-3333-4444-555555555555%7D/latest.xpi`), while the key in
`ExtensionSettings` keeps them. `normal_installed` instead of `force_installed` lets users disable the
add-on.

Firefox looks for the updates of an add-on at the `update_url` of its manifest. The author of an add-on
that is distributed this way sets it, as an `https` address, before signing:

```json
{
  "browser_specific_settings": {
    "gecko": {
      "id": "tool@example.org",
      "update_url": "https://astrum.example.com/repository/firefox-group/tool@example.org/updates.json"
    }
  }
}
```

The repository answers with an update manifest that lists every version with its `update_link`, its
`update_hash` (`sha256:<hex>`) and the `strict_min_version` and `strict_max_version` of its manifest, and
Firefox installs the newest version that fits. `updates.json` at the root of a hosted repository or a group
lists all its add-ons, and `updates.json?id=<id>` one of them.

Add-ons from addons.mozilla.org have no `update_url`: Firefox asks the address in the preference
`extensions.update.url` for them, and `extensions.update.background.url` in its background checks. Both
point at addons.mozilla.org by default. To take their updates from a proxy, or from a group that holds one,
set both through the `Preferences` policy; Firefox fills in `%ITEM_ID%` and the other placeholders. The
install address takes the slug of the add-on on addons.mozilla.org as well as its id:

```json
{
  "policies": {
    "ExtensionSettings": {
      "uBlock0@raymondhill.net": {
        "installation_mode": "force_installed",
        "install_url": "https://astrum.example.com/repository/firefox-group/ublock-origin/latest.xpi"
      }
    },
    "Preferences": {
      "extensions.update.url": {
        "Value": "https://astrum.example.com/repository/firefox-group/update/VersionCheck.php?id=%ITEM_ID%&version=%ITEM_VERSION%&appVersion=%APP_VERSION%",
        "Status": "locked"
      },
      "extensions.update.background.url": {
        "Value": "https://astrum.example.com/repository/firefox-group/update/VersionCheck.php?id=%ITEM_ID%&version=%ITEM_VERSION%&appVersion=%APP_VERSION%",
        "Status": "locked"
      }
    }
  }
}
```

`update/VersionCheck.php?id=<id>` answers the same update manifest as `<id>/updates.json`, and an empty one,
`{"addons": {}}`, for an add-on the repository does not know. Firefox downloads policy add-ons and checks
for updates without credentials, so a repository it reads should be readable anonymously: keep anonymous
access on and narrow `as-anonymous` to that repository, or let only your network reach it.

A proxy looks up `<slug or id>/latest.xpi` and `<slug or id>/<version>.xpi` with
`api/v5/addons/addon/<slug or id>/` of its remote (and `.../versions/<version>/` for a version that is not
the current one), downloads the file the answer names, following the redirect to the CDN, checks it against
the SHA-256 hash of the answer and keeps it as `<id>/<version>/<name>-<version>.xpi`, the layout of a hosted
repository. Its update manifests name the current version of the add-on, with an `update_link` into the
proxy. The answers of the API are kept as metadata (the metadata maximum age decides how long they are
reused, 24 hours by default), ids the remote does not know go into the negative cache, a version once
downloaded is not asked for again, and while the remote is unreachable the proxy serves what it has cached.
A proxy has no `updates.json` for all add-ons. Search, collections, ratings and the other pages of
addons.mozilla.org are not served.
