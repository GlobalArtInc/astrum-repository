# Content policies

A content policy is a named set of rules. It decides which components of a repository the server judges,
and what a download of a blocked component gets. The rules are judged by the [vulnerability
scanner](security-scanning.md); the policy only says what to do with the result. A policy does nothing until
it is attached to a repository.

A policy has three blocks: `vulnerabilities`, `licenses` and `signatures`. The `vulnerabilities` block is
enforced for the formats the scanner reads; components of other formats are never matched. The `licenses`
block is enforced for the package formats listed in [Licences](licenses.md). The `signatures` block is
enforced for Docker repositories only, by the check described in [Image signatures](image-signatures.md).
For Docker images, the `vulnerabilities` and `signatures` blocks apply, and the `licenses` block does not.
The rest of this page describes the `vulnerabilities` block in detail.

## The policy object

| Field | Type | Meaning |
|---|---|---|
| `name` | text | The name. Letters, digits, `-`, `_` and `.`; it must not start with `_` or `.`; at most 64 characters. It is fixed once created. |
| `notes` | text | Free text, at most 1000 characters. |
| `vulnerabilities` | object | The rule block described below. |
| `licenses` | object | Enforced for the formats listed in [Licences](licenses.md), not for Docker images: `mode`, `list` (`allow` or `deny`), `ids` (up to 1000 SPDX identifiers) and `unknown` (`allow`, `report` or `block`, `report` by default). |
| `signatures` | object | Enforced for Docker repositories: `mode` and `keys` (up to 64 names of trusted keys). See [Image signatures](image-signatures.md). |
| `revision` | number | Set by the server. It goes up on every change to the policy, and a verdict is judged against one revision. |

The REST API rejects unknown fields. A misspelt field name is an error, and it does not turn into a default.

### The vulnerabilities block

| Field | Values | Default | Meaning |
|---|---|---|---|
| `mode` | `off`, `report`, `block` | `off` | What the server does with the verdicts. See below. |
| `severity` | `critical`, `high`, `medium`, `low`, `unknown` | none | The threshold. A finding at or above it blocks the component. Without a threshold, only malicious packages can block. |
| `malicious` | `block`, `threshold` | `block` | `block`: a malicious package always blocks. `threshold`: a malicious package is judged like any other finding at its severity, which is `critical`. |
| `unscanned` | `allow`, `block` | `allow` | What happens to a component that has not been scanned yet, or whose scan did not finish. `allow` serves it. `block` refuses it. |
| `notify` | a severity | none | The threshold for the `vulnerability.found` notification. |

The modes do the following:

- `off`: the server does not judge the components of the repository, and downloads are served.
- `report`: the server judges every component and records what it would block. Downloads are served. The
  interface and REST show **Would be blocked**, so you can watch the effect before you enforce it.
