# syntax=docker/dockerfile:1.7

FROM rust:1.98.0-trixie AS build

ARG GIT_REVISION=""

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/src/target,sharing=locked \
    ASTRUM_GIT_REVISION="${GIT_REVISION}" cargo build --release --locked --bin astrum \
 && install -D -m 0755 target/release/astrum /out/astrum

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
 && mkdir -p /astrum-data \
 && chown 200:200 /astrum-data

COPY --from=build /out/astrum /usr/local/bin/astrum

ENV ASTRUM_DATA_DIR=/astrum-data \
    ASTRUM_LISTEN=0.0.0.0:8081

WORKDIR /astrum-data
USER 200:200
VOLUME /astrum-data
EXPOSE 8081

HEALTHCHECK --interval=30s --timeout=15s --start-period=60s --retries=3 CMD ["astrum", "check"]

ENTRYPOINT ["astrum"]
CMD ["serve"]
