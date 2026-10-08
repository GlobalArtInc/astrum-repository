# Upgrade

Upgrades are one-way: on start the new version migrates the database schema and data. An older
version cannot run against a migrated database, so the way back is a restore from backup.

## Procedure

1. Read the release notes of every version between the running one and the target.
2. [Back up](backup-restore.md) the database, the blob stores and the secrets key file.
3. Stop the server and wait until it has exited; give it up to three minutes.
4. Change the version:
   - container: change the image tag, for example `globalartltd/astrum-repository:3.96.4-02`;
   - distribution: unpack the new distribution into a new install directory and point the service at
     it. The data directory stays where it is.
5. Start the server and follow the log. Migration steps are logged as they run; on a large database
   the first start can take several minutes.
6. Check **Settings → Support → Status**, sign in, and download a few artifacts from hosted and proxy
   repositories.

Never run two versions against the same data directory or database at the same time.

## Rolling back

Stop the server, restore the backup taken in step 2 and start the previous version.
