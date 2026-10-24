# Connect build tools to repositories

Every repository is served at `https://<host>/repository/<repository name>/`. The examples use
`astrum.example.com` and repository names that follow a `<format>-hosted`, `<format>-proxy`,
`<format>-group` convention; use the names you created. Point clients at a group when you read and at
a hosted repository when you publish.

Clients authenticate with the user name and password of a user, with an
[access token](../configure/authentication.md#access-tokens) in place of the password (the way for CI
and for users who sign in through OpenID Connect), or with a token of their format where it has one.
See [users, realms and sign-in](../configure/authentication.md) for the realms that tokens need.
The formats this server supports are Alpine, Ansible Galaxy, APT, Bazel, Cargo, Chef Supermarket, Chromium extensions, CocoaPods, Composer,
Conan, Conda, Docker and other OCI artifacts, Firefox add-ons, Git LFS, Go, Helm, Hex, Hugging Face, JetBrains plugins, Maven, Nix, npm, NuGet,
Open VSX, P2, pacman, Protobuf, pub, Puppet Forge, PyPI, R, RubyGems, Swift, Terraform, Vagrant, Yum and raw files,
and remote caches for Nx and Turborepo.

That address belongs to the clients and has no page of its own: a browser opening it gets a page that
links to the HTML index of the repository at `https://<host>/service/rest/repository/browse/<repository
name>/` and to the repository in the web interface, provided the user may browse it. The HTML index lists
folders and files with links to download them; the summary of the repository in Settings links to it too.


## Formats

The settings of each client, how to publish and what proxies and groups do are on the page of its
format; the [list of formats](../formats/README.md) shows which repository types each one has.
