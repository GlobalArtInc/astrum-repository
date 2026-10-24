# pacman (Arch Linux)

`pacman` downloads `<Server>/<section name>.db` (with `-Fy` also `<section name>.files`), then the
packages that database lists and their `.sig` files from the same `Server`. In `/etc/pacman.conf` it puts
the section name in place of `$repo` and the architecture in place of `$arch`.

A hosted repository keeps its packages in folders named after their architecture, `x86_64/`,
`aarch64/` and so on, and `any/` for `arch=any` packages; the folders may sit below others, for example
`custom/os/x86_64/`. For every architecture folder it generates the databases under whatever name pacman
asks for (`<name>.db`, `<name>.files` and the same with `.tar.gz`), so the section can have any name. The
databases of a folder also list the packages of the `any/` folder next to it, and those packages are served
from every architecture folder. A database lists the newest version of each package in pacman's version
order; older package files stay downloadable.

```ini
[astrum]
SigLevel = Optional TrustAll
Server = https://astrum.example.com/repository/pacman-hosted/$arch
```

```shell
sudo pacman -Sy astrum-tool
```

pacman sends no credentials of its own. Where anonymous users may not read the repository, put a user and
an [access token](../configure/authentication.md#access-tokens) into the address,
`Server = https://<user>:<token>@astrum.example.com/repository/pacman-hosted/$arch`; pacman passes it on
as Basic authentication.

With a signing key (Settings > Repositories > Database Signing, an ASCII-armored OpenPGP private key, for
example from `gpg --export-secret-key --armor`, optionally with a passphrase) the repository signs every
database it serves and answers `<database>.sig`; the public key is served as `<repository name>.key`, and
the page at the repository address shows its fingerprint. Add it to the keyring of pacman once, then
require signed databases:

```shell
curl -O https://astrum.example.com/repository/pacman-hosted/pacman-hosted.key
sudo pacman-key --add pacman-hosted.key
sudo pacman-key --lsign-key <fingerprint>
```

```ini
[astrum]
SigLevel = Required DatabaseRequired
Server = https://astrum.example.com/repository/pacman-hosted/$arch
```

A package signature uploaded next to its package (`<package file>.sig`) is served as it is and written
into the database as `%PGPSIG%`; `SigLevel = Required` then needs the key of the packager in the keyring.

A proxy of an Arch Linux mirror, for example `https://geo.mirror.pkgbuild.com/`, keeps the layout of the
mirror, so name the repositories of the mirror as sections:

```ini
[core]
Server = https://astrum.example.com/repository/pacman-proxy/$repo/os/$arch

[extra]
Server = https://astrum.example.com/repository/pacman-proxy/$repo/os/$arch
```

The proxy fetches databases again after the metadata age and keeps packages and their signatures. The
official databases are unsigned, the official package signatures pass through; a proxy with a signing key
signs the databases it serves instead of passing the signatures of the remote through.

A group asks its members for the same path and merges the databases that more than one member has (the
first member wins for a package name), so a hosted repository can override packages of a proxy by storing
them in the same folders, for example `core/os/x86_64/`. A group signs the databases it serves with its own
key; without one it passes the database and the signature of a single member through and leaves a merge of
several members unsigned.

Upload a package to a hosted repository under the name its `.PKGINFO` gives,
`<pkgname>-<pkgver>-<arch>.pkg.tar.<zst|xz|gz|bz2>` (or `.pkg.tar`), into a folder of its architecture,
and its signature next to it; the repository rewrites the databases of that folder right away:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file astrum-tool-1.4.0-1-x86_64.pkg.tar.zst \
  https://astrum.example.com/repository/pacman-hosted/x86_64/astrum-tool-1.4.0-1-x86_64.pkg.tar.zst
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file astrum-tool-1.4.0-1-x86_64.pkg.tar.zst.sig \
  https://astrum.example.com/repository/pacman-hosted/x86_64/astrum-tool-1.4.0-1-x86_64.pkg.tar.zst.sig
```

The upload page and the components API take packages and signatures in the fields `pacman.asset1`,
`pacman.asset2` and so on; `pacman.directory` names the folders above the architecture folder, and the
architecture folder comes from the package:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F pacman.directory=custom/os \
  -F pacman.asset1=@astrum-tool-1.4.0-1-x86_64.pkg.tar.zst \
  -F pacman.asset2=@astrum-tool-1.4.0-1-x86_64.pkg.tar.zst.sig \
  "https://astrum.example.com/service/rest/v1/components?repository=pacman-hosted"
```

`repo-add` is not needed: deleting a package (its signature goes with it) rewrites the databases too, and
the task **pacman - Rebuild pacman databases** writes them anew, reading the file lists from the packages.
