# JetBrains plugins (IntelliJ Platform IDEs)

Repositories of the format `jetbrains` serve plugins to IntelliJ IDEA, GoLand, RustRover, WebStorm, PyCharm and
the other IDEs built on the IntelliJ Platform. A hosted repository keeps plugin files (a `.zip` with the plugin
in `lib/*.jar`, or a bare `.jar`; both hold `META-INF/plugin.xml`) and generates the `updatePlugins.xml` file of
a [custom plugin repository](https://plugins.jetbrains.com/docs/intellij/custom-plugin-repository.html). A
proxy caches JetBrains Marketplace (`https://plugins.jetbrains.com` is the remote URL) or another server that
answers the same requests, such as a hosted repository of this server, so that IDEs in networks without access
to the internet can still list, install and update plugins. A group merges its members, the first member
winning for each plugin.

To offer a hosted repository, or a group, next to JetBrains Marketplace, open **Settings → Plugins**, click
the gear icon, choose **Manage Plugin Repositories**, click **+** and enter

```text
https://astrum.example.com/repository/jetbrains-hosted/updatePlugins.xml
```

The same address can be put into the custom properties of the IDE (**Help → Edit Custom Properties**, then
restart the IDE); separate several addresses with semicolons:

```properties
idea.plugin.hosts=https://astrum.example.com/repository/jetbrains-group/updatePlugins.xml
```

The IDE adds its build number to the address (`?build=IU-243.21565.193`), and the server answers with the
newest version of every plugin whose `since-build` and `until-build` cover that build, so one file serves
every IDE version and needs no `updatePlugins-<build>.xml` files. Plugins in a release channel other than the
default are left out of it; `updatePlugins.xml?channel=eap` lists them.

In a network without internet access, make a proxy of `https://plugins.jetbrains.com` (or a group of it and a
hosted repository) the default plugin repository of the IDE instead of Marketplace, in the custom properties
or as a VM option (**Help → Edit Custom VM Options**), and restart the IDE:

```properties
idea.plugins.host=https://astrum.example.com/repository/jetbrains-group
```

```text
-Didea.plugins.host=https://astrum.example.com/repository/jetbrains-group
```

An IDE reads a plugin repository without credentials unless an authentication extension of the IDE adds them,
so a repository it reads should be readable anonymously: keep anonymous access on and narrow `as-anonymous` to
that repository, or let only your network reach it.

Publish to a hosted repository in any of three ways. The upload API takes the request that the JetBrains
Marketplace upload API documents, with an [access token](../configure/authentication.md#access-tokens) as the
bearer token; `xmlId` (or `pluginId`) is optional and, when given, has to be the id in `plugin.xml`, and
`channel` names a release channel such as `eap` or `beta` (empty for the default channel):

```shell
curl -i -H "Authorization: Bearer $ASTRUM_TOKEN" -F xmlId=org.acme.tool -F file=@build/distributions/tool-1.2.0.zip \
  https://astrum.example.com/repository/jetbrains-hosted/api/updates/upload
```

A file can also be put to any path that ends in `.zip` or `.jar`, and the form of the upload page and the
components API takes it in the field `jetbrains.asset` (and the channel in `jetbrains.channel`):

```shell
curl -i -X PUT -H "Authorization: Bearer $ASTRUM_TOKEN" -T build/distributions/tool-1.2.0.zip \
  https://astrum.example.com/repository/jetbrains-hosted/tool-1.2.0.zip
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F jetbrains.asset=@tool-1.2.0.zip \
  "https://astrum.example.com/service/rest/v1/components?repository=jetbrains-hosted"
```

The server reads `plugin.xml` for the id (or the name, when there is no id), version, vendor, `since-build`
and `until-build`, description, change notes, dependencies and category, and stores the file as
`<id>/<version>/<id>-<version>.zip` (`.jar` for a bare jar), whatever path it was put to. The version is
required. Unless the write policy allows redeploying, a version that is already there is refused with 409.

Besides `updatePlugins.xml` the repositories answer the requests that an IDE sends to its default repository:
`plugins/list` (also as `plugins/<channel>/list`), with the `build` and `pluginId` parameters, in the format
of Marketplace; `pluginManager?action=download&id=<plugin id>&build=<product>-<build>`, which downloads the
newest compatible version; and `plugin/download?pluginId=<plugin id>&version=<version>`. Files are also
served at their own path.

A proxy forwards only `build`, `channel`, `pluginId`, `os` and `arch`: the installation id (`uuid`) that IDEs
add to their requests never reaches the remote. It keeps each answer of `plugins/list` as metadata (the metadata maximum
age decides how long it is reused, 24 hours by default), and each plugin once, under the id and version
written in its `plugin.xml`, however many IDE builds ask for it. When the remote is unreachable, what is cached
is served. Proxies have no `updatePlugins.xml`, and groups list only the plugins of their hosted members in it,
while `plugins/list` merges all members.

Marketplace features beyond these requests are not served: search and `compatibleUpdates` of its `/api`,
plugin pages, ratings and paid licences.
