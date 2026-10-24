# R

Repositories have the layout of CRAN: source packages in `src/contrib/` (older versions in
`src/contrib/Archive/<package>/`), Windows binaries in `bin/windows/contrib/<R version>/` and macOS
binaries in `bin/macosx/[<platform>/]contrib/<R version>/`. A group of a hosted repository and a proxy of
CRAN in `~/.Rprofile`:

```r
options(repos = c(CRAN = "https://astrum.example.com/repository/r-group"))
install.packages("data.table")
```

Without anonymous access, put a user and an [access token](../configure/authentication.md#access-tokens)
into the address: `https://ci-reader:<token>@astrum.example.com/repository/r-group`.

Upload a package to a hosted repository at its path; the file name is `<package>_<version>` with
`.tar.gz` for source, `.zip` for Windows and `.tgz` for macOS packages:

```shell
R CMD build tool
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file tool_1.4.0.tar.gz \
  https://astrum.example.com/repository/r-hosted/src/contrib/tool_1.4.0.tar.gz
```

Hosted repositories write `PACKAGES`, `PACKAGES.gz` and `PACKAGES.rds` of each directory themselves
after every upload and delete; they list the latest version of each package, and older versions stay
downloadable at their paths, for example with
`install.packages("https://astrum.example.com/repository/r-hosted/src/contrib/Archive/tool/tool_1.3.0.tar.gz", repos = NULL)`.
They do not write `src/contrib/Meta/archive.rds`, so `remotes::install_version` finds only the versions
listed in `PACKAGES`. The task "Repair - Rebuild R repository metadata (PACKAGES)" writes the files again.
