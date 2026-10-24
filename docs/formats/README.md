# Formats

Every format has a page with the settings of its clients, how to publish to a hosted repository and
what proxies and groups of it do. What is common to all formats, the address of a repository and how
clients sign in, is in [connect build tools to repositories](../use/clients.md); the settings of
repositories themselves are in [repositories](../repositories/overview.md).

| Format | Clients | Hosted | Proxy | Group |
| --- | --- | :---: | :---: | :---: |
| [Alpine](alpine.md) | `apk` | ✓ | ✓ | ✓ |
| [Ansible Galaxy](ansible.md) | `ansible-galaxy` | ✓ | ✓ | ✓ |
| [APT](apt.md) | `apt` | ✓ | ✓ | |
| [Bazel](bazel.md) | Bazel modules (Bzlmod) | ✓ | ✓ | ✓ |
| [Cargo](cargo.md) | `cargo` | ✓ | ✓ | ✓ |
| [Chef Supermarket](chef.md) | Berkshelf, Policyfiles, `knife` | ✓ | ✓ | ✓ |
| [Chromium extensions](chrome.md) | Chrome, Edge and other Chromium browsers | ✓ | ✓ | ✓ |
| [CocoaPods](cocoapods.md) | `pod` | | ✓ | |
| [Composer](composer.md) | `composer` | ✓ | ✓ | ✓ |
| [Conan](conan.md) | `conan` | ✓ | ✓ | ✓ |
| [Conda](conda.md) | `conda`, `mamba` | ✓ | ✓ | ✓ |
| [Docker and OCI](docker.md) | `docker`, `podman`, Helm and ORAS over OCI | ✓ | ✓ | ✓ |
| [Firefox add-ons](firefox.md) | Firefox | ✓ | ✓ | ✓ |
| [Git LFS](gitlfs.md) | `git lfs` | ✓ | | |
| [Go](go.md) | `go` | ✓ | ✓ | ✓ |
| [Helm](helm.md) | `helm` | ✓ | ✓ | ✓ |
| [Hex](hex.md) | Mix, rebar3 | ✓ | ✓ | ✓ |
| [Hugging Face](huggingface.md) | `huggingface_hub`, `transformers`, `hf` | | ✓ | |
| [JetBrains plugins](jetbrains.md) | IntelliJ Platform IDEs | ✓ | ✓ | ✓ |
| [Maven](maven.md) | Maven, Gradle | ✓ | ✓ | ✓ |
| [Nix](nix.md) | `nix` | ✓ | ✓ | ✓ |
| [npm](npm.md) | npm, pnpm, Yarn | ✓ | ✓ | ✓ |
| [NuGet](nuget.md) | `dotnet`, `nuget` | ✓ | ✓ | ✓ |
| [Nx remote cache](nx.md) | Nx | ✓ | | |
| [Open VSX](openvsx.md) | VSCodium, code-server, `ovsx` | ✓ | ✓ | ✓ |
| [P2](p2.md) | Eclipse | | ✓ | |
| [pacman](pacman.md) | `pacman` | ✓ | ✓ | ✓ |
| [Protobuf](protobuf.md) | `buf` | ✓ | | |
| [pub](pub.md) | `dart`, `flutter` | ✓ | ✓ | ✓ |
| [Puppet Forge](puppet.md) | `puppet module`, r10k, Bolt | ✓ | ✓ | ✓ |
| [PyPI](pypi.md) | `pip`, `uv`, `twine` | ✓ | ✓ | ✓ |
| [R](r.md) | `install.packages` | ✓ | ✓ | ✓ |
| [Raw files](raw.md) | `curl` and any HTTP client | ✓ | ✓ | ✓ |
| [RubyGems](rubygems.md) | `gem`, Bundler | ✓ | ✓ | ✓ |
| [Swift](swift.md) | Swift Package Manager | ✓ | ✓ | ✓ |
| [Terraform](terraform.md) | `terraform` | ✓ | ✓ | ✓ |
| [Turborepo remote cache](turbo.md) | `turbo` | ✓ | | |
| [Vagrant](vagrant.md) | `vagrant` | ✓ | ✓ | ✓ |
| [Yum](yum.md) | `dnf`, `yum` | ✓ | ✓ | ✓ |
