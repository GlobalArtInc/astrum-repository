# Go

A proxy of the public module proxy, set once:

```shell
go env -w GOPROXY=https://astrum.example.com/repository/go-proxy,direct
```

Modules of a hosted repository are not in the public checksum database:

```shell
go env -w GOPROXY=https://astrum.example.com/repository/go-hosted,direct
go env -w GONOSUMDB=example.com/acme/tool
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file v1.4.0.zip \
  https://astrum.example.com/repository/go-hosted/example.com/acme/tool/@v/v1.4.0.zip
```

Upper-case letters in module paths and versions are written as `!` and the lower-case letter, as the
module proxy protocol requires.
