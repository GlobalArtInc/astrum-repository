# Open VSX (VS Code extensions)

Repositories serve VS Code extensions (`.vsix` packages) to editors built on VS Code that read an
Open VSX or Marketplace-compatible extension gallery, such as VSCodium, code-server, Theia, Gitpod and
OpenVSCode Server, and to the `ovsx` command line client. Hosted repositories take `ovsx publish`, proxies
cache another Open VSX registry such as `https://open-vsx.org` (the registry root is the remote URL), and
groups merge their members, the first member winning for each version and target platform.

Point an editor at a repository, usually a group of a hosted repository and a proxy, in the
`extensionsGallery` section of its `product.json`. VSCodium also reads a `product.json` in its user data
folder (`~/.config/VSCodium/product.json` on Linux, `~/Library/Application Support/VSCodium/product.json`
on macOS, `%APPDATA%\VSCodium\product.json` on Windows); restart the editor after changing it:

```json
{
  "extensionsGallery": {
    "serviceUrl": "https://astrum.example.com/repository/openvsx-group/vscode/gallery",
    "itemUrl": "https://astrum.example.com/repository/openvsx-group/vscode/item",
    "resourceUrlTemplate": "https://astrum.example.com/repository/openvsx-group/vscode/unpkg/{publisher}/{name}/{version}/{path}",
    "extensionUrlTemplate": "https://astrum.example.com/repository/openvsx-group/vscode/gallery/{publisher}/{name}/latest"
  }
}
```

Editors that take the gallery from the environment read the same addresses from variables:

```shell
export VSCODE_GALLERY_SERVICE_URL=https://astrum.example.com/repository/openvsx-group/vscode/gallery
export VSCODE_GALLERY_ITEM_URL=https://astrum.example.com/repository/openvsx-group/vscode/item
export VSCODE_GALLERY_EXTENSION_URL_TEMPLATE='https://astrum.example.com/repository/openvsx-group/vscode/gallery/{publisher}/{name}/latest'
export VSCODE_GALLERY_RESOURCE_URL_TEMPLATE='https://astrum.example.com/repository/openvsx-group/vscode/unpkg/{publisher}/{name}/{version}/{path}'
```

Editors send no credentials to their gallery, so the repository they use must be readable anonymously:
keep anonymous access on and narrow `as-anonymous` to that repository, or let only your network reach it.
The repositories answer cross-origin requests, so editors that run in a browser can use them too.

`ovsx` takes the repository address as its registry URL:

```shell
npx ovsx search yaml -r https://astrum.example.com/repository/openvsx-group
npx ovsx get redhat.vscode-yaml -t linux-x64 -r https://astrum.example.com/repository/openvsx-group
```

Publish to a hosted repository. The personal access token of `ovsx` (`-p` or `OVSX_PAT`) is an
[access token](../configure/authentication.md#access-tokens) of this server, or `<user>:<password>`:

```shell
npx ovsx publish acme.hello-1.2.0.vsix -r https://astrum.example.com/repository/openvsx-hosted -p "$ASTRUM_TOKEN"
npx ovsx publish -i acme.hello-1.2.0@linux-x64.vsix -r https://astrum.example.com/repository/openvsx-hosted -p "$ASTRUM_TOKEN"
```

A package must hold `extension.vsixmanifest` and `extension/package.json`, both naming the same publisher,
name and semantic version; the publisher is the namespace of the extension. Builds for target platforms
(`vsce package --target linux-x64`, `darwin-arm64`, `web` and the others) are kept side by side as one
version, and a platform without a build of its own gets the universal one. Pre-release builds
(`vsce package --pre-release`) are listed as `pre-release` and become `latest` only while there is no
release. Unless the write policy allows redeploying, a version and platform that is already there with
other content is refused, and `ovsx publish --skip-duplicate` passes over it. Namespaces need no
creation: `ovsx create-namespace` succeeds for users who may publish, and `ovsx verify-pat` tells whether
the token may publish to the repository. `ovsx unpublish` deletes versions from a hosted repository for
users with the delete privilege. The upload page and the components API take `.vsix` files in the field
`openvsx.asset`:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F openvsx.asset=@acme.hello-1.2.0.vsix \
  "https://astrum.example.com/service/rest/v1/components?repository=openvsx-hosted"
```

The repositories serve the registry API under `api/` (namespaces, extension metadata per version and
target platform, `versions`, `version-references`, the files of a version under `.../file/`, `api/-/search`,
`api/-/query`, `api/v2/-/query` and `api/-/publish`) and the gallery API under `vscode/` (`gallery/extensionquery`,
`gallery/{publisher}/{name}/latest`, `gallery/publishers/.../vspackage`, `asset/...`, `unpkg/...` and
`item`). Hosted repositories read the files of a version from inside its package and keep no signatures.
Proxies keep the answers of the remote as metadata with every address pointed at the proxy, fetch each
package and file on first use and check packages against the SHA-256 file of the remote.
