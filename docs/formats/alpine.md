# Alpine

`apk` reads `<repository URL>/<architecture>/APKINDEX.tar.gz`, so a repository line names the directory
above the architecture folders, for example `v3.20/main`. When the repository has a signing key, save its
public key under the name it is served with, `<repository name>.rsa.pub`, before the first update:

```shell
wget -O /etc/apk/keys/alpine-hosted.rsa.pub \
  https://astrum.example.com/repository/alpine-hosted/alpine-hosted.rsa.pub
echo "https://astrum.example.com/repository/alpine-hosted/v3.20/main" >> /etc/apk/repositories
apk update
apk add tool
```

Repositories without a key serve unsigned indexes, which `apk` reads only with `--allow-untrusted`. A
proxy with a key adds its signature in front of the remote's, so the official Alpine keys keep working;
a group signs its merged indexes with its own key, and without one passes the index of a single member
through as that member serves it and leaves a merge of several members unsigned.

Upload a package to a hosted repository under the name `apk` asks for, `<name>-<version>.apk`, in the
directory of its architecture (`noarch` packages take the architecture of the directory); the
repository then writes and signs the `APKINDEX.tar.gz` of that directory:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file tool-1.4.0-r0.apk \
  https://astrum.example.com/repository/alpine-hosted/v3.20/main/x86_64/tool-1.4.0-r0.apk
```

The signing key is a PEM-encoded RSA private key, for example from `openssl genrsa -out alpine.key 4096`,
optionally encrypted with a passphrase.
