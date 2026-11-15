# npm, pnpm, Yarn

Activate the **npm Bearer Token Realm** first. Then:

```shell
npm config set registry https://astrum.example.com/repository/npm-group/
npm login --registry=https://astrum.example.com/repository/npm-group/ --auth-type=legacy
```

`npm login` stores a token in `~/.npmrc`. For CI write a token directly, an npm token or an
[access token](../configure/authentication.md#access-tokens):

```ini
registry=https://astrum.example.com/repository/npm-group/
//astrum.example.com/repository/npm-group/:_authToken=${NPM_TOKEN}
//astrum.example.com/repository/npm-hosted/:_authToken=${NPM_TOKEN}
```

and publish with `npm publish --registry=https://astrum.example.com/repository/npm-hosted/`, or set
`publishConfig.registry` in `package.json`. A token for a user can also be requested without a browser:

```shell
curl -X PUT -H 'Content-Type: application/json' \
  -d '{"name":"ci-npm","password":"…"}' \
  https://astrum.example.com/repository/npm-hosted/-/user/org.couchdb.user:ci-npm
```

## Two-factor authentication

A user with a [second factor](../configure/two-factor.md) that is an authenticator app or recovery codes is
asked for a one-time password by `npm login`:

1. `npm login` sends the user name and password. With the right password and no code, the server answers
   `401` with `WWW-Authenticate: OTP`, and npm asks for the one-time password.
2. npm repeats the request with the header `npm-otp: <code>`. Six digits are checked as an authenticator
   code; any other value is checked as a recovery code.
3. A right code issues the token, as before.

The server reads the header only on `npm login`. Publishing with the token that login issued needs no code,
so `npm publish --otp` does not change what the server checks.

These answers refuse the sign-in:

| Answer | When |
|---|---|
| `401` with `WWW-Authenticate: OTP` | The password is right and no code was sent, or the code is wrong or already used |
| `401` with `the user name or password is wrong` | The password is wrong |
| `429` with `Retry-After` | The authenticator app is locked after too many wrong codes |
| `401` asking the user to set up a second factor in the web interface | The account must have a factor and has none yet |
| `401` with `this account has passkeys only; sign in with an access token` | The account has passkeys and no authenticator app or recovery code. Passkeys are not accepted here; use an [access token](../configure/authentication.md#access-tokens) |

Token issue is for local accounts only. A directory account, including one whose user name equals the name
of a local user, gets `401` and the message that the account signs in through a directory, so it uses an
[access token](../configure/authentication.md#access-tokens) instead. Such an account has an access token only
when the operator has switched on the `external_tokens` option (see [access tokens](../configure/authentication.md#access-tokens)).
