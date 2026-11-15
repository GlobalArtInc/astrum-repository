# Licences

A licence rule checks the licence each component declares against a list of licences. It is set in a
repository's policy, next to the vulnerability rule, and it says which licences a component may carry,
which it may not, and what to do when a component declares no licence or one the server cannot read.

The rule judges components of the package formats listed below. Docker images get no licence verdict:
the licences of their SBOMs are shown, not judged (see [SBOMs](sbom.md)).

The rule is the `licenses` block of a policy. Policies are managed with the REST API under
`/service/rest/v1/security/policies`. Creating, changing and deleting them needs the rights `astrum:policies:*`.

## Where a component's licence comes from

The licence of a component is read from the metadata already stored for it. The scan of a component stores
the result. The server fetches nothing more to find a licence, and it does not read the licence files inside
packages.

| Format | Declared text | Notes |
|---|---|---|
| npm (`npm`) | `license` of the version in the stored packument: a string, or the legacy `{ "type": … }` object. Without it, the `licenses` array, joined with `OR` | |
| PyPI (`pypi`) | 1. `license_expression`; 2. `license`, the free text, if it is at most 100 characters and on one line; 3. the `License ::` classifiers | A classifier in the built-in table maps to its SPDX id. Several classifiers are alternatives and are joined with `OR`. If one classifier is not in the table, the texts of all the classifiers are kept as the declared text and no id is named, so the licence is `unknown` |
| Maven (`maven2`) | the `<licenses>` of the POM: the `<name>`, or the `<url>` when there is no name, joined with `OR` | If the POM names no licence, its `<parent>` is read from the same repository, up to five parent levels. Only stored POMs are read. POMs larger than 16 MiB are not read. The declared text is cut at 1 KiB |
| NuGet (`nuget`) | 1. `licenseExpression` of the nuspec; 2. `licenseUrl`; 3. a licence file, declared as the text `license file` | A `licenses.nuget.org/` URL gives the expression in it. The deprecation URL gives nothing. Other URLs are kept as text and read through the alias table, so an OSI URL can map to its id. A licence file has no id, so it is `unknown` |
| Cargo (`cargo`) | `license` of the crate's manifest | Hosted crates only. A proxied crate gets no licence |
| RubyGems (`rubygems`) | `licenses` of the gem's specification, joined with `OR` | |

