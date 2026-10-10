# Users, realms and sign-in

## Realms

A realm is a way to authenticate. Active realms are managed under **Settings → Security → Realms**;
they are tried in the listed order.

| Realm | Needed for |
|---|---|
| Local Authenticating Realm | Local users with a password. Keep it active, the `admin` account lives here |
| OpenID Connect Realm | Sign-in through an OpenID Connect provider; activated automatically when OpenID Connect is configured |
| Docker Bearer Token Realm | Anonymous `docker pull` and Docker token authentication |
| npm Bearer Token Realm | `npm login` and `_authToken` in `.npmrc` |
| NuGet API-Key Realm | `nuget push` with the user's API key |
| Conan Bearer Token Realm | `conan remote login` |
| Cargo Bearer Token Realm | `cargo login` tokens |
| Default Role Realm | Grants one role to every signed-in user, configured as a capability |

## Users, roles and privileges

- **Privileges** grant one action. Every repository gets view privileges automatically, named
  `as-repository-view-<format>-<repository>-<action>` with the actions `browse`, `read`, `add`, `edit`
  and `delete`; `*` stands for all formats, repositories or actions.
- **Roles** bundle privileges and other roles. `as-admin` has every privilege; `as-anonymous` is what
  anonymous users get.
- **Users** come from a source: **Local** users are created in Astrum, **OpenID Connect** users appear
  after their first sign-in. The **All** source on the Users page lists both.

A typical setup is one role per team and repository group, for example `deployer-team-a` with
`as-repository-view-maven2-team-a-releases-*`, and a separate local service user per CI system.

## Anonymous access

**Settings → Security → Anonymous Access** decides whether requests without credentials are allowed
and as which user (`anonymous` by default, with the `as-anonymous` role). Turn it off to require
credentials everywhere, or keep it on and narrow `as-anonymous` to the repositories that should be
public.

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
or a format-specific token; they cannot follow the OpenID Connect browser flow. For CI and scripts
create a local service user with a generated password and only the roles it needs, and store its
credentials in the CI system's secret store. Format-specific tokens (npm, NuGet, Conan, Cargo) can be
issued for these users as described in [connect build tools](../use/clients.md).
