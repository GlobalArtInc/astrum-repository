# Git LFS

A Git LFS hosted repository stores the large files of git repositories; the git repository itself
stays on your git server. Point git-lfs at the repository's LFS API, `<repository URL>/info/lfs`, and
commit the setting so every clone uses it:

```shell
git config -f .lfsconfig lfs.url https://astrum.example.com/repository/lfs-hosted/info/lfs
git add .lfsconfig
git lfs track "*.psd"
git push
```

git-lfs asks for the user name and password of this server through the git credential helper; an
access token works as the password. Downloading needs read access, uploading and file locks
(`git lfs lock`, `git lfs locks`, `git lfs unlock`) need add access, and releasing another user's lock
with `--force` needs delete access. Objects are stored and searched by their SHA-256 and browsed under
`objects/<first two>/<next two>/<id>`.
