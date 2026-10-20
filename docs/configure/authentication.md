# Users, realms and sign-in

## Realms

A realm is a way to authenticate. Active realms are managed under **Settings → Security → Realms**;
they are tried in the listed order.

| Realm | Needed for |
|---|---|
| Local Authenticating Realm | Local users with a password. Active by default; keep it active, the `admin` account lives here |
| Access Token Realm | [Access tokens](#access-tokens) as passwords and bearer tokens. Active by default |
| LDAP Realm | Sign-in with a directory account (Active Directory, OpenLDAP and others); activated automatically when the first LDAP server is added |
| OpenID Connect Realm | Sign-in through an OpenID Connect provider; activated automatically when OpenID Connect is configured |
| Docker Bearer Token Realm | Anonymous `docker pull` and Docker token authentication |
| npm Bearer Token Realm | `npm login` and `_authToken` in `.npmrc` |
| NuGet API-Key Realm | `nuget push` with the user's API key |
| RubyGems API Key Realm | `gem signin` and `gem push` with the user's API key |
| Terraform Bearer Token Realm | Tokens in `credentials` blocks of the Terraform CLI configuration |
| Default Role Realm | Grants one role to every signed-in user, configured as a capability |

## Users, roles and privileges

- **Privileges** grant one action. Every repository gets view privileges automatically, named
  `as-repository-view-<format>-<repository>-<action>` with the actions `browse`, `read`, `add`, `edit`
  and `delete`; `*` stands for all formats, repositories or actions.
- **Roles** bundle privileges and other roles. `as-admin` has every privilege; `as-anonymous` is what
  anonymous users get.
- **Users** come from a source: **Local** users are created in Astrum, **LDAP** users are read from the
  directory, **OpenID Connect** users appear after their first sign-in. The **All** source on the Users
  page lists them all; LDAP users show up once you search for them.

**Content selectors** (**Settings → Repository → Content Selectors**) pick out content by format and path
with an expression, for example `format == "raw" and path =^ "/public/"`; a content selector privilege
then grants actions on exactly that content in the repositories it names. The page previews what a
selector matches.

A typical setup is one role per team and repository group, for example `deployer-team-a` with
`as-repository-view-maven2-team-a-releases-*`, and a separate local service user per CI system.

## Access tokens

An access token is a password for tools. Every user can create tokens under **Account menu → Access
Tokens** after signing in with a password:

- a name, an access level (**Read only** for downloading, **Read and write** for everything the user can
  do) and a validity of 1 to 3650 days, or none;
- the token is `astrum_` followed by 40 characters and is shown once; the server stores only a hash of it;
- at most 100 tokens per user, each revocable at any time without touching the password.

Clients use a token wherever they would use the password: as the password of Basic authentication
(`mvn`, `pip`, `docker login`, the netrc file of Nix, `curl -u user:astrum_...`), or as `Authorization: Bearer astrum_...`
(`dart pub token add` stores it for that; r10k, Bolt and `pdk release` send their Forge token so, and Vagrant
sends `VAGRANT_CLOUD_TOKEN` so to the host of `VAGRANT_SERVER_URL`), or as `Authorization: Token astrum_...`,
the way `ansible-galaxy` sends its token. A bare `Authorization: astrum_...` works too, so `forge_authorization`
in `puppet.conf` can hold the token alone.
Tokens work while the Access Token Realm is active. They are the way for OpenID Connect users, who
have no password, to use command-line tools, and they keep directory passwords out of CI for LDAP users.
A token cannot create or revoke tokens; that needs a sign-in with the password. A read-only token cannot
change the account either. Users with the privileges `astrum:accesstokens:read` and
`astrum:accesstokens:delete` can see and revoke the tokens of others.

## Anonymous access

**Settings → Security → Anonymous Access** decides whether requests without credentials are allowed
and as which user (`anonymous` by default, with the `as-anonymous` role). Turn it off to require
credentials everywhere, or keep it on and narrow `as-anonymous` to the repositories that should be
public.

Berkshelf, Policyfiles and the reading `knife supermarket` commands send no credentials to Chef
Supermarket repositories, so the anonymous user needs read access to them. `knife supermarket share`
and `unshare` sign their requests with a Chef client key (the `X-Ops-*` headers), which this server does
not check; they are answered `401`, and cookbooks are published with Basic authentication instead (see
[Chef Supermarket](../use/clients.md#chef-supermarket)).

## LDAP

Astrum can authenticate users against one or more LDAP directories. Servers are managed under
**Settings → Security → LDAP**, which needs the `as-ldap-*` privileges (`as-admin` has them).

1. Click **Create connection** and fill in the connection:
   - **Protocol**, **Host** and **Port**: `ldap` (389) or `ldaps` (636). For `ldaps` the server
     certificate must be trusted by the certificate store of the host (the container image uses the
     one of Debian), or added under **Settings → Security → SSL Certificates** with
     **Use the Astrum Repository truststore** checked. The host name must match the certificate.
   - **Search base DN**, for example `dc=example,dc=com`.
   - **Authentication method**: **Simple Authentication** with the DN and password of a service account
     that may search the directory (recommended), **Anonymous Authentication**, **DIGEST-MD5** or
     **CRAM-MD5** (with an optional **SASL realm**).
   - **Connection rules**: the timeout in seconds, how long to wait before retrying a server that was
     put aside, and how many failures in a row put it aside.

   **Verify connection** binds with these settings and checks that the search base exists.
2. Choose the user and group settings, or start from a template (Active Directory, Posix with static
   or dynamic groups, generic LDAP):
   - **User relative DN** is added in front of the search base, for example `ou=people`; tick
     **User subtree** to also search below it.
   - **Object class**, **User filter** (an extra LDAP filter, for example
     `(memberOf=cn=astrum-users,ou=groups,dc=example,dc=com)`), and the **User ID**, **Real name** and
     **Email** attributes.
   - **Password attribute**: leave empty to check passwords by binding as the user, which is what most
     directories expect. When set, the password is compared with that attribute instead; plain text and
     `{SHA}`, `{SSHA}`, `{SHA256}`, `{SSHA256}`, `{SHA512}`, `{SSHA512}`, `{MD5}`, `{SMD5}` values are
     understood.
   - **Map LDAP groups as roles**: **dynamic** groups are read from an attribute of the user such as
     `memberOf` (the first part of each group DN becomes the group name); **static** groups are entries
     of the group object class whose member attribute contains the user, written with the member format
     (`${username}` or `${dn}`, for example `uid=${username},ou=people,dc=example,dc=com`).

   **Verify user mapping** lists the first 20 users with their groups, **Verify login** tries a user's
   password.
3. Save. The LDAP realm is activated if it was not already. Several servers are tried in the listed
   order (**Change order**); the first server that knows the user decides.

### Roles for LDAP users

LDAP users get roles in the same ways as OpenID Connect users:

- **groups**: create a role of type **External Role Mapping** with the source **LDAP**, pick the group as
  **Mapped Role** and add privileges; every member of the group gets the role. A local role whose ID is
  the group name works the same way;
- **role mapping**: on the Users page, search the **LDAP** source, select the user and assign roles.

### Caching and availability

Users, their groups and successful sign-ins are cached for five minutes (setting `ldap.cache_ttl` in
`astrum.toml` or `ASTRUM_LDAP_CACHE_TTL`, for example `1m`; `0` turns the cache off), so a password or
group change in the directory can take that long to apply. Passwords are only kept as a
salted hash in memory. **Clear cache** on the LDAP page drops the cache at once.

When a server cannot be reached, the next one is tried. A server that fails as many times in a row as
the connection rules allow is skipped for the retry delay. While no server answers, LDAP users cannot sign
in, but they are not treated as deleted: their tokens and role mappings stay.

## OpenID Connect

Any provider that publishes an OpenID Connect discovery document works: Keycloak, Authentik, Zitadel,
Google, Microsoft Entra ID, Okta, Auth0 and others.

1. Register a client in the provider:
   - type: confidential (with a client secret) or public (PKCE only);
   - redirect URI: `https://<astrum host>/oidc/callback` (the settings page shows the exact value);
   - scopes: `openid profile email`, plus whatever scope makes the provider include groups.
2. In Astrum open **Settings → Security → OpenID Connect** and fill in:
   - **Issuer URL**, for example `https://sso.example.com` or `https://keycloak.example.com/realms/main`;
   - **Client ID** and, for a confidential client, **Client Secret**;
   - **Groups Claim** if the provider sends groups, for example `groups` or `realm_access.roles`.
3. Save. The server reads the provider's discovery document, rejects the settings if it cannot, and
   activates the OpenID Connect realm. The sign-in page now shows a single sign-on button.

Optional settings:

| Setting | Use |
|---|---|
| Username Claim | Claim that becomes the user ID. Default: `preferred_username`, falling back to `sub` |
| First/Last Name, Email Claim | Defaults `given_name`/`name`, `family_name`, `email` |
| Authorization, Token, JWKS, Logout URL | Override the endpoints from discovery |
| JSON Web Keys | Inline JWKS for providers without a JWKS endpoint |
| JWT Signature Algorithm | Accept only one of RS256, RS384, RS512, ES256, ES384, ES512 |
| Exact Match Claims | JSON object of claims that must have the given value, for example `{"hd": "example.com"}` |
| Authorization / Token Request Custom Parameters | Extra request parameters as a JSON object, for example `{"prompt": "login"}` or a different `scope` |
| Use the Astrum truststore | Trust the provider's certificate from **Settings → Security → SSL Certificates** |

### Roles for OpenID Connect users

OpenID Connect users have no roles until they get them in one of two ways:

- **groups**: a group in the groups claim that has the same name as an Astrum role grants that role,
  for example a provider group `as-admin`;
- **role mapping**: on the Users page, select the user (source **OpenID Connect**) and assign roles.
  The user must have signed in once to appear in the list.

Both can be combined. Group roles follow the provider on every sign-in.

### Sign-out

Signing out ends the Astrum session. The provider session is left alone, so the next sign-in may
complete without asking for a password.

## Build tools and CI

Command-line clients (`mvn`, `npm`, `docker`, `pip`, `helm` and the rest) send a user name and password
or a token; they cannot follow the OpenID Connect browser flow. LDAP users can use their directory
password with them, and OpenID Connect users use an [access token](#access-tokens). For CI and scripts
create a local service user with only the roles it needs, give it an access token and store the token
in the CI system's secret store. Format-specific tokens (npm, Docker, NuGet, RubyGems, Terraform) can
be issued for these users as described in [connect build tools](../use/clients.md).
