# Cleanup policies

A cleanup policy says which components of a repository may be deleted: for example snapshots nobody
downloaded for a month, or cached packages older than a year. A policy does nothing on its own. It is
assigned to hosted and proxy repositories, and a scheduled task deletes what the policies of each
repository match.

Manage policies under **Settings → Repository → Cleanup Policies**, or with the REST API under
`/service/rest/v1/cleanup-policies`.

## Creating a policy

Click **Create Cleanup Policy** and fill in:

- **Name**: letters, digits, `-`, `_` and `.`, not starting with `_` or `.`, at most 255 characters and
  unique without regard to case. The name cannot be changed later.
- **Format**: the format of the repositories that may use the policy, or **All Formats**. A policy for
  all formats offers only the age and usage criteria.
- **Description**: why the policy exists, up to 4000 characters.

Then turn on the criteria.

## Criteria

A component is deleted when it matches every criterion that is turned on.

| Criterion | REST field | A component matches when |
|---|---|---|
| **Component Age** | `criteriaLastBlobUpdated` | every file of it was stored more than this many days ago: uploaded to a hosted repository, or fetched from the remote by a proxy |
| **Component Usage** | `criteriaLastDownloaded` | none of its files was downloaded within this many days; a file that was never downloaded counts from the day it was stored |
| **Release Type** | `criteriaReleaseType` | it is a release (`RELEASES`) or a pre-release (`PRERELEASES`), for the formats in the table below. The form offers **Releases and pre-releases**, **Releases** and **Pre-Releases/Snapshots** |
| **Asset Name Pattern** | `criteriaAssetRegex` | at least one of its files has a path that matches the regular expression |

Days are whole numbers from 1 to 36500. The form needs at least one of age, usage and pattern; **Release
Type** narrows them. Over REST a policy may consist of the release type alone, and then deletes every
component of that release type. A policy without criteria deletes nothing.

There is no criterion that keeps the newest versions of a package; the REST API refuses `retain` and
`sortBy`.

### Release types

Only formats whose versions tell releases from pre-releases offer **Release Type**:

| Format | A pre-release is |
|---|---|
| Maven (`maven2`) | a `-SNAPSHOT` version |
| Ansible Galaxy (`ansible`), Bazel (`bazel`), Hex (`hex`), NuGet (`nuget`), pub (`pub`), Puppet Forge (`puppet`) | a version with a `-` before any `+` build suffix, as in `1.2.0-rc.1` |
| Open VSX (`openvsx`) | an extension published as a pre-release, or a version with a `-` before any `+` |
| Alpine (`alpine`) | a version with an `_alpha`, `_beta`, `_pre` or `_rc` suffix |
| APT (`apt`) | a version with a `~` |
| Yum (`yum`) | a version with a `~`, or a release that starts with `0.` |
| CocoaPods (`cocoapods`) | a version with a letter or a `-` |
| Firefox add-ons (`firefox`), RubyGems (`rubygems`), Vagrant (`vagrant`) | a version with a letter |
| JetBrains plugins (`jetbrains`) | a plugin in a channel other than the default one, or a version with a word such as `alpha`, `beta`, `rc`, `eap`, `snapshot`, `nightly`, `preview`, `pre` or `dev` |

The other formats have no release type criterion.

### Asset name pattern

The pattern must match the whole path of a file, with or without its leading `/`. Start it with `(?i)`
to ignore case. Examples:

| Pattern | Matches |
|---|---|
| `.*-SNAPSHOT.*` | files of Maven snapshot versions |
| `nightly/.*` | everything below `nightly/` in a raw repository |
| `(?i).*\.zip` | ZIP archives, whether the name ends in `.zip` or `.ZIP` |

The database evaluates the pattern, so constructs it would read differently are refused, for example
flags other than a leading `(?i)`, named groups, word boundaries, `\A` and `\z`, and Unicode classes
such as `\p{L}`.

## Assigning a policy to a repository

