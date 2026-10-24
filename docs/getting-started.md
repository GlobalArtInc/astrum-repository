# Quick start

This page starts a server with its database on one machine, creates an npm proxy and group, and
points npm at them. It takes a few minutes and needs podman or Docker with Compose. For a production
setup read [system requirements](install/requirements.md) and [run the container image](install/container.md).

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

## Sign in

The first start creates the user `admin` with a random password:

```shell
podman compose exec astrum cat /astrum-data/admin.password
```

Open http://localhost:8081 and sign in as `admin` with it. The setup page asks for a new password,
whether anonymous users may read repositories, and which formats to create hosted repositories for.
Pick npm; the server creates `npm-hosted`.

## Cache the public registry

A proxy repository fetches from another registry and keeps what it fetched, and a group serves several
repositories under one address. Under **Settings → Repository → Repositories** choose
**Create Repository**, search for npm, and create two repositories:

1. a proxy, named `npm-proxy`, with **Remote Storage** `https://registry.npmjs.org`;
2. a group, named `npm-group`, with the **Member Repositories** `npm-hosted` and `npm-proxy` in this
   order, so your own packages win over public ones of the same name.

## Use it

Point npm at the group and install something. This works without signing in if you let anonymous users
read on the setup page; otherwise run `npm login` first, as the [npm page](formats/npm.md) shows.

```shell
npm config set registry http://localhost:8081/repository/npm-group/
npm install left-pad
```

The package now shows in the web interface under **Browse** in `npm-proxy`, and the next install is
served from the cache. To publish, activate the **npm Bearer Token Realm** under
**Settings → Security → Realms**, sign in with `npm login` and run `npm publish` against `npm-hosted`.

## Next steps

- [Formats](formats/README.md): the same for Maven, Docker, PyPI, Cargo, Go and the other formats.
- [Repositories](repositories/overview.md): what hosted, proxy and group repositories can do.
- [Users, realms and sign-in](configure/authentication.md): roles, LDAP, OpenID Connect and access tokens.
- [Reverse proxy and TLS](configure/reverse-proxy.md): serve the server under its own name over HTTPS.
- [Backup and restore](operate/backup-restore.md): what to back up before you rely on it.
