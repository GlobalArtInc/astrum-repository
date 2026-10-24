# APT

`/etc/apt/sources.list.d/astrum.list` for a repository whose distribution is `bookworm`, after importing
the signing key of the repository (its public key is set on the repository's APT signing settings):

```text
deb https://astrum.example.com/repository/apt-hosted/ bookworm main
```

Upload a package to a hosted repository:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -H "Content-Type: multipart/form-data" \
  --data-binary "@tool_1.4.0_amd64.deb" https://astrum.example.com/repository/apt-hosted/
```
