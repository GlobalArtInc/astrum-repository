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

## Prometheus metrics

Create the capability **Metrics: Prometheus** under **Settings → System → Capabilities**, and the server
serves metrics in the Prometheus text format at `GET /service/metrics/prometheus`. Without the
capability, or while it is disabled, the path answers 404.

The scraper signs in like any other client: with HTTP Basic, as a user, or with an access token as the
password. It needs the privilege `as-metrics-all` (read access to metrics, included in `as-admin`).
Create a role with only that privilege and a user for the scraper:

```yaml
scrape_configs:
  - job_name: astrum
    metrics_path: /service/metrics/prometheus
    scheme: https
    basic_auth:
      username: prometheus
      password_file: /etc/prometheus/astrum-password
    static_configs:
      - targets: ["repo.example.com"]
```

Granting the privilege to the anonymous role lets a scraper in without credentials; do that only when
the address cannot be reached from outside. Each node answers for itself: the request counters and the
database pool are those of the node that served the scrape, the rest is the same on every node.

| Metric | Type | Labels | What it is |
|---|---|---|---|
| `astrum_build_info` | gauge | `version`, `revision` | Always 1 |
| `astrum_start_time_seconds` | gauge | | When the process started |
| `astrum_http_requests_total` | counter | `area`, `method`, `status` | Requests answered; `area` is `repository`, `rest`, `internal` (the web interface), `service`, `mcp`, `docker` or `web`, `status` the class such as `2xx` |
| `astrum_http_request_duration_seconds` | histogram | `area` | Time until the response headers were ready; a download keeps streaming after that |
| `astrum_repository_requests_total` | counter | `repository`, `method`, `status` | Requests to each repository |
| `astrum_repository_online` | gauge | `repository`, `format`, `type` | 1 online, 0 offline |
| `astrum_repository_bytes` | gauge | `repository` | Logical bytes of the assets of each repository, from its usage counter |
| `astrum_repository_quota_bytes` | gauge | `repository` | The quota limit in bytes; only repositories that have a [quota](../repositories/quotas.md) |
| `astrum_blobstore_up` | gauge | `blobstore`, `type` | 1 when the store opens and answers |
| `astrum_blobstore_blobs`, `astrum_blobstore_size_bytes` | gauge | `blobstore`, `type` | Blobs and bytes in the store |
| `astrum_blobstore_available_bytes` | gauge | `blobstore`, `type` | Free space, for stores that can tell it (file, remote, group) |
| `astrum_blobstore_quota_violation` | gauge | `blobstore` | 1 when the store passed its soft quota; only stores with a quota |
| `astrum_blobstore_space_alert` | gauge | `blobstore`, `problem` | 1 for every problem the space alert below reported and that still lasts |
| `astrum_health_check_healthy` | gauge | `check` | 1 when the named health check passes |
| `astrum_data_directory_size_bytes`, `astrum_data_directory_free_bytes` | gauge | | The volume of the data directory |
| `astrum_database_connections` | gauge | `state` | Connections to PostgreSQL, `idle` or `used` |
| `astrum_notification_deliveries_pending` | gauge | | Notifications waiting to be sent or sent again |
| `astrum_scheduler_last_tick_timestamp_seconds` | gauge | | When the scheduler of the node last looked for due tasks |
| `astrum_task_running` | gauge | `task`, `type` | 1 while a task runs or waits to run |
| `astrum_task_last_run_success`, `astrum_task_last_run_timestamp_seconds` | gauge | `task`, `type` | Outcome and end of the last finished run |
| `astrum_metrics_collection_complete` | gauge | | 0 when the measurements did not finish within 8 seconds and were left out |

Turn off **Report every repository** in the capability on a server with very many repositories; the two
`astrum_repository_*` metrics then go away. Requests to names that are not repositories are counted
in `astrum_http_requests_total` only.

## Blob store space alerts

The capability **Blob Stores: Space Alert** checks every blob store on a schedule and sends a message to
the notification channels it names (set them up under **Settings → System → Notifications**) when a
store

- has less free space than **Minimum free space (GB)**, for the stores that can tell it: file, remote
  and group stores (a group counts the free space of all its members);
