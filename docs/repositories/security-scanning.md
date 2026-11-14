# Vulnerability scanning

The vulnerability scanner finds known vulnerabilities and malicious packages in the packages that
components depend on, shows them in the web interface, the REST API and over MCP, and can refuse
downloads of components that a content policy blocks. It is off by default. Nothing is scanned, fetched
or refused until an administrator creates the advisory task, imports advisory data, creates a content
policy and attaches it to a repository.

The rules that decide what is refused are described in [Content policies](policies.md). This page covers
the scanner itself.

## Supported formats

This version scans the packages of these formats:

| Format | Ecosystem of the advisory data |
|---|---|
| Maven (`maven2`) | `Maven` |
| npm | `npm` |
| PyPI | `PyPI` |
| Go | `Go` |
| Cargo | `crates.io` |
| NuGet | `NuGet` |

A component of any other format is recorded with the scan state `unsupported` and is never matched. Container
images are not scanned in this version: the scanner has no inventory for the Docker format, so tags and
digests are not judged and not refused. Advisory data for OS packages (Alpine, Debian) can be imported, but no
component reads it yet.

## Advisory data

Advisories come from OSV, a database of open source vulnerability records. OSV publishes one ZIP archive
per ecosystem at `<base>/<Ecosystem>/all.zip`. The default base is
`https://osv-vulnerabilities.storage.googleapis.com`. The server downloads an archive only when a task
asks for it, so an installation makes no outbound calls until an administrator creates the task.

### The advisory task

The task kind is `advisories.sync`. Only one run can be active at a time. Its settings:

| Setting | Default | What it does |
|---|---|---|
| Source URL (`url`) | the OSV base above | The base URL of the dumps. Point it at an internal mirror to run without internet access. |
| Ecosystems (`ecosystems`) | `Maven,npm,PyPI,Go,crates.io,NuGet,Alpine,Debian` | Comma-separated OSV ecosystem names to download and import. |
| Stale after days (`staleAfterDays`) | 7 | How long advisory data may age before an alert is sent. From 1 to 3650. |

To create the task, open the **Vulnerability Data** page under **Settings → Security** and use
**Create advisories.sync task**. Run it once after creating it; later runs refresh the data on the schedule
you set.

On a site without internet access, download the `all.zip` archive for each ecosystem on a connected
machine and either upload it on the **Vulnerability Data** page, or send it to the REST endpoint below.
An upload is one ecosystem at a time.

### What an import does

- Each record is read leniently. Unknown fields are ignored, and a record that cannot be read is skipped and
  counted.
- A record whose `modified` time did not change is skipped. A changed record updates its advisory and the
  packages it affects, so the next match uses the new data.
- Advisories that disappeared from an ecosystem's dump are deleted. An import that would remove more than
  100 advisories and more than 20 percent of the advisories it holds is refused, unless it is forced with
  `force=true`. This guards against an empty or truncated dump.
- Withdrawn advisories stay in the database, marked as withdrawn, and never produce findings.
- Severity comes from the advisory's `database_specific.severity` text, else from the CVSS v3 base score,
  else it is `unknown`. Malicious package records (`MAL-` identifiers) are `critical` and marked
  `malicious`.

Limits, applied to every archive and record:

| Limit | Value |
|---|---|
| Archive size | 2 GiB |
| Entries in an archive | 1,000,000 |
| Uncompressed size of one entry | 1 MiB |
| Compressed size of one entry | 2 MiB |
| Uncompressed size of the whole archive | 16 GiB |
| One record | 256 KiB; over that size, the fields `affected`, `references`, `credits` and `database_specific` are dropped until it fits |

### Cost in disk and time

The design sets a target of 300 MiB of memory for an import, and the import reads the archive as a stream.
It does not give measured numbers for the size of an archive, the number of rows, the database size or the
duration of an import. Measure them on your own data before you size a server.

### When the source is unreachable

