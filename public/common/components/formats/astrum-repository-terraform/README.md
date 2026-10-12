# Terraform repositories

Astrum serves Terraform modules and providers with the
[module registry protocol](https://developer.hashicorp.com/terraform/internals/module-registry-protocol),
the [provider registry protocol](https://developer.hashicorp.com/terraform/internals/provider-registry-protocol)
and the [provider network mirror protocol](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol).
Three recipes exist:

- `terraform-hosted`: modules and providers you publish. Module packages are `.tar.gz` or `.zip` archives;
  provider versions are the release files your build writes (a zip per platform, `SHA256SUMS`, its
  signature, optionally the release manifest).
- `terraform-proxy`: caches a remote registry, for the public registry use `https://registry.terraform.io` as
  the remote URL. Version lists and package descriptions are metadata (refreshed after the metadata max age),
  packages and release files are content.
- `terraform-group`: merges the version lists of its members (the first member wins for a version) and serves
  download endpoints and packages from the first member that has them.

## Paths

Every repository answers under `https://astrum.example.com/repository/<name>/`:

| Path | Protocol |
|---|---|
| `.well-known/terraform.json` | service discovery, `modules.v1` and `providers.v1` relative to it |
| `v1/modules/<namespace>/<name>/<system>/versions` | module versions |
| `v1/modules/<namespace>/<name>/<system>/<version>/download` | `204` with `X-Terraform-Get` |
| `v1/modules/<namespace>/<name>/<system>/<version>/<name>-<system>-<version>.tar.gz` | module package |
| `v1/providers/<namespace>/<type>/versions` | provider versions with protocols and platforms |
| `v1/providers/<namespace>/<type>/<version>/download/<os>/<arch>` | package description with keys |
| `v1/providers/<namespace>/<type>/<version>/<file>` | provider release files |
| `<hostname>/<namespace>/<type>/index.json` | network mirror: versions |
| `<hostname>/<namespace>/<type>/<version>.json` | network mirror: packages with `zh:` hashes |
| `token` | bearer token of the signed-in user, see below |

## How Terraform finds a repository

Terraform reads service discovery only at the root of the host named in an address
(`https://<host>/.well-known/terraform.json`), so a host stands for one registry. Two ways lead to a
repository:

1. **Registry host.** Give one Terraform repository, usually a group, a registry host (Settings >
   Repositories > Terraform Settings, or `"terraform":{"registryHost":"astrum.example.com"}` in the REST API).
   Astrum then answers `/.well-known/terraform.json` on that host with the endpoints of the repository. A host
   without a port matches the host on any port, a host with a port only that port; one host belongs to one
   repository. Several repositories need several host names pointing at the server (DNS aliases), each the
   registry host of one repository.
2. **A host of its own behind a reverse proxy.** Map the root of a host to the repository path, for example
   `https://terraform.example.com/` to `http://astrum:8081/repository/terraform-group/`. Every URL a repository
   hands out (discovery, `X-Terraform-Get`, package descriptions, mirror documents) is relative to the document
   that carries it, so it stays valid behind such a mapping.

Terraform requires HTTPS on the registry host.

Providers can also come through the network mirror protocol, which takes a URL with a path and needs no host of
its own: the providers keep their original addresses, such as `registry.terraform.io/hashicorp/aws`.

## Configure Terraform

With `astrum.example.com` as the registry host of `terraform-group`:

```hcl
terraform {
  required_providers {
    demo = {
      source  = "astrum.example.com/acme/demo"
      version = "~> 2.0"
    }
  }
}

module "vpc" {
  source  = "astrum.example.com/acme/vpc/aws"
  version = "1.2.0"
}

# a module of the public registry through a proxy member of the group
module "eks" {
  source  = "astrum.example.com/terraform-aws-modules/eks/aws"
  version = "20.8.4"
}
```

`~/.terraformrc` (or the file named by `TF_CLI_CONFIG_FILE`):

```hcl
credentials "astrum.example.com" {
  token = "TerraformToken.0123abcd-..."
}

# optional: install every provider, including those of registry.terraform.io, through Astrum
provider_installation {
  network_mirror {
    url = "https://astrum.example.com/repository/terraform-group/"
  }
}
```

The token can also be given as `TF_TOKEN_astrum_example_com`. When anonymous users may read the repository, no
credentials are needed.

## Tokens and download URLs

Terraform sends its credentials only to the registry API and the mirror documents, as
`Authorization: Bearer <token>`:

- An administrator activates the **Terraform Bearer Token Realm** once (Security > Realms, or
  `PUT /service/rest/v1/security/realms/active` with `TerraformToken` in the list).
- A user gets a token with `curl -u <user> https://astrum.example.com/repository/terraform-group/token`
  (`{"token":"TerraformToken.<key>"}`); it works for every Terraform repository the user may read.
  `DELETE` on the same URL revokes it.

Terraform fetches module packages, provider packages, checksums and signatures without these credentials
(only a `.netrc` entry would be used). The locations Astrum hands out therefore carry a signature over the path
they name and an expiry, made with a secret of the repository: valid for 15 minutes and for that one path.
Groups sign the locations of their members again for themselves.

`"terraform":{"requireAuthentication":true}` refuses anonymous requests even when anonymous access is enabled.

## Publish

Modules, through the components API or a `PUT` to the package path (the file name may be any name ending in
`.tar.gz`, `.tgz` or `.zip`; the package is stored as `<name>-<system>-<version>.<extension>`):

    curl -u <user> -F terraform.namespace=acme -F terraform.name=vpc -F terraform.system=aws \
      -F terraform.version=1.2.0 -F terraform.asset=@vpc.tar.gz \
      "https://astrum.example.com/service/rest/v1/components?repository=terraform-hosted"
    curl -u <user> --upload-file vpc.tar.gz \
      https://astrum.example.com/repository/terraform-hosted/v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz

When all files of a package sit in one top directory, Terraform is pointed at that directory.

Providers, the release files of one version in one request or one `PUT` per file:

    curl -u <user> -F terraform.namespace=acme -F terraform.type=demo -F terraform.version=2.0.0 \
      -F terraform.asset1=@terraform-provider-demo_2.0.0_linux_amd64.zip \
      -F terraform.asset2=@terraform-provider-demo_2.0.0_darwin_arm64.zip \
      -F terraform.asset3=@terraform-provider-demo_2.0.0_SHA256SUMS \
      -F terraform.asset4=@terraform-provider-demo_2.0.0_SHA256SUMS.sig \
      -F terraform.asset5=@terraform-provider-demo_2.0.0_manifest.json \
      -F terraform.asset6=@release-key.asc \
      "https://astrum.example.com/service/rest/v1/components?repository=terraform-hosted"
    for f in terraform-provider-demo_2.0.0_*; do
      curl -u <user> --upload-file "$f" \
        https://astrum.example.com/repository/terraform-hosted/v1/providers/acme/demo/2.0.0/"$f"
    done
    curl -u <user> --upload-file release-key.asc \
      https://astrum.example.com/repository/terraform-hosted/v1/providers/acme/demo/2.0.0/signing-key.asc

Files are recognized by their names: `terraform-provider-<type>_<version>_<os>_<arch>.zip`, `..._SHA256SUMS`,
`..._SHA256SUMS.sig` (also `..._SHA256SUMS.<key id>.sig`, stored as `..._SHA256SUMS.sig`; an ASCII-armored
signature is stored in binary form), `..._manifest.json` (its `metadata.protocol_versions` become the plugin
protocols of the version; without one, `terraform.protocols` or `?protocols=6.0` sets them, else `5.0`) and a
public key (`.asc`, `.gpg`, `.pub`, `.key`, stored as `signing-key.asc`).

Uploaded files are checked against each other: packages against the uploaded checksums (a package the
checksums do not list is refused), checksums against stored packages, the signature and the key against the
checksums once both are there. The files of one request are checked together before any of them is stored, so a
release that does not fit together is refused as a whole.

Without uploaded checksums the repository writes `SHA256SUMS` from the packages and rewrites it when packages
come and go. Without an uploaded signature it signs the checksums with the key of the repository
(`"terraformSigning":{"keypair":"<armored secret key>","passphrase":"..."}`, hosted only; the passphrase is
never returned). Terraform installs a provider through the registry protocol only with a signature and a key
that verifies it; the network mirror protocol needs neither.

## Proxy

The proxy reads the service discovery of its remote (falling back to `v1/modules/` and `v1/providers/` under
the remote URL) and sends the bearer token configured for the remote to the registry host only.

- Module packages are cached when the upstream location is an archive over HTTP (`.tar.gz`, `.tgz`, `.zip`,
  `.tar.bz2` or an `archive=` argument) or a GitHub repository at a ref (`git::https://github.com/<owner>/<repo>?ref=<ref>`,
  fetched from `codeload.github.com`). A subdirectory glob such as `//*` is resolved against the cached package,
  since Terraform does not expand it for registry modules. Other locations (other Git hosts, SSH, cloud
  buckets) are handed to Terraform unchanged.
- Provider package descriptions point at the release files under the proxy; a package is stored only when its
  SHA-256 matches the `shasum` the registry published.
- The network mirror answers for the providers of the remote host, or of the upstream registry host
  (`"terraform":{"upstreamHost":"registry.terraform.io"}`) when the remote is itself a mirror of that registry.
  Its `zh:` hashes come from the upstream `SHA256SUMS` after its signature verified with a key the upstream
  registry lists.

## Create repositories

    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/terraform/hosted \
      -d '{"name":"terraform-hosted","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"allow_once"},"terraformSigning":{"keypair":"-----BEGIN PGP PRIVATE KEY BLOCK-----..."}}'
    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/terraform/proxy \
      -d '{"name":"terraform-proxy","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true},"proxy":{"remoteUrl":"https://registry.terraform.io","contentMaxAge":1440,"metadataMaxAge":1440},"negativeCache":{"enabled":true,"timeToLive":1440},"httpClient":{"blocked":false,"autoBlock":true}}'
    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/terraform/group \
      -d '{"name":"terraform-group","online":true,"storage":{"blobStoreName":"default"},"group":{"memberNames":["terraform-hosted","terraform-proxy"]},"terraform":{"registryHost":"astrum.example.com"}}'

## Search and cleanup

A component is a module or provider version: its group is the namespace, its name `name/system` for a module
and the type for a provider, so group and name read as the address. Search takes `terraform.namespace`, `terraform.name` (module name or
provider type), `terraform.provider` (module system or provider type) and `terraform.kind` (`module` or
`provider`). Cleanup policies support age, last download, a path pattern and keeping the newest versions of
each module or provider.
