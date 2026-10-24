# Routing rules

A routing rule decides by the path of a request whether a proxy repository serves it. Its main use is to
keep requests for your own packages away from public registries: when a group may ask its proxy for
`com.example` artifacts or `acme-*` packages, a package of the same name that someone else published
there can end up in your builds. A rule assigned to the proxy makes the group skip it for those paths.

Manage rules under **Settings → Repository → Routing Rules**, or with the REST API under
`/service/rest/v1/routing-rules`. Both need the `as-all` privilege, which the `as-admin` role has.

## How a rule decides

A rule has a mode and one or more matchers. A matcher is a regular expression that must match the
whole path of the request.

| Mode | A request is served when |
|---|---|
| **Block** | its path matches none of the matchers |
| **Allow** | its path matches at least one matcher |

The path is the part of the URL after the repository name, starting with `/` and without the query
string. For `https://astrum.example.com/repository/maven-central/com/example/app/1.0/app-1.0.jar` it is
`/com/example/app/1.0/app-1.0.jar`. For Docker repositories it is the registry path from `/v2/` on, the
same for path routing, connector ports and subdomains: `docker pull astrum.example.com/docker-hub/library/alpine:3.20`
asks for `/v2/library/alpine/manifests/3.20`, which `/v2/library/.*` matches.

Matchers are case-sensitive; start one with `(?i)` to ignore case. The syntax is that of the Rust
`regex` crate, which has no look-around and no back-references. Since a matcher must match the whole
path, `com/example/.*` matches nothing (paths start with `/`) and `/com/example/` matches only that
folder itself; write `/com/example/.*`.

## Where rules apply

- **Proxy repositories.** A request to the proxy for a path its rule does not serve is answered `403`
  with the text `Routing rules block the requested item from this repository`, and the remote is not
  asked.
- **Groups.** A group skips a member whose rule does not serve the path, as if the member did not have
  the file. When no other member has it, the group answers `404`. For documents a group merges from its
  members, such as `maven-metadata.xml`, the skipped member contributes nothing.
- **Hosted repositories** have no routing rule.

The repository form offers routing rules for proxies only. Over REST a group can carry
`routingRuleName` too; it then answers `403` itself for the paths its rule does not serve, and a group
that contains it skips it for those paths.

A changed rule takes effect within ten seconds.

## Creating a rule

Click **Create Rule** and fill in:

- **Name**: letters, digits, `-`, `_` and `.`, not starting with `_` or `.`, at most 200 characters
  and unique without regard to case. The form cannot change the name later.
- **Description**: optional.
- **Mode**: **Block** (the default) or **Allow**.
- **Matchers**: one or more, added with **Add Matcher**. Each has a kind: **Regex** takes a regular
  expression as it is, while **Starts with**, **Ends with** and **Contains** take plain text and turn it
  into an expression (`/org/apache/` under **Starts with** becomes `^/org/apache/.*`). The rule stores
  regular expressions, so reopened rules show them as such.

**Test Path** below the matchers checks a path against the matchers on the screen: it marks each matcher
that matches and says whether the request would be allowed or blocked. **Rule Summary** repeats the rule
in words.

## Assigning a rule to a repository

Open the proxy under **Settings → Repository → Repositories**. In its **Routing Rule** section choose
the rule under **Routing Rule**, or **None (allow every request)**, and save. Over REST the repository
takes:

```json
"routingRuleName": "no-internal-groups"
```

A repository has at most one rule; one rule can serve many repositories. The **Repositories** column of
the list of rules counts them.

## Testing across repositories

**Preview** on the list of rules opens **Global Routing Preview**. Enter a **Test Path**, choose a
**Filter** (**All Repositories**, **Group Repositories** or **Proxy Repositories**) and click **Test**.
The result lists every group with its members, and every proxy, each marked **Allowed** or **Blocked**
with its rule; select a rule to see its patterns. It shows what the rules decide, not whether a member
has the file.

## Examples

**Your own Maven groups never come from Maven Central.** The group `maven-public` has the members
`maven-releases` (hosted) and `maven-central` (proxy). Create a rule `no-internal-groups` with the mode
**Block** and the matchers `/com/example/.*` and `/org/example/.*`, and assign it to `maven-central`.
A build asking `maven-public` for `/com/example/app/1.0/app-1.0.jar` gets the file from
`maven-releases` or a `404`, and the request never reaches Maven Central.

**Internal Python packages never come from PyPI.** Your packages are named `acme-*`. Create a rule with
the mode **Block** and the matcher `/simple/acme-.*`, and assign it to the PyPI proxy. pip looks up
`/simple/acme-tools/` in the group, which leaves the proxy out, so a package of the same name on PyPI is
never offered.

**A proxy that serves one directory of a vendor's site.** A raw proxy of `https://downloads.example.org/`
should only fetch the vendor's CLI. Create a rule with the mode **Allow** and the matcher `/tools/cli/.*`
(**Starts with** `/tools/cli/`). Requests for anything else are answered `403`.

## Changing and deleting rules

A change to the mode or the matchers applies to every repository that uses the rule. Over REST a rule
can also be renamed by sending a new `name` with `PUT`; the repositories that use it follow.

A rule that is assigned to a repository cannot be deleted: take it off every repository first. The page
of the rule names them.

## REST API

| Method and path | Use |
|---|---|
| `GET /service/rest/v1/routing-rules` | List the rules |
| `POST /service/rest/v1/routing-rules` | Create a rule; answers `204` |
| `GET`, `PUT`, `DELETE /service/rest/v1/routing-rules/<name>` | Read, change or delete a rule |

```sh
curl -u admin -X POST https://astrum.example.com/service/rest/v1/routing-rules \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "no-internal-groups",
    "description": "Our own Maven groups never come from Maven Central",
    "mode": "BLOCK",
    "matchers": ["/com/example/.*", "/org/example/.*"]
  }'
```

Then set `routingRuleName` in the settings of the proxy, see the [repositories](overview.md#rest-api)
page. Deleting a rule that is still assigned answers `400` with the number of repositories that use it.
