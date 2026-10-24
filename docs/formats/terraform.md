# Terraform

Give the Terraform group a registry host (Settings > Repositories > Terraform Settings), here
`astrum.example.com`; Astrum then answers Terraform's service discovery on that host. Terraform needs HTTPS.

```hcl
module "vpc" {
  source  = "astrum.example.com/acme/vpc/aws"
  version = "1.2.0"
}
```

`~/.terraformrc`, with a token from `curl -u <user> https://astrum.example.com/repository/terraform-group/token`
once the **Terraform Bearer Token Realm** is active:

```hcl
credentials "astrum.example.com" {
  token = "TerraformToken.<key>"
}

provider_installation {
  network_mirror {
    url = "https://astrum.example.com/repository/terraform-group/"
  }
}
```

The network mirror is optional; it installs every provider, also those of `registry.terraform.io`, through the
group.

A hosted repository takes module packages at `v1/modules/<namespace>/<name>/<system>/<version>/<file>`
and the release files of a provider at `v1/providers/<namespace>/<type>/<version>/<file>`, one `PUT`
per file:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file vpc-aws-1.2.0.tar.gz \
  https://astrum.example.com/repository/terraform-hosted/v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file terraform-provider-demo_2.0.0_linux_amd64.zip \
  https://astrum.example.com/repository/terraform-hosted/v1/providers/acme/demo/2.0.0/terraform-provider-demo_2.0.0_linux_amd64.zip
```

When an upload brings no `SHA256SUMS` file for a provider, the server writes it and signs it with the
signing key of the repository.
