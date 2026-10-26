# Quick start

This page starts a server with its database on one machine, creates a proxy and a group for npm or
Docker, and points the client at them. Where the steps differ, a switch chooses between npm and
Docker. It takes a few minutes and needs podman or Docker with Compose. For a production setup read
[system requirements](install/requirements.md) and [run the container image](install/container.md).

## Start the server

Save this as `compose.yaml` in an empty directory:

```yaml
services:
  postgres:
    image: docker.io/library/postgres:18
    environment:
      POSTGRES_DB: astrum
      POSTGRES_USER: astrum
      POSTGRES_PASSWORD: astrum
    volumes:
      - postgres:/var/lib/postgresql
    healthcheck:
      test: ["CMD", "pg_isready", "-U", "astrum", "-d", "astrum"]
      interval: 5s

  astrum:
    image: docker.io/globalartltd/astrum-repository:latest
    depends_on:
      postgres:
        condition: service_healthy
    ports:
      - "8081:8081"
    environment:
      ASTRUM_DATABASE_URL: postgres://astrum:astrum@postgres:5432/astrum
    volumes:
      - astrum:/astrum-data

volumes:
  postgres:
  astrum:
```

and start it:

```shell
podman compose up -d
```

`docker compose up -d` does the same. The password is written into the file to keep the example short;
[compose with PostgreSQL](install/container.md#compose-with-postgresql) shows how to keep it in a secret.
The server creates its tables on the first start and is ready when this returns HTTP 200:

```shell
curl -fs http://localhost:8081/service/rest/v1/status/writable
```

The image is built for `linux/amd64` only. On an ARM machine, such as a Mac with Apple Silicon, add
`platform: linux/amd64` under `image:` of the `astrum` service and the server runs through emulation;
without it the pull stops with `no matching manifest for linux/arm64/v8`.

## Sign in

The first start creates the user `admin` with a random password:

```shell
podman compose exec astrum cat /astrum-data/admin.password
```

Open http://localhost:8081 and sign in as `admin` with it. The setup page asks for a new password,
whether anonymous users may read repositories, and which formats to create hosted repositories for.
Pick npm, Docker or both; the server creates `npm-hosted` and `docker-hosted`.

## Cache the public registry

A proxy repository fetches from another registry and keeps what it fetched, and a group serves several
repositories under one address. Under **Settings → Repository → Repositories** choose
**Create Repository**, search for the format, and create two repositories:

<ClientSwitch>
<template #npm>

1. a proxy, named `npm-proxy`, with **Remote Storage** `https://registry.npmjs.org`;
2. a group, named `npm-group`, with the **Member Repositories** `npm-hosted` and `npm-proxy` in this
   order, so your own packages win over public ones of the same name.

</template>
<template #docker>

1. a proxy, named `docker-proxy`, with **Remote Storage** `https://registry-1.docker.io`, the address of
   Docker Hub; leave **Docker Index** on **Use the proxied registry**;
2. a group, named `docker-group`, with the **Member Repositories** `docker-hosted` and `docker-proxy` in
   this order, so your own images win over public ones of the same name.

Leave **Routing Mode** on **Use path routing**: the name of the repository is then the first part of
the name of the image, as the commands below show. [Docker](formats/docker.md) explains the other mode.

</template>
</ClientSwitch>

## Use it

Point the client at the group and fetch something.

<ClientSwitch>
<template #npm>

This works without signing in if you let anonymous users read on the setup page; otherwise run
`npm login` first, as the [npm page](formats/npm.md) shows.

```shell
npm config set registry http://localhost:8081/repository/npm-group/
npm install left-pad
```

The package now shows in the web interface under **Browse** in `npm-proxy`, and the next install is
served from the cache. To publish, activate the **npm Bearer Token Realm** under
**Settings → Security → Realms**, sign in with `npm login` and run `npm publish` against `npm-hosted`.

</template>
<template #docker>

Docker repositories ask for a login even when anonymous users may read, because **Force Basic
Authentication** is set on them. Sign in as `admin`, or as any user who may read, and pull through the
group. The first part of the image name is the repository, and the official images of Docker Hub live
under `library/`:

```shell
docker login localhost:8081
docker pull localhost:8081/docker-group/library/alpine:3
```

The image now shows in the web interface under **Browse** in `docker-proxy`, and the next pull is
served from the cache. To publish, give an image the name of `docker-hosted` and push it; the group
serves what you pushed:

```shell
docker tag localhost:8081/docker-group/library/alpine:3 \
  localhost:8081/docker-hosted/team/alpine:1.0
docker push localhost:8081/docker-hosted/team/alpine:1.0
docker pull localhost:8081/docker-group/team/alpine:1.0
```

Docker speaks plain HTTP to `localhost` without further setup; for any other host name serve the
server over HTTPS, see [reverse proxy and TLS](configure/reverse-proxy.md). With podman or skopeo
add the flag `--tls-verify=false` to talk plain HTTP. To pull without `docker login`, allow anonymous
access and clear **Force Basic Authentication** in the repository, as the
[Docker page](formats/docker.md) describes.

</template>
</ClientSwitch>

## Next steps

- [Formats](formats/README.md): the same for Maven, PyPI, Cargo, Go and the other formats.
  [Docker](formats/docker.md) goes on with OCI artifacts and Helm charts.
- [Repositories](repositories/overview.md): what hosted, proxy and group repositories can do.
- [Users, realms and sign-in](configure/authentication.md): roles, LDAP, OpenID Connect and access tokens.
- [Reverse proxy and TLS](configure/reverse-proxy.md): serve the server under its own name over HTTPS.
- [Backup and restore](operate/backup-restore.md): what to back up before you rely on it.