- passed its [soft quota](../configure/blob-stores.md#soft-quotas), unless that is turned off;
- cannot be opened or reached, such as a remote store whose agent does not answer, unless that is
  turned off;
- will be full soon: its growth over the last 30 days fills it in fewer days than **Forecast alert
  (days)**, see [the forecast](#fill-forecast).

The same checks watch the repositories that have a quota (`storage.quota` in the repository settings).
A repository is reported when it uses at least its warning percentage of the limit and when it reaches
the limit. Both are one problem that changes kind: a repository that crosses the warning gets one
message, the same repository reaching the limit gets one more, and one message tells when it is back
below the warning. Repositories without a quota are not checked, and neither is a group repository.

A problem is reported once when it starts, again after **Repeat after (hours)** while it lasts (0 never
repeats it), and, with **Send a message when a problem is over**, once more when it ends. The checks run
every **Check interval (minutes)** on one node at a time. The **Status** tab of the capability shows the
last check and the problems that last; the metric `astrum_blobstore_space_alert` shows them too.

Messages follow the language of the channel. A webhook receives the events `blobstore.alert` and
`blobstore.recovered` with the message in `text`; `repository` and `component` are `null`.

### Fill forecast

The task that records the daily usage (see [tasks](tasks.md)) keeps one row per blob store and day. The
forecast fits a straight line through the last 30 of those days and needs at least 7. When the line
rises, the days left are the room left divided by the bytes it grows per day. The room is:

- the free space the store reports, for file and remote stores;
- the limit of a soft quota of the type space used, minus what the store uses; a store that has both
  counts the smaller of the two;
- nothing for an S3 store without a soft quota, which has no forecast.

A group store has no forecast of its own; each of its members is forecast on its own. The setting
**Forecast alert (days)** (`forecastDays`, from 0 to 365, 14 by default) is the number of days below
which the space alert sends the message "will be full in N days"; 0 turns this alert off. The days
change with every check, so the message is sent once when the problem starts, like the others.

`GET /service/rest/v1/blobstores/{name}/forecast` answers the same numbers for one store, to callers who
may read the blob stores:

```json
{"daysUntilFull": 11.2, "bytesPerDay": 1000.0, "basis": "capacity", "sampleDays": 9}
```

`daysUntilFull` is `null` when the store does not grow, when there is no room to measure against or when
fewer than 7 days were recorded; `bytesPerDay` is `null` below 7 days; `basis` is `capacity` (free
space), `softQuota` or `null`; `sampleDays` is the number of days used. The blob store list and the page
of a store show "Full in N days" when there is a value.

## Usage history

The daily task **Record repository usage** (at 00:30 UTC, see [scheduled tasks](tasks.md)) keeps one row per
day for every repository, with its components, assets, bytes and downloads, and one per blob store, with its
blob count, bytes and free space. It records the day before. A run for a day that already has rows replaces
them. Rows older than the setting `usage.history.daysKept` (default `730`) are deleted. Downloads are taken
from the download records of the assets, so downloads of assets deleted on the same day are lost.

Two routes read the repository rows:

- `GET /service/rest/v1/repositories/{name}/usage/history?days=` returns the days that have a row, oldest
  first, as objects with `day`, `components`, `assets`, `bytes` and `downloads`. `days` is from 1 to 730,
  default 90; any other value is answered with 400 and the field `days`. Rights are those of the repository
  summary: anonymous callers get 401, callers without access to the repository get 403, and an unknown
  repository gets 404.
- `GET /service/rest/internal/ui/usage-history?days=` serves the web interface. It answers an object that
  maps each repository the caller may browse to the `bytes` of its days, oldest first, for the last `days`
  days that have a row. `days` is from 1 to 730, default 30; other values get 400 as above. A repository the
  caller may not browse has no key, and anonymous callers get 401.

## What to watch

With the Prometheus metrics above, alert on `astrum_blobstore_up == 0`,
`astrum_health_check_healthy == 0`, `astrum_task_last_run_success == 0` and on the rate of
`astrum_http_requests_total{status="5xx"}`. Without them, watch the server from the outside and with the
checks above:

- `/service/rest/v1/status/writable` not returning 200;
- `/service/rest/v1/status/check` reporting `"healthy": false` for any check; a scraper such as the
  Prometheus `json_exporter`, or a script run by your monitoring, turns that into an alert;
- free space on the volumes of the data directory and the blob stores, and the quota of every blob
  store, or the space alert above;
- failed tasks: alert on `astrum_task_last_run_success == 0`, and give important tasks a
  **Notification Email**, which the server sends through the email server when a run fails; the email is
  not retried when sending fails, so it does not replace the alert, see
  [notification email](tasks.md#notification-email);
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
