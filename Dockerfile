# syntax=docker/dockerfile:1.7

FROM node:22-bookworm-slim AS web

# The translations are pulled from Ownlate here, so the image carries them and git does not.
# The token is a build secret: docker build --secret id=ownlate_token,env=ASTRUM_OWNLATE_TOKEN
ARG ASTRUM_OWNLATE_PROJECT=f1f93c0e-4902-434d-a551-e94ba7e1e304

ENV COREPACK_ENABLE_DOWNLOAD_PROMPT=0 \
    CI=true

WORKDIR /src/web
COPY web/package.json web/pnpm-lock.yaml ./
RUN --mount=type=cache,target=/root/.cache/node/corepack,sharing=locked \
    --mount=type=cache,target=/root/.local/share/pnpm/store,sharing=locked \
    corepack pnpm install --frozen-lockfile
COPY web ./
COPY tools/i18n /src/tools/i18n
RUN --mount=type=secret,id=ownlate_token \
    ASTRUM_OWNLATE_TOKEN="$(cat /run/secrets/ownlate_token)" node /src/tools/i18n/i18n.mjs pull \
 && node /src/tools/i18n/i18n.mjs check
RUN --mount=type=cache,target=/root/.cache/node/corepack,sharing=locked \
    corepack pnpm build

FROM rust:1.98.0-trixie AS build

ARG GIT_REVISION=""

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/src/target,sharing=locked \
    ASTRUM_GIT_REVISION="${GIT_REVISION}" cargo build --release --locked --bin astrum --bin astrum-blob-agent \
 && install -D -m 0755 target/release/astrum /out/astrum \
 && install -D -m 0755 target/release/astrum-blob-agent /out/astrum-blob-agent

FROM debian:trixie-slim

ARG VERSION=dev
ARG GIT_REVISION=unknown

LABEL org.opencontainers.image.title="Astrum Repository" \
      org.opencontainers.image.description="Artifact repository manager" \
      org.opencontainers.image.vendor="GlobalArt" \
      org.opencontainers.image.source="https://github.com/GlobalArtInc/astrum-repository" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${GIT_REVISION}"

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd --system --gid 200 astrum \
 && useradd --system --uid 200 --gid 200 --home-dir /astrum-data --no-create-home --shell /usr/sbin/nologin astrum \
 && mkdir -p /astrum-data /astrum-agent \
 && chown 200:200 /astrum-data /astrum-agent

COPY --from=build /out/astrum /usr/local/bin/astrum
COPY --from=build /out/astrum-blob-agent /usr/local/bin/astrum-blob-agent
COPY --from=web /src/web/dist /usr/share/astrum/web
COPY LICENSE.txt NOTICE.txt /usr/share/doc/astrum/

ENV ASTRUM_DATA_DIR=/astrum-data \
    ASTRUM_LISTEN=0.0.0.0:8081 \
    ASTRUM_WEB_DIR=/usr/share/astrum/web

WORKDIR /astrum-data
USER 200:200
VOLUME /astrum-data
EXPOSE 8081

HEALTHCHECK --interval=30s --timeout=15s --start-period=60s --retries=3 CMD ["astrum", "check"]

ENTRYPOINT ["astrum"]
CMD ["serve"]
