# pub (Dart and Flutter)

The hosted URL of a pub repository is its address without the trailing slash, for example
`https://astrum.example.com/repository/pub-group`. `dart pub` and `flutter pub` send a token as
`Authorization: Bearer`; use an [access token](../configure/authentication.md#access-tokens) of your user
and add it once per repository (the command asks for the token, `--env-var ASTRUM_TOKEN` reads it from
the environment in CI):

```shell
dart pub token add https://astrum.example.com/repository/pub-group
```

A repository that anonymous users may not read answers 401 to a client without a token, and the client
prints where to create one. `dart pub` sends tokens only over HTTPS, except to `localhost`.

A dependency from the repository in `pubspec.yaml`:

```yaml
dependencies:
  mylib:
    hosted: https://astrum.example.com/repository/pub-group
    version: ^1.2.0
```

`dart pub add 'mylib:{hosted: https://astrum.example.com/repository/pub-group, version: ^1.2.0}'` writes
the same lines. To take every package through a group of a hosted repository and a proxy of
`https://pub.dev`, set `PUB_HOSTED_URL=https://astrum.example.com/repository/pub-group` for `dart pub` and
`flutter pub`.

To publish, name the hosted repository in the `pubspec.yaml` of the package and run `dart pub publish`
(`--force` skips the question):

```yaml
publish_to: https://astrum.example.com/repository/pub-hosted
```

The repository refuses an archive without `pubspec.yaml` at its top, a name that is not a lower-case
Dart identifier and, unless the write policy allows redeploying, a version that it already has with
other content; `dart pub publish` shows the reason. Archives also go up through the components API in
the field `pub.asset`:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F pub.asset=@mylib-1.2.0.tar.gz \
  "https://astrum.example.com/service/rest/v1/components?repository=pub-hosted"
```

Retract a version or mark a package discontinued with the option paths of pub.dev; clients then skip
the retracted version and report the discontinued package:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -X PUT -H 'Content-Type: application/json' \
  -d '{"isRetracted": true}' \
  https://astrum.example.com/repository/pub-hosted/api/packages/mylib/versions/1.2.0/options
curl -u "ci-deployer:$ASTRUM_PASSWORD" -X PUT -H 'Content-Type: application/json' \
  -d '{"isDiscontinued": true, "replacedBy": "mylib2"}' \
  https://astrum.example.com/repository/pub-hosted/api/packages/mylib/options
```

Proxies keep the version listings, security advisories and package name completion data of the remote
as metadata, point `archive_url` at themselves and fetch each archive from the address the remote
listed, checking it against `archive_sha256`. Groups merge the listings of their members (the first
member wins for a version) and their advisories; publish to the hosted repository itself.
