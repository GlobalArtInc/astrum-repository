# SSL certificates

The truststore is a list of certificates the server trusts on connections it opens itself, on top of the
certificates of its system. Use it for remotes, mail servers and the like whose certificate comes from a
private CA or is self-signed. Manage it under **Settings → Security → SSL Certificates**, or with the
REST API under `/service/rest/v1/security/ssl/truststore`.

The certificates are stored in the database, so every server on the same database uses them, within ten
seconds and without a restart. Additions and removals are recorded in the audit log.

The truststore only concerns outgoing connections. To serve Astrum itself over HTTPS, see
[reverse proxy and TLS](reverse-proxy.md).

## Adding a certificate

Click **Add Certificate** and choose a **Certificate Source**:

- **Load from server**: enter a host name, `host:port` or a URL under **Hostname or URL**, for example
  `ldap.example.com:636` or `https://repo.example.com/maven2/`. Without a port, `https://` URLs and bare
  host names use 443 and `http://` URLs use 80. Astrum connects and takes the certificate the server
  presents, its own certificate rather than the CA's. On ports 25 and 587 it sends `STARTTLS` first, as a
  mail server expects.
- **Paste PEM**: paste the certificate under **Certificate in PEM Format**, starting with
  `-----BEGIN CERTIFICATE-----`. Only the first certificate of the text is taken, so add the certificates
  of a bundle one by one.

**Load Certificate** shows the subject, the issuer, the validity and the fingerprint. Check them, then
click **Add Certificate**. A certificate that is already in the truststore cannot be added again.

**Load from server** connects directly, not through the [outbound HTTP proxy](outbound-http.md), gives
up after 15 seconds, and is subject to the SSRF protection: a server on an internal address has to be
allowed first (see [runtime configuration](runtime.md#outbound-http-proxy)), or its certificate pasted
instead.

What to add:

- the certificate of the CA, root or intermediate, when a private CA signs your servers: every
  certificate it issues is then trusted;
- the server's own certificate, which is what **Load from server** takes, when it is self-signed: only
  that certificate is trusted, until the server gets a new one.

Next to the trust options of a proxy repository, of **Settings → System → Email Server** and of
**Settings → Security → OpenID Connect**, **View Certificate** shows the certificate the configured
server presents, with **Add Certificate to Truststore** or **Remove Certificate from Truststore**. For an
LDAP server, use **Load from server** with its `host:port`.

## The list

The list shows **Name**, **Issued To**, **Issued By** and **Fingerprint** (SHA-1) of each certificate and
can be filtered by name, organization or fingerprint. A certificate that expires within 30 days is marked
**This certificate expires soon**, one that has expired **This certificate has expired**; nothing replaces
it automatically.

Open a certificate to see its details, and **Delete Certificate** to remove it. Connections that relied on
it fail from then on.

## Connections that use the truststore

The truststore applies where a connection has its option turned on:

| Connection | Option |
|---|---|
| A [proxy repository](../repositories/overview.md#proxy-repositories) whose **Remote Storage** starts with `https://` | **Use the Astrum Repository Truststore** in the **Proxy** section of the repository |
| The email server | **Use Astrum Trust Store** under **Settings → System → Email Server** |
| An [LDAP server](authentication.md#ldap) over `ldaps` or StartTLS | **Use Astrum SSL Trust Store** on the connection of the server |
| The [OpenID Connect](authentication.md#openid-connect) provider: discovery, token, user info and JWKS requests | **Use the Astrum Repository Truststore** under **Settings → Security → OpenID Connect** |

With the option on, the stored certificates are trusted in addition to the system certificates; with it
off, only the system certificates count. Proxy repositories, LDAP servers and the OpenID Connect provider
always check that the certificate names the host they connect to, also for a stored certificate. The
email server checks the name only with **Server Identity Check** on.

Adding or removing a certificate applies to LDAP and OpenID Connect at once: the server drops its cache
of LDAP sign-ins and users and reads the provider's discovery document again, so a sign-in that relied on
a removed certificate fails from then on. Other servers on the same database pick the change up within
ten seconds, for LDAP at the next sign-in or lookup through a server that has the option on.

The system certificates are the CA bundle of the operating system; in the container image that is the
`ca-certificates` package of Debian. The environment variables `SSL_CERT_FILE` (a PEM file) and
`SSL_CERT_DIR` (directories) replace that bundle for the whole server, so a file named there must hold the
public CAs too.

Other connections do not use the truststore:

- a remote blob store trusts the certificate in its own **Trusted certificate** field, see
  [blob stores](blob-stores.md#remote-blob-stores);
- S3 blob stores and notification channels use the system certificates.

## Privileges

| Privilege | Allows |
|---|---|
| `as-ssl-truststore-read` | Seeing the truststore and loading the certificate of a server, also through **View Certificate** |
| `as-ssl-truststore-create` | Adding certificates |
| `as-ssl-truststore-delete` | Removing certificates |
| `as-ssl-truststore-all` | All of the above |

`as-admin` has them all.

## REST API

| Request | Does |
|---|---|
| `GET /service/rest/v1/security/ssl?host=<host>&port=<port>` | Loads and describes the certificate a server presents; `host` may also be `host:port` or a URL, `port` is optional, and `protocolHint=smtp` forces `STARTTLS` |
| `GET /service/rest/v1/security/ssl/truststore` | Lists the certificates |
| `POST /service/rest/v1/security/ssl/truststore` | Adds the PEM certificate in the body; `201`, or `409` when it is already there |
| `DELETE /service/rest/v1/security/ssl/truststore/<id>` | Removes a certificate; the ID is its SHA-1 fingerprint, with or without colons |

```sh
curl -u "admin:$ASTRUM_PASSWORD" --data-binary @corporate-ca.pem \
  https://astrum.example.com/service/rest/v1/security/ssl/truststore

curl -u "admin:$ASTRUM_PASSWORD" -X DELETE \
  https://astrum.example.com/service/rest/v1/security/ssl/truststore/C83E08A1B2...
```

Each certificate is returned with its `id`, `fingerprint`, `pem`, `serialNumber`, the common name,
organization and organizational unit of subject and issuer, and `issuedOn` and `expiresOn` in
milliseconds since 1970.
