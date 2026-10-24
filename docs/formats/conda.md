# Conda

A repository is a conda channel, and a hosted repository can hold several channels below it: packages
uploaded to `main/noarch/` belong to the channel `.../conda-hosted/main`. The server writes
`repodata.json` (with `.bz2` and `.zst` copies) and `current_repodata.json` for every platform
subdirectory and `channeldata.json` for every channel. A proxy points at a channel, for example
`https://conda.anaconda.org/conda-forge/`, and a group merges the `repodata.json` of its members.

```shell
conda install -c https://astrum.example.com/repository/conda-group --override-channels numpy
micromamba create -p ./env -c https://astrum.example.com/repository/conda-hosted/main --override-channels tool
```

In `~/.condarc`, to use the repository instead of the default channels:

```yaml
channels:
  - https://astrum.example.com/repository/conda-group
```

Upload a `.conda` or `.tar.bz2` package to its channel and subdirectory; the name, version, build and
subdirectory must match the package's `info/index.json`:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file tool-1.4.0-h1_0.conda \
  https://astrum.example.com/repository/conda-hosted/main/linux-64/tool-1.4.0-h1_0.conda
```

Clients sign in with the user name and password in the channel address
(`https://user:password@host/...`) or through `~/.netrc`. Sharded repodata and JLAP patches are not
served, so clients read the whole `repodata.json`.
