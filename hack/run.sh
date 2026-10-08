#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
data="${ASTRUM_DATA:-$root/.data}"
assembly="$root/public/selfhosted/assemblies/astrum-repository-core/target/assembly"
java_home="${JAVA_HOME:-/opt/homebrew/opt/openjdk@25/libexec/openjdk.jdk/Contents/Home}"
mkdir -p "$data/astrum3"
cd "$assembly"
jar="$(ls bin/astrum-repository-core-*.jar)"
exec "$java_home/bin/java" -Xmx"${ASTRUM_HEAP:-2g}" -Dkaraf.data="$data/astrum3" -Djava.io.tmpdir="$data/astrum3/tmp" \
  -Dastrum.webresources.cachebuster="$(date -r "$jar" +%Y%m%d%H%M%S)" -jar "$jar"