Every other format, including Go (`go`) and Docker (`docker`), records no licence. Their components have
state `none` (see [states](#states-of-a-declared-licence)). The licence of a component is stored once per
component and is replaced when the component is scanned again.

## How a declared text is read

The server reads the declared text as an SPDX licence expression. The embedded list is the SPDX licence list,
including its deprecated ids and the equivalent id of each deprecated one.

- **Ids** are matched without regard to case and come back in their canonical spelling: `mit` is stored as
  `MIT`, and `apache-2.0` as `Apache-2.0`.
- **Operators**: `AND`, `OR` and `WITH` (without regard to case), parentheses, and the `+` suffix, which
  means "or later" when the SPDX list has a non-deprecated or-later id for the licence. `GPL-2.0+` has none,
  because `GPL-2.0` is deprecated, so it stays as written and is flagged deprecated. A `/` between two
  licences is read as `OR`, the form older Cargo crates use.
- **Length**: an expression longer than 1024 bytes is not read and is `unknown`.
- **`LicenseRef-*`** ids (and `DocumentRef-…:LicenseRef-…`) are kept as written. They are judged like any
  other id, so they must be named in a policy list to be allowed or denied.
- **Aliases and URLs**: a table of common spellings maps to an id, for example `Apache 2.0`,
  `Apache License, Version 2.0`, `GPLv3` (to `GPL-3.0-only`), `GPLv2+` (to `GPL-2.0-or-later`),
  `MPL 2.0`, `CC0`, and the OSI and FSF URLs of the common licences.
- **Deprecated ids** stay as written, so the stored text shows what the package declared. They are flagged
  deprecated. When the SPDX list names an equivalent for one, a policy list that names the equivalent also
  matches it, and the reverse.

Some names are deliberately not mapped, because a name without a version does not say which licence it is.
`BSD`, `BSD License`, `GPL`, `LGPL`, `AGPL`, `CDDL`, `GNU General Public License`, `Public Domain`,
`Dual License`, `Proprietary`, `Commercial` and `Other` stay `unknown`. Write the version or the id.

### States of a declared licence

| State | Meaning |
|---|---|
| `spdx` | Every id is an SPDX id or a `LicenseRef-*` id, and the text parsed as it is |
| `mapped` | The text was changed to reach an id: an alias, a URL, a `+` turned into an or-later id, or a `/` read as `OR` |
| `unknown` | The text is present but cannot be read, or it parses and names an id the list does not know |
| `none` | The component declares no licence, or its format records none |

## The policy block `licenses`

```json
{
  "mode": "report",
  "list": "deny",
  "ids": ["GPL-3.0-only", "AGPL-3.0-only"],
  "unknown": "report"
}
```

| Field | Values | Default | Meaning |
|---|---|---|---|
| `mode` | `off`, `report`, `block` | `off` | `off` judges nothing. `report` computes the verdict and marks what would be blocked, but serves the download. `block` refuses it |
| `list` | `allow`, `deny` | `deny` | `allow`: only the listed licences may be used. `deny`: the listed licences may not be used |
| `ids` | up to 200 ids, each at most 100 characters | empty | Each must be a single id: an SPDX id (current or deprecated) or a `LicenseRef-*` id. Expressions (`MIT OR ISC`), names (`MIT License`) and aliases (`GPLv3`) are refused |
| `unknown` | `allow`, `report`, `block` | `report` | What to do with a component whose licence is `unknown` or `none` and that the list does not judge |

A policy with `mode: block`, `list: allow` and no ids is refused when it is saved, because it would block
every component.

The strictest mode of the rules of a policy decides what the gate does. If the vulnerability rule is in
`block` and the licence rule in `report`, downloads are refused for vulnerabilities and licences are only
reported.

## How a component is judged

Judgement follows these steps. The list is applied first, and only to a licence that parses. The `unknown`
setting is applied only when the list does not decide.

1. **A licence that parses and is `spdx` or `mapped`.**
   - With an **allow** list, the component passes when some `OR` branch of the expression has every `AND`
     term on the list. Otherwise it breaks the rule with the breach "not on the allow list".
   - With a **deny** list, the component breaks the rule when every `OR` branch holds a denied id.
     `MIT OR GPL-3.0-only` passes against a deny list of `GPL-3.0-only`, because the MIT branch is clean.
     `MIT AND GPL-3.0-only` breaks it.
2. **A licence that parses and is `unknown`**: the list decides only when it passes (allow list satisfied) or
   fails (deny list violated). In any other case the `unknown` setting decides.
3. **No parsable licence** (`none`, or a text that does not parse): the `unknown` setting decides.

The `unknown` setting gives `allow` (the component passes), `report` (it is reported) or `block` (it is
blocked). A component with no declared licence is reported with the breach "undeclared".

Other rules of the matching:

- A `WITH` exception is ignored for matching. `GPL-2.0-only WITH Classpath-exception-2.0` matches
  `GPL-2.0-only`.
- An empty deny list denies nothing.
- An empty allow list in `report` mode reports every licence that is not on it. It is refused in `block`
  mode (see above).

## Exceptions

A licence exception is a policy exception with the rule `license`. Create one with
`POST /service/rest/v1/security/exceptions`; it needs the rights `astrum:policyexceptions:create`, so security
staff can grant exceptions without editing policies. A licence exception cannot name an advisory.

| Field | Meaning |
|---|---|
| `rule` | `license` |
| `license` | A licence id, such as `GPL-3.0-only`. A deprecated id matches its equivalent |
| `policy` | The policy the exception applies to. Omitted, it applies to every policy |
| `name`, `namespace`, `version`, `format`, `digest` | Restrict the exception to one component: `name` names it, and the other fields narrow it further. `digest` limits it to one exact artifact |
| `reason` | Why the exception exists, at most 1000 characters |
| `expiresAt` | Optional. An expired exception stops applying |

An exception must name at least one of `license`, `name` or `digest`.

- **By licence id** (`license` only): the id counts as acceptable for every component that declares it. The
  expression is judged again with that id on the list (added to an allow list, removed from a deny list). The
  exception is shown in the verdict only when it changes the outcome.
- **By component** (`name`, optionally with `license`): the component is excused from the rule. With
  `license` too, only that licence of the component is excused.

When several exceptions apply, the one that lives longest is used for each licence id.

## The gate

Package downloads of a repository whose policy has a licence rule are checked, through the same gate as
vulnerabilities. A refused download gets `403` with the policy name and the reason. The reasons name the
licence:

- `license MIT AND GPL-3.0-only: on the deny list`
- `license ISC: not on the allow list`
- `license LicenseRef-Own: not recognised`
- `no license is declared`

The refusal text starts with `blocked by policy <name>:`. Refused downloads are written to the audit log.

In `report` mode the download is served. A component that would be blocked is marked, and it appears as
"reported" in the report below.

A component's verdict is stored by the scan and kept in the policy verdict table. A change to a component's
licence marks the component for judgement again (table `policy_dirty`), and the verdict is recomputed in the
background. Until that runs, the gate uses the verdict it has. A component without a stored verdict is judged
when it is downloaded.

## The licence report

Everyone who may browse a repository can read its licence records. The report is a list over the components
you can see, with the declared text, the expression, the ids, the state and the verdict of the repository's
policy. A verdict is one of `pass`, `blocked`, `reported` or `unjudged`.

| Method and path | Use |
|---|---|
| `GET /service/rest/v1/security/licenses` | One page of 100 components. Parameters: `repository`, `format`, `license` (an id), `state` (`spdx`, `mapped`, `unknown` or `none`) and `continuationToken` from the previous page |
| `GET /service/rest/v1/security/licenses/export.csv` | The whole list as CSV, with the same parameters except `continuationToken` |

Without `state`, the list holds only components that have a licence record. Use `state=none` to see components
with no licence, including those of formats that record none.

The JSON page holds `items` and `continuationToken`. Each item has `repository`, `format`, `namespace`, `name`,
`version`, `declared`, `expression`, `ids`, `state` and `verdict`.

The CSV has the header `repository,format,namespace,name,version,declared,expression,state,verdict`. Fields
that begin with `=`, `+`, `-`, `@`, a tab or a carriage return are prefixed with `'`, so a spreadsheet does not
run them as formulas. A query that matches more than 500 000 components is refused with `400`: narrow it with
`repository`, `format`, `license` or `state` and export again.

The MCP tool `list_license_findings` returns the same page for agents, with the parameters `repository`,
`license`, `state`, `format` and `continuationToken` (see [agents over MCP](../use/mcp.md)).

The web page of the report is `/admin/security/licenses`. It needs the administrator right `astrum:*`.

## Rolling out a licence rule

1. Create the policy with `mode: report` and `unknown: report`, and attach it to the repository. Attaching a
   policy queues every component of the repository for a scan, and the scan records the licences.
2. Open the licence report and look at the `unknown` and `none` rows. Write exceptions or fix the list
   where the declared texts are not what you expect.
3. Only then set `unknown: block`. A component with no licence record is judged as undeclared, so with
   `unknown: block` every component of a format that records no licence (Go, Docker and the others) is
   blocked. Do not set it on a repository that holds such formats.
4. Switch `mode` to `block` when the report shows what you want refused.

## Limits

- Only the six formats listed above record a licence. The rule is not applied to other formats, and it cannot be
  applied to Docker images.
- The licence is read from stored metadata. A proxied Cargo crate has no licence, and a component is not
  fetched again to find one.
- For Cargo, RubyGems, NuGet and Maven the declared text is cut at 1 KiB before it is stored and judged.
- The rule does not read licence files, so a package that declares only a licence file is `unknown`.
- Judgement uses the SPDX list the server embeds. A licence id newer than that list is `unknown` until the
  server is updated.
