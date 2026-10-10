# syntax=docker/dockerfile:1.7

FROM eclipse-temurin:25-jdk AS build

ARG MAVEN_VERSION=3.9.11
ARG GIT_REVISION=unknown

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl git \
 && rm -rf /var/lib/apt/lists/* \
 && git config --global --add safe.directory /src \
 && curl -fsSL "https://archive.apache.org/dist/maven/maven-3/${MAVEN_VERSION}/binaries/apache-maven-${MAVEN_VERSION}-bin.tar.gz" \
    | tar -xz -C /opt \
 && ln -s "/opt/apache-maven-${MAVEN_VERSION}/bin/mvn" /usr/local/bin/mvn

WORKDIR /src
COPY . .

RUN --mount=type=cache,target=/root/.m2/repository \
    --mount=type=cache,target=/root/.yarn/berry/cache \
    mvn -B -Ppublic -DskipTests -Dnpm.skipTests=true \
      "-Dmaven.buildNumber.revisionOnScmFailure=${GIT_REVISION}" install \
 && cp -r public/selfhosted/assemblies/astrum-repository-core/target/assembly /opt/astrum \
 && chmod -R a+rX /opt/astrum

FROM eclipse-temurin:25-jre

ARG VERSION=dev
ARG GIT_REVISION=unknown

LABEL org.opencontainers.image.title="Astrum Repository" \
      org.opencontainers.image.description="Artifact repository manager" \
      org.opencontainers.image.vendor="GlobalArt" \
      org.opencontainers.image.source="https://github.com/GlobalArtInc/astrum" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${GIT_REVISION}" \
      org.opencontainers.image.licenses="EPL-1.0"

RUN apt-get update \
 && apt-get install -y --no-install-recommends curl \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd --system --gid 200 astrum \
 && useradd --system --uid 200 --gid 200 --home-dir /astrum-data --no-create-home --shell /usr/sbin/nologin astrum \
 && mkdir -p /astrum-data \
 && chown 200:200 /astrum-data

COPY --from=build /opt/astrum /opt/astrum
COPY --chmod=0755 docker/entrypoint.sh /opt/astrum/bin/astrum
COPY LICENSE.txt NOTICE.txt /opt/astrum/

ENV ASTRUM_DATA=/astrum-data \
    ASTRUM_JAVA_OPTS="-Xms2703m -Xmx2703m -XX:MaxDirectMemorySize=2703m"

WORKDIR /opt/astrum
USER 200:200
VOLUME /astrum-data
EXPOSE 8081

ENTRYPOINT ["/opt/astrum/bin/astrum"]
