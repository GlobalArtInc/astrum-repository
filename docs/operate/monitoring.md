# Monitoring and logs

## Health endpoints

| Endpoint | Authentication | Returns 200 when |
|---|---|---|
| `GET /service/rest/v1/status` | none | The database answers |
| `GET /service/rest/v1/status/writable` | none | The database accepts writes (it is not a standby), read-only mode is off and the data directory is writable; use it for load balancer and container health checks |
| `GET /service/rest/v1/status/check` | privilege `astrum:metrics:read` (included in `as-admin`) | Always 200; the body is a JSON object of named health checks with `healthy` and `message` |

Both status endpoints answer 503 with a plain-text reason otherwise. The container image runs
`astrum check` as its health check, which asks `GET /service/rest/v1/status` on the loopback address.

The named checks are the same ones **Settings → Support → Metrics and Health** shows, and the red dot
on the bell in the header turns on when one fails:

| Check | Fails when |
|---|---|
| Available CPUs | Never fails; it reports the cores the host gives the server |
| Available Disk Space | The data directory has less than 1 GB free |
| Blob Stores Quota | A blob store passed its soft quota |
| Blob Stores Ready | A blob store cannot be opened |
| Database | The database does not answer |
| Database Writes | The database is a standby, or read-only mode is on |
| Default Admin Credentials | The file `admin.password` still exists, so the first password was not changed |
| Default Role Realm | The Default Role Realm is active but no role is chosen, or the chosen role does not exist |
| Log Files | The log directory cannot be written, so logs go to standard error only |
| Scheduler | The task scheduler stopped running |

## What to watch

The server has no metrics endpoint for Prometheus. Watch it from the outside and with the checks above:

- `/service/rest/v1/status/writable` not returning 200;
- `/service/rest/v1/status/check` reporting `"healthy": false` for any check; a scraper such as the
  Prometheus `json_exporter`, or a script run by your monitoring, turns that into an alert;
- free space on the volumes of the data directory and the blob stores, and the quota of every blob
  store;
- failed scheduled tasks: set the **Notification Email** of a task and **Send Notification** to failures
  only under **Settings → System → Tasks**, after setting up the mail server under **Settings → System →
  Email Server**;
- PostgreSQL itself: connections, replication lag, disk, and the slow-query log.

## Logs

| Where | Content |
|---|---|
| standard error | The application log, as text or, with `ASTRUM_LOG_FORMAT=json`, one JSON object per line; `podman logs`, `docker logs` and `journalctl` show it |
| `<data>/log/astrum-<node>.log` | The same log in a file, 32 MiB per file and four rotated files |
| **Audit** in the web interface (`/audit`) | Security and configuration changes and downloads, kept in the database |
| **Settings → System → Tasks** | The result of every task run |

Read the file log under **Settings → Support → Logs**. Change the level of any module at runtime under
**Settings → Support → Logging**; the change stays until you reset it. Errors that reach a client carry a
reference that can be found in the log.

## Support ZIP

**Settings → Support → Support ZIP** collects system information, the configuration (without passwords),
an overview of the security settings, the health checks, the logs, the task log and the audit log into one
archive for troubleshooting. The archive is created on the server that served the request and waits in
`<data>/downloads` for a day.
