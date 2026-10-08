# Run the distribution without a container

## Build or unpack the distribution

The server is a single executable jar plus its default configuration:

```text
<install>/
  bin/astrum-repository-core-<version>.jar
  etc/astrum-default.properties     default port, host and context path
  etc/default-application.properties
  etc/fabric/                       cache and MyBatis defaults
  etc/jetty/                        Jetty configuration, including HTTPS templates
  etc/logback/                      logging configuration
```

Build it from source with Java 25 and Maven 3.9.6+:

```shell
corepack enable
corepack yarn install
mvn clean install -Ppublic -DskipTests
```

The distribution is then in `public/selfhosted/assemblies/astrum-repository-core/target/assembly`.
Copy that directory to the install location, for example `/opt/astrum`.

## Start the server

The working directory must be the install directory, and `karaf.data` points at the data directory:

```shell
cd /opt/astrum
java -Xms2703m -Xmx2703m -XX:MaxDirectMemorySize=2703m \
  -Dkaraf.data=/var/lib/astrum \
  -Djava.io.tmpdir=/var/lib/astrum/tmp \
  -jar bin/astrum-repository-core-*.jar
```

Keep the install directory and the data directory apart: upgrades replace the install directory and
leave the data directory alone.

## systemd service

Create a service user and the data directory:

```shell
useradd --system --home-dir /var/lib/astrum --shell /usr/sbin/nologin astrum
install -d -o astrum -g astrum /var/lib/astrum /var/lib/astrum/tmp
```

`/etc/systemd/system/astrum.service`:

```ini
[Unit]
Description=Astrum Repository
Wants=network-online.target
After=network-online.target

[Service]
Type=simple
User=astrum
Group=astrum
WorkingDirectory=/opt/astrum
EnvironmentFile=-/etc/astrum/astrum.env
ExecStart=/bin/sh -c 'exec java $ASTRUM_JAVA_OPTS -Dkaraf.data=/var/lib/astrum -Djava.io.tmpdir=/var/lib/astrum/tmp -jar /opt/astrum/bin/astrum-repository-core-*.jar'
Restart=on-failure
TimeoutStopSec=180
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

`/etc/astrum/astrum.env`, readable only by root and the service user:

```shell
ASTRUM_JAVA_OPTS="-Xms2703m -Xmx2703m -XX:MaxDirectMemorySize=2703m"
ASTRUM_DATASTORE_ASTRUM_JDBCURL=jdbc:postgresql://db.example.com:5432/astrum
ASTRUM_DATASTORE_ASTRUM_USERNAME=astrum
ASTRUM_DATASTORE_ASTRUM_PASSWORD=the-database-password
```

```shell
systemctl daemon-reload
systemctl enable --now astrum
journalctl -u astrum -f
```

The generated `admin` password is written to `/var/lib/astrum/admin.password` on the first start;
see [first sign-in](container.md#first-sign-in).
