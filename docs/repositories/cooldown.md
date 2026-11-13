# Release cooldown

A release cooldown is a defence against fresh malicious releases. A package version that was published
upstream fewer than N days ago is hidden from clients or refused to them, so a compromised release has
time to be noticed and yanked before the proxy hands it out. The cooldown belongs to a content policy and
applies to **proxy repositories** only: a hosted repository holds your own releases, so nothing in it is
held back. A group applies the policy of each member proxy to the versions that member holds.

The cooldown needs the content policy machinery: the repository must have a content policy assigned, and
the cooldown is one block of that policy. Without a policy nothing is held back.

## The policy block

Open **Settings → Security → Content Policies**, pick or create a policy, and fill in the **Cooldown**
block. Over REST it is the `cooldown` object of the policy:

| Field | Values | Meaning |
|---|---|---|
| `mode` | `off`, `report`, `block` | `off` (the default when the block is missing) does nothing; `report` only records; `block` hides and refuses |
| `minAgeDays` | 1 to 90, default 7 | a version becomes available this many days after its upstream publication |
| `exclude` | patterns | packages that are never held back |

`exclude` takes at most 100 patterns, each 1 to 200 characters. A pattern is matched against the package
name as clients write it: `@scope/name` for npm, the lower-case normalised project name for PyPI. The only
wildcard is `*`, which matches any text, so `@globalart/*` covers every package of an internal scope. Use
`exclude` for permanent opt-outs such as internal scopes.

For a one-off approval of a single version, use an exception instead (below).

## Exceptions

An exception of rule **Cooldown** lets one component through the cooldown: a package or a single version,
with a reason and an expiry, as for the other rules. Use it for an audited approval, for example a fresh
security fix that must reach clients before the minimum age. Exceptions are listed under
**Settings → Security → Policy Exceptions**, and they are the escape hatch for a version that a release
would otherwise hold back.

## Publication dates

The cooldown needs the date a version was published upstream. Only dates that the upstream registry sets
count; a date written by the package author is not trusted.

| Format | Where the date comes from | Coverage |
|---|---|---|
| npm | the packument's `time[version]` | proxy |
| PyPI | `upload-time` of each file in the JSON index | proxy; a project served only as HTML gives no date |

Maven, NuGet, cargo, Go and Docker are not covered yet: their versions are never held back by the cooldown.

A missing or unreadable date means the version is **allowed**. A date is accepted only between
2005-01-01 and one day in the future. Dates are stored per cached asset when the proxy fetches it, so an
asset cached before the feature has no date and is never hidden until it is fetched again.

For PyPI the date belongs to each file. A new wheel added to an old version is therefore held back on its
own. For npm the tarball is the version, so the date is per version.

## What clients see in block mode

**npm.** The documents of a proxied package (full, abbreviated, one version and dist-tags) omit the fresh
versions. `latest` then falls back to the highest version that is still visible, and any other tag that
pointed only at hidden versions disappears. `npm install` therefore picks the previous version. A package
whose versions are all hidden answers `200` with no versions.

**PyPI.** The project pages (JSON and HTML) omit the fresh files.

**Direct requests.** A lockfile that pins a hidden version, or any direct download of a hidden file, gets
`403` with the text:

```
cooldown: <name> <version> was published <date>; available from <date>
```

**Report mode** hides and refuses nothing. It only records the versions that the block would have held
back, so you can see them on the Cooldown page before you switch to `block`.

## The waiting list

Every refusal and every report hit is remembered per repository and component. An entry is written at most
once an hour per component per node, so the counts are approximate. The list shows:

- on the **Cooldown** page (**Settings → Security → Cooldown**), with the repository filter and paging;
- on the component page of a proxied version, as one line with its publication and availability dates;
- over REST at `GET /service/rest/v1/security/cooldown`, with `repository` and `continuationToken`
  parameters, 50 rows per page, only for repositories you may browse in full. Each row has `repository`,
  `format`, `namespace`, `name`, `version`, `publishedAt`, `availableAt`, `blocked`, `refusals` and
  `firstRefusedAt`;
- through the MCP tool `list_cooldown_waits` (see [Agents over MCP](../use/mcp.md)).

`blocked` tells whether the version was refused (`block`) or only reported (`report`). After a mode change
the flag can lag up to an hour, until the next hit of that component is written.

## Release notice

When a waiting version becomes available, the event `cooldown.released` ("Version released after cooldown")
is sent to the notification channels that chose it. The notice is announced only for versions that somebody
asked for while they were blocked; a report-only wait is removed without a notice.

The notice needs the task **Admin - Delete expired sessions and uploads** (`security.purge-expired`). Its
default schedule is manual, so give it a schedule. Channels choose the event in their form under
**Settings → System → Notifications**.

## Rolling out

1. Start with `mode: report` and the default `minAgeDays` of 7. Watch the Cooldown page for a week to see
   what would have been held back.
2. Add `exclude` patterns for your internal scopes and create exceptions for urgent security fixes that
   must not wait.
3. Switch to `mode: block`.

Keep in mind that a fresh security fix is hidden for N days unless an exception lets it through. Tags other
than `latest` that point at hidden versions disappear until those versions are released, so `npm install
pkg@next` can fail for that time.

## Limits

- A refusal is not written to the audit log; the waiting list is the only record of it.
- The `blocked` flag of a wait can lag up to an hour after a mode change.
- Hosted repositories are never held back, and neither are Maven, NuGet, cargo, Go and Docker versions for
  now.