- `block`: the server refuses downloads of blocked components with HTTP 403. See
  [the refusal](security-scanning.md#the-refusal).

## Attaching a policy to a repository

A repository holds at most one policy, in its `policy` section:

```json
{"policy": {"name": "strict"}}
```

Choose the policy in the repository form, under the content policy field. Hosted and proxy repositories can
have one. A group has none of its own: a download through a group is judged by the policy of the member that
stores the file.

Attaching a policy queues every component of the repository for a scan. Changing the policy changes its
revision, and the verdicts are judged again.

A policy that a repository uses cannot be deleted. The server answers `409 Conflict` and lists the
repositories that use it. When a policy is deleted, its exceptions go with it.

## Administration in the interface

These screens are under **Settings → Security**. Each one needs the right shown in brackets.

| Screen | Path | Needs |
|---|---|---|
| Content Policies, the list | `/admin/security/policies` | `as-policies-read` |
| Create a policy | `/admin/security/policies/create` | `as-policies-read` and `as-policies-create` |
| Edit a policy | `/admin/security/policies/<name>` | `as-policies-read` |
| Policy Exceptions | `/admin/security/policy-exceptions` | `as-policyexceptions-read` |
| Trusted Keys | `/admin/security/trusted-keys` | `as-policies-read` |
| Vulnerability Data | `/admin/security/vulnerability-data` | `as-policies-read` |

The **Vulnerability Data** page lists the advisory feeds with their record counts, the time of the last sync and
the last error. It has an upload for an OSV archive and a link that creates the `advisories.sync` task. See
[Advisory data](security-scanning.md#advisory-data).

## Privileges

Each privilege is named as the interface shows it. The permission string the server checks is in brackets.

| Privilege | Permission | Grants |
|---|---|---|
| `as-policies-read` | `astrum:policies:read` | Read policies, trusted keys and advisory feeds. |
| `as-policies-create` | `astrum:policies:create` | Create policies and trusted keys. |
| `as-policies-update` | `astrum:policies:update` | Change policies and trusted keys, import advisory archives, and queue rescans. |
| `as-policies-delete` | `astrum:policies:delete` | Delete policies and trusted keys. |
| `as-policies-all` | `astrum:policies:*` | Every action above. |
| `as-policyexceptions-read` | `astrum:policyexceptions:read` | List exceptions. |
| `as-policyexceptions-create` | `astrum:policyexceptions:create` | Create exceptions, without editing policies. |
| `as-policyexceptions-delete` | `astrum:policyexceptions:delete` | Delete exceptions. |
| `as-policyexceptions-all` | `astrum:policyexceptions:*` | Every action on exceptions. |

Security staff can have `as-policyexceptions-*` without `as-policies-*`: they can grant exceptions but cannot
change the rules. Anyone who may browse a repository sees the findings of its components. The findings do not
need a policy privilege.

## REST API

All endpoints are under `/service/rest/v1/security`.

### Policies

| Method and path | What it does | Needs |
|---|---|---|
| `GET /policies` | Lists the policies. | `as-policies-read` |
| `POST /policies` | Creates a policy. The body is the policy object without `revision`. | `as-policies-create` |
| `GET /policies/{name}` | Reads one policy. | `as-policies-read` |
| `PUT /policies/{name}` | Replaces the policy. The body is the policy object. | `as-policies-update` |
| `DELETE /policies/{name}` | Deletes the policy. Answers `409` while a repository uses it. | `as-policies-delete` |

A rule block that is given needs its `mode`. A policy with no `mode` in a block is refused with a validation
error that names the field.

### Exceptions

| Method and path | What it does | Needs |
|---|---|---|
| `GET /exceptions` | Lists the exceptions. | `as-policyexceptions-read` |
| `POST /exceptions` | Creates an exception. | `as-policyexceptions-create` |
| `DELETE /exceptions/{id}` | Deletes an exception. | `as-policyexceptions-delete` |

The body of `POST /exceptions`:

| Field | Required | Meaning |
|---|---|---|
| `rule` | yes | `vulnerability`, `license` or `signature`. |
| `reason` | yes | Why the exception is justified. At most 1000 characters. |
| `policy` | no | The policy it applies to. Left out, it applies to every policy. |
| `advisory` | one of the targets | An advisory identifier or alias. |
| `name` | one of the targets | The component name. |
| `digest` | one of the targets | An image digest, `sha256:` or `sha512:` with lowercase hexadecimal digits. It covers that image only. |
| `license` | for `license` only | An SPDX identifier. |
| `format`, `namespace`, `version` | no | Narrow a name target to one format, namespace or version. |
| `expiresAt` | no | When the exception ends, as an RFC 3339 time. |

An exception needs at least one of `advisory`, `license`, `name` or `digest`. A `license` exception needs a
`license` target and cannot name an advisory. A `vulnerability` exception cannot name a license. A `signature`
exception needs a `policy` and cannot name an advisory. An unknown `format` is refused.

A `name` exception covers an image through the tags of the repository that point at its digest; at most 50 such
tags are considered for each digest. An advisory exception covers the image when the scanner finds that advisory
in it. An exception takes effect when the verdict is judged again.

### Trusted keys

| Method and path | What it does | Needs |
|---|---|---|
| `GET /keys` | Lists the trusted keys. | `as-policies-read` |
| `POST /keys` | Adds a trusted key. The body has `name`, `publicKey` (a PEM public key, at most 8 KiB) and `notes`. | `as-policies-create` |
| `DELETE /keys/{name}` | Deletes a trusted key. | `as-policies-delete` |

The server reads the algorithm and the fingerprint from the PEM and returns them with the key. A private key is
refused, with the message that the public key must be given. Keys are used by the `signatures` block, which
the signed-only check of Docker repositories enforces.

### Advisory data

| Method and path | What it does | Needs |
|---|---|---|
| `GET /advisories/feeds` | Lists the advisory feeds per ecosystem: record count, last sync, last attempt and last error. | `as-policies-read` |
| `POST /advisories/import?ecosystem=<name>` | Imports an OSV `all.zip` for one ecosystem. The body is the raw archive. `force=true` lifts the guard against large removals. | `as-policies-update` |

### Rescans

| Method and path | What it does | Needs |
|---|---|---|
| `POST /scan` | Queues the components of a repository with a policy for a scan. Body: `{"repository": "<name>"}`, with an optional `componentId`. | `as-policies-update` |

## Rollout

Turn the scanner on one step at a time. Each step is a change of the policy, and each step can be undone.

1. Create the advisory task and let it finish. Check the **Vulnerability Data** page: every ecosystem you need
   should show records and a recent sync.
2. Create a policy with `mode: report`. Use a severity threshold that you can act on, for example `high`.
   Leave `unscanned` at `allow`.
3. Attach the policy to one repository that does not serve production traffic, and wait until the scans finish.
4. Read the findings: the **Security** badge in the repository list, the dashboard card, and
   `GET /security/summary`. Look at the components that show **Would be blocked**.
5. For each finding you accept, add an exception with a reason and, where possible, an expiry date. Fix the
   components you can update.
6. Switch the mode to `block` on the same repository and watch the refusals in the audit log and the
   `policy.blocked` notifications for a day.
7. Attach the policy to more repositories, one at a time. Do not attach a `block` policy to a proxy that feeds
   your build cluster until you have read the refusals of a full day.

Set `unscanned: block` last. It refuses every component that has not been scanned, including the components that
are not supported in this version, so a new installation with `block` and `unscanned: block` refuses most
downloads until the scans finish.

## Audit

Every change is written to the audit log: policies (`created`, `updated`, `deleted`), exceptions (`created`,
`deleted`) and trusted keys (`created`, `deleted`). A change made through MCP is marked with `via: mcp`.
