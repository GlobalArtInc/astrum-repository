# Nix

Nix repositories are binary caches: Nix substitutes store paths from them, and `nix copy` uploads store
paths to hosted ones. A proxy of `https://cache.nixos.org` and a group of it and a hosted repository give
clients both in one substituter. Nix reads the cache with `GET` of `nix-cache-info`, `<hash>.narinfo` and
the NAR file the narinfo names (`nar/<hash>.nar.xz` and the like), and optionally `<hash>.ls`,
`log/<derivation>` and `realisations/<id>.doi`. The address of the repository without the trailing slash
is the store URL. In `/etc/nix/nix.conf` (restart `nix-daemon` on a multi-user installation):

```text
substituters = https://astrum.example.com/repository/nix-group
trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= astrum-1:<public key>
netrc-file = /etc/nix/netrc
```

Nix sends the credentials of the netrc file with every request to the host, for reading and uploading; a
repository that anonymous users may read needs none. Use an
[access token](../configure/authentication.md#access-tokens) as the password:

```text
machine astrum.example.com
login ci-reader
password <access token>
```

The server does not sign store paths: Nix accepts a substituted path only with a signature by a key in
`trusted-public-keys`, so list the key of cache.nixos.org for the proxy and your own key for what you
upload. `nix-cache-info` comes from the Nix settings of the repository (the `nix` section over REST):
`storeDir` (`/nix/store`), `priority` (40, the priority of cache.nixos.org; Nix asks lower numbers first)
and `wantMassQuery` (on). A group answers with its own settings, not those of its members.

To upload, make a signing key once, give its public key to the clients and let `nix copy` sign while it
uploads to a hosted repository (the user needs the add privilege, and edit to replace files when the write
policy allows redeploying):

```shell
nix key generate-secret --key-name astrum-1 > /etc/nix/astrum-1.secret
nix key convert-secret-to-public < /etc/nix/astrum-1.secret
nix copy --to 'https://astrum.example.com/repository/nix-hosted?secret-key=/etc/nix/astrum-1.secret' ./result
```

`nix store sign --key-file /etc/nix/astrum-1.secret --recursive ./result` signs before a plain
`nix copy --to https://astrum.example.com/repository/nix-hosted ./result`; add `&write-nar-listing=true`
to upload the `.ls` listings, and `nix store copy-log --to <store URL> ./result` uploads build logs.
`nix copy` asks for the narinfo and the NAR file with `HEAD` and uploads what is missing with `PUT`, the
NAR file before its narinfo. The repository refuses a narinfo that is compressed (leave `narinfo-compression` unset), that names
a store path outside its store directory or under another hash than its file name, whose `URL` is not a
NAR file uploaded to the repository before, whose `FileSize` or `FileHash` differ from that file, or whose
hashes, references, deriver, signatures or content address are malformed; `nix copy` prints the reason.
Listings and logs keep the `Content-Encoding` they were uploaded with. With the write policy "Allow
once" a second narinfo for a store path is refused, while the same NAR file again is accepted.

Every store path is a component: its group is the hash part, its name and version are split from the name
of the store path the way `nix-env` does (`hello` and `2.12.1` for `…-hello-2.12.1`), and its assets are the
narinfo, the listing and the NAR file. Two store paths with identical contents share one NAR file, which
then belongs to neither component. Deleting a component, or `DELETE` of `<hash>.narinfo`, removes the
narinfo, its listing and the NAR file unless another narinfo still uses it; a narinfo whose NAR file is
gone is no longer served. The task "Nix - Delete NAR files no narinfo uses" removes the NAR files of
uploads that never got their narinfo and of shared files whose store paths were all deleted, once they
are older than its deploy offset.

Proxies keep narinfo files as metadata for the maximum metadata age (the form starts proxies at 60
minutes) and remember misses in the negative cache (60 minutes in the form; Nix itself remembers misses
for an hour), since most narinfo lookups of a client are misses. NAR files never change and are not
checked again. A proxy refuses a narinfo of the remote that names another store path or another store
directory with 502, rewrites a `URL` that points under the remote to a relative one so that clients
download through the proxy, and leaves `URL`s on other hosts alone. Listings and logs keep the
`Content-Encoding` of the remote (cache.nixos.org compresses them). Groups answer narinfo, NAR and other
files from the first member that has them; upload to the hosted repository itself.
