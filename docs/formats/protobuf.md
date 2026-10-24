# Protobuf (Buf Schema Registry)

Hosted `protobuf` repositories are a registry of Protobuf modules for the [buf](https://buf.build) CLI: they
answer the `buf.registry.module.v1` services (modules, uploads, commits, labels, the dependency graph and
downloads) and `buf.registry.owner.v1.OwnerService` over the Connect protocol, which is what `buf push`,
`buf dep update`, `buf build` and `buf registry module ...` of buf 1.75 call. The format has hosted
repositories only. A proxy of another registry, groups, and modules that depend on modules of another
registry (such as `buf.build/googleapis/googleapis`) come later; buf asks such a registry through the
`v1beta1` services, which answer 501 here.

buf speaks HTTPS only and sends its calls to the host of the module name with no path in front, for
example `https://protos.example.com/buf.registry.module.v1.ModuleService/GetModules`. So the repository
needs a domain of its own: give it a [domain](../configure/reverse-proxy.md#a-domain-of-its-own-for-a-repository) with the whole domain as its
use, and send that domain to this server from the reverse proxy. The address under `/repository/` serves
the same calls, but buf cannot use it.

Create an [access token](../configure/authentication.md#access-tokens) and give it to buf, which sends it
as `Authorization: Bearer`; a read token is enough to pull, a write token also pushes. A repository that
anonymous users may not read answers `unauthenticated`, and buf prints how to log in:

```shell
export BUF_TOKEN=<access token>
```

Name the module after the host of the repository in `buf.yaml`, then push it. The first push with
`--create` creates the module; its owner (the part before the module name) comes into being with the first
module under it and is not a user of the server:

```yaml
version: v2
modules:
  - path: .
    name: protos.example.com/acme/greeter
```

```shell
buf push --create --create-visibility private
buf push --label release/v1
```

A push answers with the commit. Pushing the same files again gives the same commit, and a push without
`--label` moves the label `main`, the default label of the module, to the commit. Commits never change;
labels do, and `buf registry module label` and `buf registry module commit` list, archive and add them.
The default label cannot be archived. Another module of the repository goes into `deps` of
`buf.yaml`, and `buf dep update` pins its commit and digest in `buf.lock`:

```yaml
version: v2
deps:
  - protos.example.com/acme/greeter
```

```shell
buf dep update
buf build
buf registry module commit list protos.example.com/acme/greeter
buf registry module label list protos.example.com/acme/greeter
```

A module consists of its `.proto` files, the `LICENSE` file and one of `buf.md`, `README.md` and
`README.markdown` at the top; buf sends nothing else and the server keeps nothing else. The commit carries
the digest buf checks on every download, so a clean cache pulls the same bytes that were pushed. A push
that imports files of the modules named in `deps` records them as dependencies of the commit, and the
digest covers them. One push may carry several modules of a workspace that import each other.

Names follow the rules of the Buf Schema Registry: an owner has 1 to 48 lowercase letters, digits and
dashes and starts with a letter; a module has 2 to 100 of them or underscores; a label has up to 250
characters and is not a 32-digit hex number, which would be taken for a commit. Deleting a module with
`buf registry module delete` is refused while a commit of another module depends on one of its commits.
The visibility of a module is stored and reported, but who may read and write comes from the roles and
content selectors of the server alone: reading needs the read privilege, `Add` creates modules and
commits and `Edit` moves labels, and a repository with the write policy "Deny" takes no pushes.

Everything is stored as files of the repository, so the repository browser shows
`<owner>/<module>/module.json`, one `commits/<commit>/` folder per commit with its files and
`commit.json`, and `labels/<label>.json`; the folder of a commit is a component of the repository. Do not
attach a cleanup policy that deletes commits to such a repository, since labels and dependent modules
would point at commits that are gone. Calls with JSON messages, the `FileDescriptorSet` service, plugins,
policies and the management of users and organizations are not served.
