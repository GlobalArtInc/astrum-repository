#!/bin/sh
set -eu

data="${ASTRUM_DATA:-/astrum-data}"
mkdir -p "$data/tmp" "$data/javaprefs"

if [ -n "${ASTRUM_DATASTORE_ASTRUM_PASSWORD_FILE:-}" ]; then
  ASTRUM_DATASTORE_ASTRUM_PASSWORD="$(cat "$ASTRUM_DATASTORE_ASTRUM_PASSWORD_FILE")"
  export ASTRUM_DATASTORE_ASTRUM_PASSWORD
fi

cd /opt/astrum
set -- /opt/astrum/bin/astrum-repository-core-*.jar
exec java ${ASTRUM_JAVA_OPTS:-} \
  -Dkaraf.data="$data" \
  -Djava.io.tmpdir="$data/tmp" \
  -Djava.util.prefs.userRoot="$data/javaprefs" \
  -jar "$1"