A failed sync leaves the old data in place, and scans keep matching against it. The **Vulnerability Data**
page shows the age of the data and the last error per ecosystem. When the data is older than the stale
limit, the server sends an `advisories.stale` alert to the notification channels that subscribe to
`vulnerability.found`, and repeats it at most once a day for each ecosystem. See
[Notifications](#notifications).

## How scanning works

A scan has two parts. The scanner first inventories a component: it works out the packages the component
depends on, with their ecosystem, name and version. Then it matches each distinct package against the
advisories. A package that several components share is matched once. A finding is a join between the
package and the advisory, so an advisory that changes updates one place for every component that uses the
package.

### When scans run

A component is queued for a scan in these cases, but only in repositories that have a content policy:

- a component is created, or the file of a component changes;
- a policy is attached to a repository, which queues every component of the repository;
- someone asks for a scan: the REST endpoint `POST /service/rest/v1/security/scan` or the `rescan` tool over
  MCP. The body is `{"repository": "<name>"}`, with an optional
  `componentId` to scan one component. A group has no components of its own and cannot be scanned. The
  response gives the number of queued components.
- the task `scan.repository` runs. Its settings are the repository (`repositoryName`, or all repositories
  with a policy) and **Extract again** (`reextract`), which queues every component. Without it, only
  components that were never scanned, were scanned by an older extractor or whose last scan failed are
  queued.

There are no periodic rescans. When an advisory changes, only the packages whose ecosystem and name changed
are matched again.

A download in a repository with a `block` mode does not wait for a queued scan. If the component has no
verdict yet, the server scans it at download, with a budget of 2 seconds. See [Verdicts and downloads](#verdicts-and-downloads).

### The queue

The queue lives in PostgreSQL, so several nodes can share it. Each node runs `scan.workers` workers, 2 by
default and at most 8. A worker takes a batch of queued components and locks them so that no other worker
takes the same rows. A scan that does not finish in 10 minutes is stopped. A failed scan is retried after 1,
5 and 30 minutes. After the fourth failure its state becomes `failed` and the scanner does not try again
until someone asks for a rescan.

Two more settings control the background work on each node: `scan.matchers` (1 by default, at most 4) runs
the matcher that compares packages with advisories, and `scan.verdictWorkers` (1 by default, at most 4)
turns the matches into verdicts.

### Scan states

The scan state of a component says how far its scan got. The **Security** tab of a component and the
`/service/rest/v1/components/{id}/security` endpoint show it:

| State | Meaning |
|---|---|
| `queued` | The component is waiting for a scan. |
| `unscanned` | The component has never been scanned. |
| `scanned` | The inventory is complete and the packages were matched. |
| `incomplete` | The scanner read part of the component only. |
| `unsupported` | The format has no inventory in this version. |
| `failed` | The scan failed on every retry. |

A component that has never been scanned is `unscanned`. The verdict of a policy in `block` mode with
`unscanned: block` refuses it, because the component has not been judged yet. Read [Content
policies](policies.md) before you choose this setting.

## Verdicts and downloads

A verdict is the result of one policy for one component: whether the component is blocked, why, and the
findings behind the decision. It is stored with the policy revision it was judged against, so a changed
policy leads to a new verdict. The verdict states are:

| State | Meaning |
|---|---|
| `pass` | Nothing in the policy applies to the component. |
| `report` | The policy would block the component, but its mode is `report`. |
| `block` | The policy blocks the component. |
| `unscanned` | The component has no completed scan, and the policy does not block unscanned components. |
| `error` | The policy could not be evaluated. |

The policy mode decides what a download sees:

- `off`: the policy is not judged and downloads are served.
- `report`: the server judges the component and records what it would block. Downloads are served. The
  component page and the REST endpoint show **Would be blocked**.
- `block`: the server refuses downloads of components that the verdict blocks.

In `block` mode a download uses the stored verdict. If there is none, the server scans the component
inside the request. The scan has a budget of 2 seconds. If the scan finishes, its verdict decides. If it
does not finish in time or fails, the policy's `unscanned` setting decides: `allow` serves the file,
`block` refuses it.

### The refusal

A refused download gets HTTP status `403 Forbidden`. The body is a plain text line:

```text
blocked by policy <policy>: <reasons>
```

The header `x-astrum-blocked-by` carries the name of the policy. The reasons list each blocking finding, for
example `GHSA-xxxx-xxxx-xxxx: high in npm/lib@1.0.0, fixed in 1.0.1`. Malicious packages are marked
`malicious`. A component that has not been scanned gets `the component has not been scanned yet`, and a
policy that cannot be read gets `the policy cannot be read`. The line is cut to 512 characters, with an
ellipsis at the end when it is longer.

Each refusal is written to the audit log. The log keeps one entry per component per hour, so a client that
retries does not fill it. The refusal also triggers the `policy.blocked` notification, at most once a day
for each component.

The refusal names the policy and the reasons, but it does not link to the interface. Find the component in
the browser to see the full verdict.

Downloads are refused only for the components that a policy applies to: those whose repository holds the
policy. A group serves a member's file under the policy of the member that stores it.

## Exceptions

An exception lets a specific finding through. It is the only way through a block: an administrator's rights
do not skip a policy. An exception has:

- **Rule**: `vulnerability` in this version. Licence and signature exceptions can be stored, but nothing
  applies them yet.
- **Policy**: one policy, or every policy when the field is left out.
- **Target**: at least one of an advisory identifier or alias (such as `GHSA-xxxx-xxxx-xxxx` or `CVE-2024-0001`),
  a component name, or an image digest. Advisory identifiers are compared without regard to case. The
  format, namespace and version narrow a name target further.
- **Reason**: why the finding is accepted. It is required and appears in the verdict as `Excepted: <reason>`.
- **Expiry**: optional. An expired exception stops applying, and the server re-judges the affected
  components within one sweep, which runs every minute.

An image digest target has no effect in this version, because container images have no verdicts.

Exceptions are created in the interface on the policy exceptions page, under `/admin/security/policy-exceptions`,
or over REST, and by the `create_policy_exception` tool over MCP. The REST endpoints are described in
[Content policies](policies.md#rest-api).

## Findings

### In the interface

- **Component page, Security tab.** The scan state, the policy, the verdict with its reasons, and a table of
  vulnerabilities: advisory, aliases, severity, score, summary, fixed version, and whether the finding is
  excepted. The table lists up to 1,000 vulnerabilities, worst first, and says when some are left out.
  When no policy applies to the repository the tab says so, and it does not show a pass.
- **Repository list.** A **Security** badge shows the number of blocked components in each repository.
- **Dashboard.** The **Vulnerabilities** card shows the number of blocked components, the number of scanned
  and vulnerable components, the age of the advisory data and a warning when the data is stale. The card links
  to the **Vulnerability Data** page.

A user sees the findings of a component only when the user may browse its repository.

### REST

All endpoints are under `/service/rest/v1`.

| Endpoint | Returns |
|---|---|
| `GET /components/{id}/security` | The scan state, the policy and the verdict of one component, and its vulnerabilities. The `license` field is `null` in this version. |
| `GET /security/findings` | One row per component and advisory. Parameters: `repository`, `severity` (`critical`, `high`, `medium`, `low` or `unknown`), `advisory` (an identifier or alias), `blocked` (`true` or `false`), `limit` (from 1 to 200, 50 by default) and `continuationToken`. |
| `GET /security/advisories/{id}` | One advisory with its aliases, summary, severity, score, references, affected packages and the components it affects that the caller may see. |
| `GET /security/summary` | Totals per severity, the number of blocked, scanned and vulnerable components, and the same figures per repository. |

Components and advisories are not returned to a caller who may not browse the repository. The list endpoints
return only the repositories that the caller may browse.

### MCP

These tools are available to an agent. [Agents over MCP](../use/mcp.md) lists them with their arguments.

| Tool | Needs |
|---|---|
| `get_component_security` | a signed-in user |
| `list_vulnerable_components` | a signed-in user |
| `find_advisory` | a signed-in user |
| `rescan` | `as-policies-update` |
| `create_policy_exception` | `as-policyexceptions-create` |

## Notifications

The scanner adds two events to the notification channels, next to `component.published`, `component.updated`
and `component.deleted`. The form of a channel offers them under **Security**:

| Event | Sent when |
|---|---|
| `vulnerability.found` | A component of a repository of the channel gains a finding at or above the `notify` threshold of the policy. This happens after an upload or an advisory update. |
| `policy.blocked` | A download was refused. At most one notice is sent per component per day. |

The `advisories.stale` alert is sent to the channels that subscribe to `vulnerability.found`, when the
advisory data is older than the stale limit. It repeats at most once a day for each ecosystem.

## Limits and known gaps

These are the limits of this version. Read them before you turn on `block` in production.

- **Container images are not scanned or refused.** The Docker format has no inventory, and the gate does not
  check manifests. A `block` policy on a Docker repository does nothing.
- **Only the download is refused.** Package metadata (npm packuments, the PyPI simple index,
  `maven-metadata.xml`) still lists a blocked version, so a resolver can choose it and then fail on the
  download.
- **Licence and signature rules are stored but not enforced.** The policy object and the interface accept
  them. Nothing in this version refuses a download because of a licence or a signature.
- **Digest exceptions have no effect**, for the reason given in [Exceptions](#exceptions).
- **First scans are not complete.** A component that has never been scanned is `unscanned` until the queue
  reaches it. In `block` mode the first download waits at most 2 seconds for an inline scan. With
  `unscanned: allow` (the default) the download is served without a verdict. With `unscanned: block` it is
  refused. Components of formats without an inventory get the state `unsupported`, so `unscanned: block`
  refuses them too.
- **No state for missing advisory data.** The design says that a scan with no advisory data ever imported ends
  as `no-data`, and the gate then applies `unscanned`. This version reads that state but never writes it. A
  component scanned before any import may be recorded as `scanned` with no findings, and then it passes. Test
  this on a new installation before you rely on `unscanned: block`.
- **Per-node caches.** Each node keeps a copy of the policies and exceptions and refreshes it at most once a
  second. A change made on one node reaches the others within that second. Verdicts are stored in PostgreSQL,
  so every node reads the same ones.
- **Advisory data is global.** There is one set of imported advisories for the whole server; an ecosystem
  cannot be updated on one node only.

## Notes on untrusted text

Advisory text comes from OSV and is not trusted. Before it reaches the interface, the REST API or a refusal,
the server removes control characters and the Unicode characters that change the direction of text, and cuts
the text to 256 characters with an ellipsis. A verdict lists at most 200 reasons; the rest are counted and
shown as "more reasons are not shown". Policy names, notes, reasons and targets have their own length limits,
and the REST API rejects longer values.
