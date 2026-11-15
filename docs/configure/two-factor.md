# Two-factor authentication

A second factor asks a local user for a second proof after the password, when the user signs in to the web
interface: a code from an authenticator app, a passkey, or a recovery code. The administrator decides who
must have one. This page covers the methods, how users set them up, the sign-in steps, the administrator
policy, and the limits you should know before you require it.

## What it protects

- **Password sign-in to the web interface** and the REST sessions it opens. A user with a second factor
  signs in with the password, then with a code or a passkey.
- **Password sign-in on other interfaces** for a user with a second factor. HTTP Basic authentication
  with the password is refused, with the same answer as a wrong password, because a client cannot be
  asked for a code. Use an [access token](authentication.md#access-tokens) there instead.
- **Not covered:** access tokens, the tokens of format clients (npm, Docker, Conan, Terraform, NuGet,
  RubyGems, Cargo) and OAuth tokens of agents. These are the credentials of tools, and enrolling a second
  factor does not revoke them. `npm login` is the one place where a code is asked for in order to issue a
  token, and it takes an authenticator code or a recovery code, never a passkey; see
  [npm](../formats/npm.md#two-factor-authentication).
- **Not covered:** users of a directory. The second factor applies to local users only. LDAP users and
  OpenID Connect users sign in through the directory or the identity provider, which owns their factors.

## Methods

- **Authenticator app (TOTP).** Six-digit codes that change every 30 seconds, as in RFC 6238 with SHA-1.
  Any app that supports this standard works. A code is accepted once; a code from the previous or the next
  30-second step is accepted too, to allow for small differences of clock.
- **Passkeys.** A passkey is a key kept by this device or by a security key, and the browser asks the user
  to confirm it with a PIN, biometrics or the like. A passkey is a second factor here, not passwordless
  sign-in: the password is still asked first. The server requires user verification and refuses an answer
  without it. A passkey belongs to the relying party it was made for (see [the relying party](#relying-party-and-origins)),
  so it works only at the address that matches that relying party. Synced passkeys, kept by a password
  manager or a platform account, are accepted. The list marks a passkey **Backed up** when the authenticator
  reports that its key is backed up.
- **Recovery codes.** Ten one-time codes of the form `xxxxx-xxxxx`, issued when the first factor is set up
  (an authenticator app or a passkey) and on demand. Each code works once. They are shown once and can be
  downloaded as a text file.

A person may have an authenticator app, passkeys, or both. The recovery codes are a fallback for either.

## Set up and manage a factor

Users manage their own factor on the **Security** page, opened from the account menu (`/user/security`).

Every change needs the current password, including setting up, removing and regenerating, and a passkey
set-up asks for it twice: when the set-up starts and when the browser has answered. Renaming a passkey does
not need it. A wrong password counts as a failed sign-in.

- **Set up an authenticator app.** Enter the password. Scan the QR code with the authenticator app, or enter
  the secret key by hand. Enter a code from the app to confirm. The setup must be confirmed within 10 minutes
  of starting. After the confirmation the ten recovery codes are shown once; save them before you leave the page.
- **Add a passkey.** Choose **Add a passkey**, enter a name of 1 to 64 characters without control characters,
  and enter the current password. The browser then asks you to confirm the passkey on this device or with a
  security key. The set-up must be answered within 10 minutes. The same credential cannot be registered
  twice, by the same person or by another one.
- **Passkey list.** Each passkey shows its name, a **Backed up** badge when it is set, the date it was
  created, and the date it was last used or **Not used yet**. A passkey made for another relying party shows
  **Not usable here** and an explanation; it stays listed but cannot be used at this address.
- **Rename a passkey.** Choose the new name. No password is needed.
- **Remove a passkey.** Enter the password. A passkey that is the only factor of the account cannot be removed
  while a second factor is required for it: add another factor first. When a removed passkey was the last factor and there is no
  authenticator app, the recovery codes are removed with it.
- **Limit.** A person may hold up to 10 passkeys, usable or not.
- **First factor.** When the first factor is a passkey, the ten recovery codes are shown after the passkey is
  added, as they are after an authenticator app. A passkey added after an authenticator app brings no codes.
- **Browser without WebAuthn.** When the browser has no WebAuthn (no `PublicKeyCredential`), the Security page
  shows no passkey section, and the sign-in step does not offer the passkey button.
- **No usable relying party.** The passkey section says that passkeys are not available here, because this
  server has no base URL to make them for, and no passkey can be added. Ask an administrator to set the base URL.
- **Passkeys turned off.** When an administrator turns passkeys off, the section says that they are turned off and
  no passkey can be added. Passkeys already added keep working at sign-in.
- **Clock.** If the code does not match because the device clock is off, the confirmation says roughly how
  many minutes the clock is off.
- **Regenerate recovery codes.** The current set stops working at once.
- **Remove the authenticator app.** Removal is refused while a second factor is required for the account and
  no other factor is set up; a passkey counts as another factor. The recovery codes are removed with the app
  unless a passkey is set up.

Access tokens and API keys cannot manage a second factor. The account's factor endpoints answer 400 for an
account from a directory.

## Sign-in with a second factor

The web interface signs in in two steps.

1. The user sends the user name and password to `POST /service/rest/internal/session`. For a user with a
   factor the answer is HTTP 200 with `"authenticated": false` and a `secondFactor` object that lists the
   methods the account can use now, in this order: `totp`, `passkey`, `recovery`. A `passkey` method is listed
   only when one of the account's passkeys belongs to the current relying party; `recovery` only while recovery
   codes are left. The answer also says when the sign-in step expires. The server sets the cookie
   `astrum_signin` (HttpOnly, SameSite=Strict, valid for 5 minutes, sent only to the sign-in path). No
   session is opened yet.
2. The user answers the step:
   - with a code: `POST /service/rest/internal/session/second-factor` with the body
     `{"method": "totp", "code": "<code>"}` or `{"method": "recovery", "code": "<recovery code>"}`;
   - with a passkey: first `POST /service/rest/internal/session/second-factor/passkey-options` (no body; it
     answers `ceremony` and `publicKey`, the WebAuthn request options, with `Cache-Control: no-store`), then the
     browser prompt, then `POST /service/rest/internal/session/second-factor` with the body
     `{"method": "passkey", "ceremony": "<ceremony>", "credential": <what the browser returned>}`.

   A right answer opens the session.

The web interface chooses the first method to show in this order: the code field when the account has an
authenticator app, the passkey button when a passkey is offered, the recovery code field otherwise. An account
with only passkeys therefore sees the passkey button first, with a link to a recovery code below it. An account
with an authenticator app sees the code field first, and the passkey button beside it. The passkey button is not
offered in a browser without WebAuthn; if no method remains, the page says that none of the second factors can be
used on this address.

A cancelled browser prompt, which the browser reports as a refused request, changes nothing: nothing is sent
to the server, nothing is counted, and the sign-in step stays open. The user can try the passkey again or enter
a code.

The sign-in step fails in these ways:

| Answer | Meaning |
|---|---|
| 401 with code `second-factor-invalid` | The code is wrong or was already used, or the passkey was not accepted or was already used. The user can try again. |
| 401 with code `sign-in-expired` | The sign-in step is gone: it expired after 5 minutes, it was closed after five wrong answers, or the cookie is missing. The user starts again with the password. |
| 403 with code `passkeys-unusable` | Answer to the password step: none of the account's passkeys belongs to the current relying party, and it has no authenticator app and no recovery codes. See [resetting](#resetting-a-second-factor). |
| 409 | The relying party cannot be resolved any more, so no passkey ceremony can be started. |
| 429 with `Retry-After` | The authenticator app is locked after too many wrong codes. See [limits](#limits-you-should-know). |
| 429 without `Retry-After` | Too many failed sign-ins for this user from this node. Wait a few minutes. |

### Replay, expiry and attempts

- A passkey ceremony is single use. The first answer sent to it is used up, whether it is right or wrong, so a
  refused passkey needs a new call to `passkey-options` before the next try. A replayed answer is refused
  because its ceremony is gone, and the signature counter of the passkey would not increase either.
- A passkey ceremony expires with the sign-in step, 5 minutes after it was created. A set-up ceremony expires
  10 minutes after it was started.
- A sign-in step accepts five second-factor answers. After the fifth wrong one the step is closed and the user
  starts again with the password.

## Administrator policy

Administrators set the policy on **Settings → Security → Two-factor authentication**
(`admin/security/two-factor`). The page needs the `astrum:*` right.

The policy holds:

- **Methods.** Authenticator apps (`totp`) and passkeys (`passkeys`), each on or off. Keep at least one of them
  usable while anyone is required to have a factor: a policy that requires a factor is refused while no method
  can be enrolled (see [below](#why-a-required-factor-needs-a-method)). Turning authenticator apps off stops
  new set-ups of apps. Turning passkeys off stops new passkeys: the add button is hidden and the section says they are turned off.
  Factors that are already set up keep working; a passkey that is already added still signs in while it belongs
  to the current relying party.
- **Relying party.** The `relyingParty` object, described below.
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

### Relying party and origins

Passkeys are bound to a relying party: a domain name, and the web origins that may use passkeys for it. The
relying party comes from the configured base URL and the policy, never from the `Host` header of a request.

- **Id.** When `relyingParty.id` is not set, the id is the host of the base URL, in lower case. When it is set,
  it must be lower case, must be the host of the base URL or a parent domain of it, and the policy refuses it
  while no base URL is set. A single-label id is accepted only when it is the host itself, such as `localhost`.
  An IP address cannot be used, and neither can a top-level domain as the id of another host.
- **Origins.** The origin of the base URL is always allowed. `relyingParty.origins` adds more, at most 20. Each
  must be written exactly as `https://host[:port]` or `http://localhost[:port]`, in lower case, with no path,
  query, user information, default port or trailing slash, and at most 255 bytes. Each origin's host must be the
  id or a subdomain of it. `http://localhost` is allowed for local testing only.
- **Base URL.** It must be an `http` or `https` URL with a host name. An IP address, and an `http` URL whose host
  is not `localhost`, cannot be used for passkeys.

Passkeys are **unavailable** when there is no usable relying party: no base URL and no id, a base URL that is an
IP address, or an `http` base URL other than `http://localhost`. Then the Security page shows the unavailable
message, no passkey can be added, and passkeys are not offered at sign-in. Authenticator apps keep working.

### What happens when the base URL or the relying party changes

A passkey keeps the id it was made with. When the base URL or the relying party id changes so that the id
changes, the passkeys made for the old id stop working:

- they are listed with **Not usable here**, and the sign-in step does not offer them,
- a person whose only factor was such a passkey cannot sign in with it, and
- the server writes the audit event `passkey.stranded` with the number of passkeys and of people affected; the
  object of the event is `relyingParty`. It is written once for each change of the id that strands passkeys.

To recover, either restore the old base URL or relying party id (the passkeys work again), or reset the factor of
each affected person, which deletes their passkeys so they can add new ones.

### Why a required factor needs a method

A policy that requires a factor for anyone is saved only when a method can be enrolled. The save is refused
(400) with one of these messages:

- on field `totp`: `no second factor method can be enrolled: turn on authenticator apps (totp), or turn on passkeys and set an https base URL or a relying party id`, when both methods are off;
- on field `passkeys`: `passkeys need a usable relying party: set an https base URL (or http://localhost), or a relying party id with an origin`, when passkeys are on but no relying party is usable.

Any `relyingParty` that cannot be resolved also refuses the save, whether or not passkeys are on. The messages
name the field and the reason, for example `the base URL is not usable for passkeys: its host is an IP address`.

If the methods are changed later, for example the base URL is removed while authenticator apps are off, the
policy is not checked again. Users who would have to set up a factor are then let in without one, and the server
logs a warning. Keep one method usable.

## Restricted sessions

A user who must have a factor and has none, after the deadline, can still sign in with the password. The
session is restricted. A restricted session may only:

- read and close its own session (`GET` and `DELETE` on `/service/rest/internal/session`),
- read the account and the interface state (`GET` on `/service/rest/internal/ui/user` and
  `/service/rest/internal/ui/state`), and the server status (`GET` on `/service/rest/v1/status`), and
- use the second factor pages of the account (`/service/rest/internal/ui/user/two-factor`, which include the
  passkey pages under it).

Any other request answers 403 with the code `two-factor-enrollment-required`. The interface state reports
`"restricted": true` and carries no permissions, and the web interface shows only the Security page to such a
session. Setting up a factor, an authenticator app or a passkey, lifts the restriction for that session.

## Resetting a second factor

An administrator resets the factor of a user who lost the device. The reset:

- deletes the authenticator app, the passkeys, the recovery codes, any set-up or sign-in step in progress, and
  the lock,
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

**Passkeys that can no longer be used.** A person whose passkeys were made for another relying party, with no
authenticator app and no recovery codes left, cannot sign in. The password step answers 403 with the code
`passkeys-unusable` and the message that none of the passkeys works on this server any more. Such a person
must ask an administrator, who resets the factor, or who restores the old base URL.

**Delegated administrators.** A caller with `astrum:users:update` but without `astrum:*` cannot reset the
factor of an administrator, cannot change the requirement of an administrator, and cannot set **Exempt** for
anyone. Those actions need `astrum:*`.

## Audit

Changes to factors, the policy and the requirements are audited in the `security.twofactor` domain. Codes,
secrets and recovery codes are never written to the log. The events for passkeys are:

| Event | Written when | Contents |
|---|---|---|
| `passkey.enrolled` | a passkey is added | user id, passkey id, whether it is backed up |
| `passkey.renamed` | a passkey is renamed | user id, passkey id |
| `passkey.removed` | a passkey is removed | user id, passkey id |
| `passkey.counter` | a passkey signs with a counter that did not increase | user id, passkey id when known |
| `passkey.stranded` | the relying party id changes and some passkeys cannot be used any more | the number of passkeys and of people; the object is `relyingParty` |
| `verify.failed` | a second step fails | user id and method (`passkey` for a passkey) |

A passkey set-up that is the first factor also writes `recovery.generated`. A successful sign-in with a passkey
is recorded as a login whose `secondFactor` is `passkey`. None of these events holds a credential id, a public
key, a challenge or a signature.

## Directory users

The policy and the requirements apply to local users only. LDAP and OpenID Connect users are not asked for a
second factor by the server. The account's own factor endpoints answer 400 for them, because the factor is
managed in the directory or the identity provider.

## Limits you should know

- **Attempts are counted in two places.** Ten failed codes of the authenticator app lock it for the user,
  in PostgreSQL, so every node sees the same lock. The lock lasts 15 minutes and doubles each time it is
  set again, up to 24 hours. A few guesses that are already in flight can pass the threshold. The failed
  sign-in counter that blocks a user for a few minutes is kept in the memory of each node, so with N nodes
  an attacker gets about N times as many attempts before it blocks. Passkeys are not locked this way: see
  [passkey limits](#passkey-limits).
- **The password holder can lock the owner out.** Anyone with the password can give ten wrong codes and lock
  the authenticator app for the lock window, and the right code is refused while it lasts. A right recovery
  code lifts the lock. Recovery codes are not locked by the database counter, so keep a set of them
  somewhere safe. A passkey still works while the authenticator app is locked.
- **The cookie flag follows the headers.** The `Secure` flag of the session and sign-in cookies is set when
  the request carries `X-Forwarded-Proto: https` or a `Forwarded` header with `proto=https`, whoever sent it.
  Configure the reverse proxy to overwrite both headers, so a client cannot choose them. See
  [reverse proxy and TLS](reverse-proxy.md).
- **Set-up and sign-in steps expire.** An authenticator set-up lasts 10 minutes, a passkey set-up 10 minutes too,
  and a sign-in step 5 minutes. A step is also closed after five wrong answers. Expired steps are removed when a
  new one is stored.
- **The secrets key matters for the authenticator app.** The authenticator secret is kept sealed with the
  [secrets key](runtime.md#secrets-key). A database restored with a different key cannot check codes: every
  attempt counts as wrong and locks the app. Recovery codes still work, and an administrator reset clears the
  secret. Passkeys are stored as public keys and do not depend on the key. Back up the key together with the
  database, as described in [backup and restore](../operate/backup-restore.md).

### Passkey limits

- **Attestation is not checked.** The server asks for no attestation and does not check one, so any
  authenticator is accepted, and the server does not know which kind of authenticator a passkey came from.
- **Counters.** A signature counter that does not increase is refused and audited as `passkey.counter`. A counter
  that stays at zero, which many synced passkeys report, is accepted every time.
- **Cross-origin use is refused.** An answer that the browser marks as made in a cross-origin context is refused,
  and an answer must come from one of the allowed origins of the relying party.
- **Failures and passkeys.** A wrong passkey answer counts as a failed sign-in in the memory budget described
  above, and it writes `verify.failed` with the method `passkey`. It does not write the database lock of the
  authenticator app. Asking for options is not a failure; while the budget is exhausted, asking answers 429 too.
  The budget is ten failures of one user within a five-minute window that starts at the first of them: wrong
  passwords at sign-in or on the Security page, wrong codes, wrong recovery codes and refused passkeys. The user
  is blocked until that window ends, and a successful sign-in clears the count.
- **Cookies and sessions** are those of the authenticator app: the same sign-in cookie, the same session, and the
  same restricted session rules.
- **Passkeys are not used elsewhere.** A passkey is not accepted by `npm login`, not for access tokens, and not for
  HTTP Basic authentication. `npm login` for a user with a usable passkey asks for an authenticator code or a
  recovery code, if the account has one; an account with passkeys only is refused, with a message to use an
  [access token](authentication.md#access-tokens) instead. See [npm](../formats/npm.md#two-factor-authentication).

## Troubleshooting

- **The browser prompt never appears.** The page must be served over `https`, or from `http://localhost` for
  local testing; without it the browser has no WebAuthn and the passkey button is not shown. Also open the page at
  the address the relying party allows: a different address makes the browser refuse the request, and the page
  says that the passkey could not be used.
- **"Passkeys are not available here".** The server has no usable relying party. Set the base URL to an `https`
  address with a host name, not an IP address, or to `http://localhost` for local testing. See
  [relying party and origins](#relying-party-and-origins).
- **A passkey shows "Not usable here".** It was made at another address of this server, or before the base URL
  changed. Open the address it was made for, or add a new passkey at this address and remove the old one.
- **A person is locked out.** When no passkey works and no recovery code is left, an administrator resets the
  factor. If the cause is a changed base URL, restoring the old one is enough. If the last administrator is
  locked out, use `astrum two-factor reset` on the server.