Open the repository under **Settings → Repository → Repositories**. In its **Cleanup** section,
**Cleanup Policies** shows the **Available Policies**, those for its format and for all formats, and
the **Applied Policies**; move policies between them with the buttons between the lists (**Apply
selected**, **Apply all**, **Remove selected**, **Remove all**), then save. Over REST the repository
takes:

```json
"cleanup": { "policyNames": ["maven-snapshots-30d"] }
```

A repository can have several policies; each deletes what it matches. Group repositories have no
cleanup policies, since they hold no content of their own: assign policies to their members.

## Previewing

The policy form shows what the policy would delete if it ran now, for the criteria as they are on the
screen, saved or not. Under **Preview Cleanup Results** choose a **Preview Repository** of the format
to list the matching components with their group, name and version, page by page. **Download CSV
Report** saves every file of those components with its namespace, name, version, path, blob store, size,
last download and the date it was stored.

**Calculate** under **What this policy would delete** counts the components, files and bytes. On a large
repository the count stops after two minutes and shows what it found so far as a lower bound.

## When cleanup runs

The task **Admin - Cleanup repositories using their associated policies** applies the policies. A new
installation creates it as **Cleanup service**, running daily at 01:00 UTC. Change its schedule, or
start it with **Run Now**, under **Settings → System → Tasks**. There is only one task of this type.

The task goes through every hosted and proxy repository that has policies and applies each policy in
turn. It writes to its log what each policy deleted, and warns about a policy that no longer exists or
is for another format.

## What gets deleted

The task deletes whole components: the component and every file that belongs to it, also files the
criteria did not look at. Files that belong to no component, such as indexes and other metadata, are
never deleted by a cleanup policy. A component deleted from a proxy is fetched from the remote again
the next time a client asks for it.

Every deleted component is recorded in the audit log with the repository, the policy, the number of
files and their size.

For Docker a component is a tag. Cleanup deletes the tag; the manifests and layers no tag uses any more
are deleted from hosted Docker repositories by the task **Docker - Delete unused manifests and images**.

## Reclaiming space

There is no soft deletion and no trash: deleting a component deletes its files from the blob store at
once, so the space is free as soon as the task has run. A file that could not be deleted, for example
because its blob store did not answer, stays in the store without an asset; the task **Admin - Compact
blob store** removes such files later. See [reclaiming space](../configure/blob-stores.md#reclaiming-space)
for these tasks and their defaults.

Proxy repositories have one more task, **Repository - Delete unused components**, which deletes every
cached file, metadata included, that nobody downloaded for a number of days. Create it under
**Settings → System → Tasks** if a proxy should not grow without end.

## Changing and deleting policies

A change to a policy applies from the next run of the task. Its format can only change to **All
Formats**, or to another format while no repository of a different format uses it.

Deleting a policy takes it off every repository that uses it; those repositories are no longer cleaned
up by it.

## REST API

| Method and path | Use |
|---|---|
| `GET /service/rest/v1/cleanup-policies` | List the policies |
| `POST /service/rest/v1/cleanup-policies` | Create a policy; answers `201` |
| `GET`, `PUT`, `DELETE /service/rest/v1/cleanup-policies/<name>` | Read, change or delete a policy |

The format is named as the server names it, `maven2` for Maven and `*` for all formats:

```sh
curl -u admin -X POST https://astrum.example.com/service/rest/v1/cleanup-policies \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "maven-snapshots-30d",
    "format": "maven2",
    "notes": "Snapshots nobody downloaded for a month",
    "criteriaLastDownloaded": 30,
    "criteriaReleaseType": "PRERELEASES"
  }'
```

Then add the policy to the `cleanup` section of a repository (see the [repositories](overview.md#rest-api)
page) and, to clean up at once, start the task:

```sh
curl -u admin 'https://astrum.example.com/service/rest/v1/tasks?type=repository.cleanup'
curl -u admin -X POST https://astrum.example.com/service/rest/v1/tasks/<id>/run
```
