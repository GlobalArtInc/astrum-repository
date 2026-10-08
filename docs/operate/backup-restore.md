# Backup and restore

A complete backup has three parts that belong together:

| Part | Where | Holds |
|---|---|---|
| Database | PostgreSQL | Repositories, components and assets metadata, users, roles, settings, tasks, sessions, the audit log |
| Blob stores | `<data>/blobs/<name>`, the configured path or S3 bucket | The artifact files |
| Secrets key | `<data>/etc/secrets.key`, or the file named by `secrets_key_file` | The key that opens the stored passwords and keys |

Back up the configuration file (`<data>/etc/astrum.toml` or the file named by `ASTRUM_CONFIG`) as well. The
rest of the data directory (logs, temporary files, support ZIPs) is not needed for a restore.

Back up the database first and the blob stores right after it. Blobs written in between only exist in
the blob store; the task **Admin - Compact blob store** removes them again once they are older than its
grace period. The opposite order would leave database rows that point at missing files.

## PostgreSQL

Use the usual PostgreSQL tools; the server can stay online.

```shell
pg_dump --format=custom --file=astrum-$(date +%F).dump \
  --host=db.example.com --username=astrum astrum
```

Continuous archiving (WAL archiving, `pgBackRest`, managed database snapshots) works as well and
gives point-in-time recovery.

## Blob stores

File blob stores are directories of immutable files; copy them with any file-level tool after the
database backup:

```shell
rsync -a --delete /srv/astrum/data/blobs/ /backup/astrum/blobs/
```

File system or volume snapshots taken right after the database backup are faster and consistent.
For S3 blob stores enable bucket versioning or replication instead.

## Secrets key

```shell
install -m 0600 /srv/astrum/data/etc/secrets.key /backup/astrum/secrets.key
```

Store it apart from the database dump: the dump holds the sealed secrets, the key opens them, and
together they give access to every remote system the server uses. A database restored without the key
it was written with works, but its stored secrets (proxy credentials, signing keys, the LDAP bind
password, the OpenID Connect client secret, S3 secret keys, the email password) cannot be opened and
must be entered again.

## Restore

1. Stop the server (all of them, if several share the database).
2. Restore the blob stores to their original paths.
3. Restore the database into an empty database owned by the role the server uses (drop and recreate
   the old one if there is one):

   ```shell
   pg_restore --no-owner --role=astrum --dbname=astrum astrum-2026-10-18.dump
   ```

4. Restore the secrets key and the configuration file.
5. Start the server of the same version as the backup, or a newer one: it brings an older database up
   to date on start. Then check **Settings → Support → Metrics and Health** and download a few artifacts.

Artifacts uploaded after the database backup are in the blob store but unknown to the database: clients
do not see them, they only take up space until the compaction task removes them. Re-publish them from CI
if they are still needed. If the totals shown for a blob store look wrong after the restore, run the task
**Repair - Recalculate blob store storage**.

## Test the backup

Restore into a scratch instance from time to time: start the container image against a copy of the
blob stores and a restored database on another port, sign in and download a few artifacts. A backup
that was never restored is a hope, not a backup.
