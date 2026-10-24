# Yum (dnf)

`/etc/yum.repos.d/astrum.repo` for a hosted repository that generates its repodata at the root:

```ini
[astrum]
name=Astrum
baseurl=https://astrum.example.com/repository/yum-hosted/
enabled=1
gpgcheck=0
```

When the repository signs its metadata, add `repo_gpgcheck=1` and
`gpgkey=https://astrum.example.com/repository/yum-hosted/repodata/repomd.xml.key`. A repository
with a repodata depth of 2 keeps one repodata per directory pair, so the `baseurl` ends with them,
for example `.../yum-hosted/el9/x86_64/`. For a proxy of `https://download.rockylinux.org/pub/rocky/`
the `baseurl` follows the remote's layout: `.../rocky-proxy/9/BaseOS/x86_64/os/`.

Uploading to a hosted repository; it rebuilds the repodata a few seconds later:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file hello-2.12.1-3.el9.x86_64.rpm \
  https://astrum.example.com/repository/yum-hosted/Packages/hello-2.12.1-3.el9.x86_64.rpm
```
