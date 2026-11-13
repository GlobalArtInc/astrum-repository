# Two-factor authentication

A second factor asks a local user for a code from an authenticator app after the password, when the
user signs in to the web interface. The administrator decides who must have one. This page covers the
methods, how users set them up, the sign-in steps, the administrator policy, and the limits you should
know before you require it.

## What it protects

- **Password sign-in to the web interface** and the REST sessions it opens. A user with a second factor
  signs in with the password, then with a code.
- **Password sign-in on other interfaces** for a user with a second factor. HTTP Basic authentication
  with the password is refused, with the same answer as a wrong password, because a client cannot be
  asked for a code. Use an [access token](authentication.md#access-tokens) there instead.
- **Not covered:** access tokens, the tokens of format clients (npm, Docker, Conan, Terraform, NuGet,
  RubyGems, Cargo) and OAuth tokens of agents. These are the credentials of tools, and enrolling a second
  factor does not revoke them. `npm login` is the one place where a code is asked for in order to issue a
  token; see [npm](../formats/npm.md#two-factor-authentication).
- **Not covered:** users of a directory. The second factor applies to local users only. LDAP users and
  OpenID Connect users sign in through the directory or the identity provider, which owns their factors.

## Methods

- **Authenticator app (TOTP).** Six-digit codes that change every 30 seconds, as in RFC 6238 with SHA-1.
  Any app that supports this standard works. A code is accepted once; a code from the previous or the next
  30-second step is accepted too, to allow for small differences of clock.
- **Recovery codes.** Ten one-time codes of the form `xxxxx-xxxxx`, issued when an authenticator app is set
  up and on demand. Each code works once. They are shown once and can be downloaded as a text file.

Passkeys are not built yet, so no user can set one up in this version.

## Set up and manage a factor

Users manage their own factor on the **Security** page, opened from the account menu (`/user/security`).

Every change needs the current password, including setting up, removing and regenerating. A wrong password
counts as a failed sign-in.

- **Set up.** Enter the password. Scan the QR code with the authenticator app, or enter the secret key by
  hand. Enter a code from the app to confirm. The setup must be confirmed within 10 minutes of starting.
  After the confirmation the ten recovery codes are shown once; save them before you leave the page.
- **Other sessions.** Confirming a new factor signs the user out of every other session. The current
  session stays. Access tokens keep working.
- **Clock.** If the code does not match because the device clock is off, the confirmation says roughly how
  many minutes the clock is off.
- **Regenerate recovery codes.** The current set stops working at once.
- **Remove the authenticator app.** The recovery codes are removed with it. Removal is refused while a
  second factor is required for the account and no other factor is set up.

Access tokens and API keys cannot manage a second factor. The account's factor endpoints answer 400 for an
account from a directory.

## Sign-in with a second factor

The web interface signs in in two steps.

1. The user sends the user name and password to `POST /service/rest/internal/session`. For a user with a
   factor the answer is HTTP 200 with `"authenticated": false` and a `secondFactor` object that lists the
   methods (`totp`, `recovery`) and when the sign-in step expires. The server sets the cookie
   `astrum_signin` (HttpOnly, SameSite=Strict, valid for 5 minutes, sent only to the sign-in path). No
   session is opened yet.
2. The user sends the code to `POST /service/rest/internal/session/second-factor` with the body
   `{"method": "totp", "code": "<code>"}` or `{"method": "recovery", "code": "<recovery code>"}`. A right code
   opens the session.

The sign-in step fails in these ways:

| Answer | Meaning |
|---|---|
| 401 with code `second-factor-invalid` | The code is wrong or was already used. The user can try again. |
| 401 with code `sign-in-expired` | The sign-in step is gone: it expired after 5 minutes, five wrong codes were given, or the cookie is missing. The user starts again with the password. |
| 429 with `Retry-After` | The authenticator app is locked after too many wrong codes. See [limits](#limits-you-should-know). |
| 429 without `Retry-After` | Too many failed sign-ins for this user from this node. Wait a few minutes. |

## Administrator policy

Administrators set the policy on **Settings → Security → Two-factor authentication**
(`admin/security/two-factor`). The page needs the `astrum:*` right.

The policy holds:

- **Methods.** Authenticator apps on or off. Keep them on while anyone is required to have a factor: a
  policy that requires a factor is refused while no method is on, and turning the app off stops new set-ups.
  Factors that are already set up keep working.
- **Who must have a factor.** Nobody, everyone, administrators (users with `astrum:*`), or the holders of
  listed roles. The listed roles must exist.
- **Grace period.** From 0 to 90 days, 7 by default.

Each user also has a requirement on the user's page in the administration area:

- **Follow the policy** (`policy`), the default.
- **Always required** (`required`). The user must have a factor whatever the policy says.
- **Exempt** (`exempt`). The user is never asked for a factor. Exemptions are audited. Use them for service
  users that must keep signing in with a password, and not for people.

When a requirement starts to apply to a user, the user gets the grace period from the later of the day the
requirement was widened and the day the account was created. The day the requirement was last widened is
kept as `requiredSince`. Until the deadline the user only sees a banner. After it:

- the web interface sign-in opens a **restricted session** (see below), and
- HTTP Basic authentication with the password is refused.

The REST endpoints are:

| Method and path | Right | Purpose |
|---|---|---|
| `GET /service/rest/v1/security/two-factor` | `astrum:*` | Read the policy |
| `PUT /service/rest/v1/security/two-factor` | `astrum:*` | Replace the policy |
| `GET /service/rest/v1/security/users/{userId}/two-factor` | `astrum:users:read` | Status of a user's factor |
| `PUT /service/rest/v1/security/users/{userId}/requirement` | `astrum:users:update` | Body with a `requirement` field: `policy`, `required` or `exempt` |
| `DELETE /service/rest/v1/security/users/{userId}/two-factor` | `astrum:users:update` | Reset, see below |

## Restricted sessions

A user who must have a factor and has none, after the deadline, can still sign in with the password. The
session is restricted. A restricted session may only:

- read and close its own session (`GET` and `DELETE` on `/service/rest/internal/session`),
- read the account and the interface state (`GET` on `/service/rest/internal/ui/user` and
  `/service/rest/internal/ui/state`), and the server status (`GET` on `/service/rest/v1/status`), and
- use the second factor pages of the account (`/service/rest/internal/ui/user/two-factor`).

Any other request answers 403 with the code `two-factor-enrollment-required`. The interface state reports
`"restricted": true` and carries no permissions, and the web interface shows only the Security page to such a
session. Setting up a factor lifts the restriction for that session.

## Resetting a second factor

An administrator resets the factor of a user who lost the device. The reset:

- deletes the authenticator app, the recovery codes, any set-up in progress and the lock,
- signs the user out of every session, in the same database transaction, and
- sets the deadline to now if the user is required to have a factor, so the user must set one up at the
  next sign-in.

Access tokens that are already issued keep working.

Through the REST endpoint in the table above. The endpoint refuses a reset of the account you are signed in as
(400); another administrator resets it, or the command line does.

From the command line, on the server, with the same configuration as the server:

```shell
astrum two-factor reset <USER_ID>
```

This is the way to reset the last administrator's factor. It reads the configuration and the database
directly, so run it where the server runs. It signs the user out everywhere and records the reset in the
audit log under the CLI.

**Delegated administrators.** A caller with `astrum:users:update` but without `astrum:*` cannot reset the
factor of an administrator, cannot change the requirement of an administrator, and cannot set **Exempt** for
anyone. Those actions need `astrum:*`.

Changes to factors, the policy and the requirements are audited in the `security.twofactor` domain. Codes,
secrets and recovery codes are never written to the log.

## Directory users

The policy and the requirements apply to local users only. LDAP and OpenID Connect users are not asked for a
second factor by the server. The account's own factor endpoints answer 400 for them, because the factor is
managed in the directory or the identity provider.

## Limits you should know

- **Attempts are counted in two places.** Ten failed codes of the authenticator app lock it for the user,
  in PostgreSQL, so every node sees the same lock. The lock lasts 15 minutes and doubles each time it is
  set again, up to 24 hours. A few guesses that are already in flight can pass the threshold. The failed
  sign-in counter that blocks a user for a few minutes is kept in the memory of each node, so with N nodes
  an attacker gets about N times as many attempts before it blocks.
- **The password holder can lock the owner out.** Anyone with the password can give ten wrong codes and lock
  the authenticator app for the lock window, and the right code is refused while it lasts. A right recovery
  code lifts the lock. Recovery codes are not locked by the database counter, so keep a set of them
  somewhere safe.
- **The cookie flag follows the headers.** The `Secure` flag of the session and sign-in cookies is set when
  the request carries `X-Forwarded-Proto: https` or a `Forwarded` header with `proto=https`, whoever sent it.
  Configure the reverse proxy to overwrite both headers, so a client cannot choose them. See
  [reverse proxy and TLS](reverse-proxy.md).
- **Set-up and sign-in steps expire.** An authenticator set-up lasts 10 minutes, a sign-in step 5 minutes
  and five wrong codes. Expired steps are removed when a new one is stored.
- **The secrets key matters.** The authenticator secret is kept sealed with the [secrets key](runtime.md#secrets-key).
  A database restored with a different key cannot check codes: every attempt counts as wrong and locks the
  app. Recovery codes still work, and an administrator reset clears the secret. Back up the key together
  with the database, as described in [backup and restore](../operate/backup-restore.md).
