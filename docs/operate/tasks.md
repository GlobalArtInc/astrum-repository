# Tasks

Tasks do maintenance in the background: running cleanup policies, freeing blob store space, rebuilding
the metadata of some formats. A task has a type, a name, the settings of its type and a schedule, and
every run keeps a log. Manage tasks under **Settings → System → Tasks**, or with the REST API under
`/service/rest/v1/tasks`.

Tasks are stored in the database. With several servers on one database, each run happens on one of them;
see [nodes and several servers](nodes.md#tasks-on-several-servers).

## Built-in tasks

On its first start the server creates four tasks. It does not create them again, so a built-in task you
delete stays deleted until you create one of its type yourself.

| Task | Type | Schedule |
|---|---|---|
| Cleanup service | Admin - Cleanup repositories using their associated policies | Daily at 01:00 UTC, `0 0 1 * * ?` |
| Delete unused Docker manifests and layers | Docker - Delete unused manifests and images, all repositories, deploy offset 24 hours | Daily at 01:30 UTC, `0 30 1 * * ?` |
| Compact blob stores | Admin - Compact blob store, all blob stores, 1 day | Daily at 02:00 UTC, `0 0 2 * * ?` |
| Delete expired sessions and uploads | Admin - Delete expired sessions and uploads | Hourly, from five minutes after the first start |

## Task types

The ID is the `type` in the REST API. A type marked "one only" can exist once on a server. A repository
or blob store setting that offers **(All Repositories)** or **(All Blob Stores)** works on every one that
fits.

| Type | ID | Settings | What it does |
|---|---|---|---|
| Admin - Cleanup repositories using their associated policies | `repository.cleanup` | none; one only | Runs the [cleanup policies](../repositories/cleanup-policies.md) assigned to each hosted and proxy repository and deletes the components they match. A policy without criteria deletes nothing; a policy that is missing or made for another format is skipped with a warning in the log |
| Admin - Compact blob store | `blobstore.compact` | **Blob Store** (a group compacts its members), **Minimum Blob Age (days)**, default `1` | Deletes the blobs no asset refers to and no unfinished Docker upload uses, such as the leftovers of interrupted uploads, once they are older than the minimum age; blobs younger than ten minutes are always kept |
| Repair - Recalculate blob store storage | `blobstore.metrics.reconcile` | **Blob Store** (not a group; all stores includes the members of groups) | Counts the blobs and bytes of a store again and corrects the totals shown for it; the log lists the old and the new totals |
| Repository - Delete unused components | `repository.purge-unused` | **Repository** (proxy repositories), **Last used in days**, default `30` | Deletes the cached assets of proxy repositories that nobody downloaded for that many days (counted from when they were cached, if never downloaded), and components left without assets; every deletion is audited |
| Admin - Delete expired sessions and uploads | `security.purge-expired` | none; one only | Deletes web sessions idle for longer than the [session timeout](../configure/runtime.md#sessions) (none while it is `0`), expired tokens that formats such as Docker and npm hand out to clients, and Docker uploads left unfinished for 24 hours |
| Docker - Delete unused manifests and images | `repository.docker.gc` | **Repository** (hosted Docker repositories), **Deploy offset (hours)**, default `24` | Deletes untagged manifests and the layers no manifest uses any more, if they were pushed longer ago than the deploy offset |
| Nix - Delete NAR files no narinfo uses | `repository.nix.gc` | **Repository** (hosted Nix repositories), **Deploy offset (hours)**, default `24` | Deletes NAR files that no narinfo file refers to, if they were uploaded longer ago than the offset, so that a `nix copy` in progress can still add their narinfo files |

Some formats have a task that writes their metadata anew, for example after a change of the remote URL
or when the metadata of a hosted repository is damaged:

| Type | ID | Repositories | Options |
|---|---|---|---|
| Alpine - Rebuild Alpine indexes | `repository.alpine.rebuild.metadata` | Hosted, proxy, group, or all | **Reset proxy and group indexes** |
| Apt - Rebuild Apt metadata | `repository.apt.rebuild.metadata` | Hosted or proxy, one at a time | **Full rebuild (hosted only)**, **Reset proxy metadata** |
| Conda - Rebuild Conda metadata | `repository.conda.rebuild.metadata` | Hosted or proxy, one at a time | **Reset proxy metadata** |
| Hex - Rebuild Hex registries | `repository.hex.rebuild.metadata` | Hosted, proxy, group, or all | **Reset proxy and group registries** |
| pacman - Rebuild pacman databases | `repository.pacman.rebuild.metadata` | Hosted, proxy, group, or all | **Reset proxy and group databases** |
| Repair - Rebuild R repository metadata (PACKAGES) | `repository.r.rebuild.metadata` | Hosted or proxy, one at a time | **Reset proxy metadata** |
| Repair - Rebuild Yum repository metadata (repodata) | `repository.yum.rebuild.metadata` | Hosted, one at a time | **Soft repair** |

A reset option deletes the cached metadata of proxy repositories, and the merged metadata of groups, so
that it is fetched or merged again; hosted metadata is always written anew. **Full rebuild (hosted only)**
reads every package of a hosted APT repository again. **Soft repair** only adds the RPM changes since the
last generation of the Yum metadata. Some pages of the [formats](../formats/README.md) say when to run
these tasks.

## Creating and changing a task

1. Click **Create Task**, pick a type under **Select Type** and click **Continue**.
2. Under **Configure**, give the task a **Name** no other task uses, fill in the settings of its type and
   decide whether it is **Enabled**.
3. Under **Schedule**, choose when it runs, then click **Create**.

Open a task from the list to change its settings or schedule. Its type cannot change, and the web
interface keeps its name. A disabled task does not run on its schedule and cannot be started by hand.

## Schedules

| Schedule | The task runs |
|---|---|
| **Manual** | Only when someone starts it |
| **Once** | At **Date** and **Time**, which must lie in the future |
| **Hourly** | Every hour from the start date and time |
| **Daily** | Every day at **Time**, from the start date |
| **Weekly** | At **Time** on the **Days to Run** |
| **Monthly** | At **Time** on the **Days of Month to Run**; **Last** is the last day of each month, and a month without the chosen day, such as the 31st, is skipped |
| **Advanced (Cron)** | When the **Cron Expression** says |

The web interface takes dates and times in the time zone of the browser and saves them with its UTC
offset at that moment. The offset is fixed, so a schedule does not move with daylight saving time. Cron
expressions entered in the web interface are read in UTC. Over the REST API, `timeZoneOffset` sets the
offset of any schedule, cron included.

### Cron expressions

A cron expression has six or seven fields separated by spaces:

| Field | Values |
|---|---|
| Seconds | `0`-`59` |
| Minutes | `0`-`59` |
| Hours | `0`-`23` |
| Day of month | `1`-`31`, or `?`, `L`, `L-<n>`, `<n>W`, `LW` |
| Month | `1`-`12` or `JAN`-`DEC` |
| Day of week | `1`-`7` with `1` for Sunday, or `SUN`-`SAT`, or `?`, `L`, `<d>L`, `<d>#<n>` |
| Year, optional | `1970`-`2099` |

- `*` means every value, `,` separates values, `-` gives a range (`MON-FRI`), and `/` a step: `0/15` in
  the minutes is every 15 minutes from minute 0, `*/4` in the hours every fourth hour.
- One of day of month and day of week must be `?`, the other names the days.
- In the day of month, `L` is the last day, `L-3` the third day before it, `15W` the weekday nearest the
  15th and `LW` the last weekday of the month.
- In the day of week, `L` alone is Saturday, `6L` the last Friday of the month and `2#1` the first Monday.

| Expression | Runs |
|---|---|
| `0 0 3 * * ?` | 03:00 every day |
| `0 0 */4 * * ?` | Every fourth hour |
| `0 0 9 ? * MON-FRI` | 09:00 Monday to Friday |
| `0 30 8 1 * ?` | 08:30 on the first day of every month |
| `0 0 2 ? * SUN` | 02:00 every Sunday |

## Running and stopping

**Run Now** on the task, or the run button in the list, queues a run, which starts within seconds. A task
that is running or queued cannot be started again until that run ends.

**Stop** cancels a queued run at once. A running task stops at its next checkpoint, usually within a few
seconds, and the run ends as **Canceled**. Deleting a task stops its run and deletes its history.

- Each server runs at most four tasks at a time; further runs wait as **Queued**.
- When a task's time comes while it still runs, that time is skipped.
- When the time of a task passed while no server was running, the task runs once as soon as a server
  starts, however many times were missed.
- A server that shuts down asks its running tasks to stop and waits up to ten seconds for them. A run whose
  server stopped or crashed is marked failed with the message
  "the server running the task stopped before it finished".

## Results and logs

The list shows the **Status** of every task (**Disabled**, **Waiting**, **Queued** or **Running**), its
**Schedule**, **Next Run**, **Last Run** and **Last Result**: **OK**, **Failed** or **Canceled**.

On the **History** tab of a task, **Execution History** lists the last 20 runs with the time they
**Started**, who started them (**Started by**: the schedule or a user), their **Duration**, **Result** and
summary, and **Show log** shows a run's log, which keeps its first 1000 lines. A run that is still going
shows its log as it is written.

The same log lines go to the application log, with the name of the task and the number of the run; read
it under **Settings → Support → Logs**. The end of every run is also recorded in the audit log, and the
[metrics](monitoring.md#prometheus-metrics) `astrum_task_running`, `astrum_task_last_run_success` and
`astrum_task_last_run_timestamp_seconds` report the state of each task. The health check **Scheduler**
turns red when the scheduler of a server stops looking for due tasks.

The **Tasks & Capabilities** tab of a repository lists the tasks whose **Repository** is that repository.

### Notification email

A task has **Email** (**Notification Email** once it exists) and **Send Notification**, with **Only when a
run fails** (`FAILURE` in the REST API, the default) or **After every run** (`SUCCESS_FAILURE`). When a run
of a task with an address ends, the server sends a plain-text email to that address through the mail server
under **Settings → System → Email Server**:

| Send Notification | Failed run | Successful run | Stopped run |
|---|---|---|---|
| **Only when a run fails** | Email | None | None |
| **After every run** | Email | Email | Email |

- A run started with **Run Now**, over the REST API or through MCP notifies like a scheduled one.
- The server that ran the task sends the email, so with several servers on one database a run is reported
  once. A run whose server stopped or crashed counts as failed and is reported by the server that marks it
  failed.
- A queued run that is stopped before it starts sends nothing.

The subject is `Task failed: <name>`, `Task finished: <name>` or `Task canceled: <name>`, after the subject
prefix of the email server. The message names the task and its type, the result, when the run started and
finished, who started it (the schedule or a user), the server that ran it, and the error of a failed run or
the summary of any other. With a [base URL](../configure/runtime.md#base-url) set, it links to the task in
the web interface; without one there is no request to build the link from, so the email has none.

Sending never fails or delays the task: the email goes out after the run is recorded. When the email
server is disabled or not set up, or the mail server cannot be reached or refuses the message, the server
writes the warning "the notification of a task run was not sent" to the application log, and the email is
not tried again. So keep alerting on the metric `astrum_task_last_run_success == 0` as well, see
[monitoring](monitoring.md#what-to-watch).

## Privileges

| Privilege | Allows |
|---|---|
| `as-tasks-read` | Seeing tasks, their runs and logs |
| `as-tasks-create` | Creating tasks |
| `as-tasks-update` | Changing tasks |
| `as-tasks-delete` | Deleting tasks |
| `as-tasks-run` | **Run Now** and **Stop** |
| `as-tasks-all` | All of the above |

## REST API

| Request | Does |
|---|---|
| `GET /service/rest/v1/tasks` | Lists the tasks; `?type=<ID>` keeps one type |
| `GET /service/rest/v1/tasks/templates` | Lists the types that can still be created, with their settings |
| `GET /service/rest/v1/tasks/templates/<ID>` | Describes one type |
| `POST /service/rest/v1/tasks` | Creates a task; `409` when the name is taken |
| `GET`, `PUT`, `DELETE /service/rest/v1/tasks/<task id>` | Reads, changes or deletes a task; `PUT` changes only the fields it names |
| `POST /service/rest/v1/tasks/<task id>/run` | Starts a task; `409` when it is running or queued, `405` when it is disabled |
| `POST /service/rest/v1/tasks/<task id>/stop` | Stops a task; `409` when it is not running |

The `frequency` of a task has `schedule` (`manual`, `once`, `hourly`, `daily`, `weekly`, `monthly` or
`cron`), `startDate` in milliseconds since 1970, `timeZoneOffset` such as `+02:00` or `Z`,
`recurringDays` (weekly `1`-`7` with `1` for Sunday, monthly `1`-`31` and `999` for the last day) and
`cronExpression`. Settings go into `properties` as strings.

```sh
curl -u "admin:$ASTRUM_PASSWORD" -X POST -H "Content-Type: application/json" \
  https://astrum.example.com/service/rest/v1/tasks -d '{
    "type": "repository.purge-unused",
    "name": "Purge the npm proxy",
    "frequency": {"schedule": "cron", "cronExpression": "0 0 3 ? * SUN", "timeZoneOffset": "+01:00"},
    "properties": {"repositoryName": "npm-proxy", "lastUsed": "90"}
  }'

curl -u "admin:$ASTRUM_PASSWORD" -X POST \
  https://astrum.example.com/service/rest/v1/tasks/<task id>/run
```

The answer to `POST /service/rest/v1/tasks` and `GET /service/rest/v1/tasks/<task id>` carries the `id`
of the task, its `currentState` (`WAITING`, `RUNNING`, or `DONE` for a finished one-off task),
`lastRunResult` (`OK`, `FAILED` or `CANCELED`), `nextRun` and `lastRun`.
