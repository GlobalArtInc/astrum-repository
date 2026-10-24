# Swift

Swift Package Manager resolves packages from a registry by identifier, `scope.name`, since Swift 5.7 and
publishes to one since Swift 5.9:

```shell
swift package-registry set --global https://astrum.example.com/repository/swift-group/
swift package-registry login https://astrum.example.com/repository/swift-group/ --username ci-deployer
```

and in `Package.swift`: `.package(id: "mona.LinkedList", from: "1.1.0")`. Add `--scope mona` to `set`
to use the registry only for one scope. Publish from the package directory to a hosted repository, or to
a group whose `writableMember` (set over the REST API) is hosted; `package-metadata.json` next to
`Package.swift` becomes the
release metadata:

```shell
swift package-registry publish mona.LinkedList 1.1.0 \
  --url https://astrum.example.com/repository/swift-hosted/
```

Swift only uses HTTPS registries. For a server on plain HTTP add `--allow-insecure-http` to `set` and
`publish`; `login` always switches to HTTPS, so put the credentials into `~/.netrc` instead
(`machine astrum.example.com login ci-deployer password …`). A Swift proxy caches another Swift registry;
its remote URL is the registry's base URL. Identifier lookups (`swift package resolve
--replace-scm-with-registry`) work for releases whose metadata lists `repositoryURLs`.
