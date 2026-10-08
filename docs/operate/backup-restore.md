# Backup and restore

A complete backup has three parts that belong together:

| Part | Where | Holds |
|---|---|---|
| Database | PostgreSQL, or `<data>/db` for H2 | Repositories, components and assets metadata, users, roles, settings, tasks |
| Blob stores | `<data>/blobs/<name>` or the configured path / S3 bucket | The artifact files |
| Secrets key file | the file named by `astrum.secrets.file` | The key that decrypts stored passwords |

Back up the database first and the blob stores right after it. Blobs written in between only exist
in the blob store and are harmless; the opposite order would leave database rows that point at
missing files.

## PostgreSQL

Use the usual PostgreSQL tools; the server can stay online.

```shell
pg_dump --format=custom --file=astrum-$(date +%F).dump \
  --host=db.example.com --username=astrum astrum
```

Continuous archiving (WAL archiving, `pgBackRest`, managed database snapshots) works as well and
gives point-in-time recovery.

## Embedded H2

The H2 file cannot be copied while the server runs. Schedule the task **Admin - Backup H2 Database**
under **Settings → System → Tasks** with a **Location** outside the data directory, for example
`/backup/astrum/db`. Each run writes a zip archive there.

## Blob stores

File blob stores are directories of immutable files plus small metadata files; copy them with any
file-level tool after the database backup:

```shell
rsync -a --delete /srv/astrum/data/blobs/ /backup/astrum/blobs/
```

File system or volume snapshots taken right after the database backup are faster and consistent.
For S3 blob stores enable bucket versioning or replication instead.

## Restore

1. Stop the server.
2. Restore the blob stores to their original paths.
3. Restore the database:
   - PostgreSQL: `pg_restore --clean --if-exists --dbname=astrum astrum-2026-10-09.dump` into the
     database the server is configured for;
   - H2: delete `<data>/db/astrum.mv.db`, put the backup zip into `<data>/restore-from-backup/` and
     the server restores it on the next start.
4. Restore the secrets key file if one is configured.
5. Start the server and check **Settings → Support → Status** and a few downloads.

Artifacts uploaded after the database backup are in the blob store but unknown to the database: clients
do not see them, they only take up space. Re-publish them from CI if they are still needed.

## Test the backup

Restore into a scratch instance from time to time: start the container image against a copy of the
blob stores and a restored database on another port, sign in and download a few artifacts. A backup
that was never restored is a hope, not a backup.
