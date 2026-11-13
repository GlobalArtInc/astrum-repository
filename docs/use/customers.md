# Customers and their download tokens

A customer is a named recipient of artifacts, for example a company you ship builds to. A customer holds
tokens, and each token reads only the repositories that were chosen for it when it was issued. Tokens
expire, and the server counts what each customer and each token downloads. Customer tokens read only;
the repositories are chosen at issuance and cannot be widened later.

The customer list is the Customers page at `/admin/customers` in the web interface. The examples below
use `astrum.example.com`; use your own host.

## Privileges

- `astrum:customers:read` lists customers, their tokens and their usage.
- `astrum:customers:create` creates customers and issues tokens.
- `astrum:customers:update` changes a customer, including disabling it.
- `astrum:customers:delete` deletes a customer and revokes its tokens.

Create, update and delete include read. Issuing a token needs `astrum:customers:create` and read access
to every listed repository and to every member of a listed group, that is
`astrum:repository-view:<format>:<repository>:read`. A token that is itself limited to repositories
cannot issue tokens.

## Creating a customer

A customer has an id, a name, a reference and a note. The id is a slug of lower-case letters, digits and
hyphens, 1 to 64 characters. The reference is free text for an outside number, such as a contract or a
ticket. A customer can be disabled: a disabled customer's tokens stop working at once, and it cannot get
new tokens until it is enabled again.

The customer id and a user id cannot be the same. Creating a customer is refused when a user has its id,
and creating a user is refused when a customer has its id.

## Issuing a token

Open the customer and choose **Issue Token**, or send a `POST` to
`/service/rest/v1/customers/<customer>/tokens` with `name`, `owner`, `repositories` and `expiresInDays`.

- **Name**: 1 to 200 bytes.
- **Owner**: an enabled local user. The token has the owner's rights, narrowed to read on the listed
  repositories, so it never has more than its owner. Give each offering its own service user with read
  rights only on the repositories it needs.
- **Repositories**: at least one and at most 100 existing repository names, each once. Wildcards are not
  allowed.
- **Expiry**: required. The presets are 30, 90, 365 and 730 days; a custom number of days can be given
  from 1 to 3650.

The token is shown once, when it is issued. The server keeps only its hash, so a lost token has to be
revoked and replaced. The dialog then shows a ready line for each listed repository:

- Docker: `docker login -u <customer> <host>`, with the token as the password.
- Helm: `helm repo add <repository> https://<host>/repository/<repository>/ --username <customer> --password <token>`.
- npm: `//<host>/repository/<repository>/:_authToken=<token>`.
- Every other format: `curl -u <customer>:<token> -fLO https://<host>/repository/<repository>/<path>`.

The customer id is the user name in every line. The owner's id is accepted as the user name too.

Limits and rules:

- A customer holds at most 20 unexpired tokens. Expired tokens do not count, and revoked tokens are
  deleted, so they do not count either.
- Customer tokens do not count toward the owner's limit of 100 personal tokens, and they do not appear
  on the owner's Access Tokens page.
- A disabled customer cannot get a token.

## Groups in a token

A group in a customer's list gives access to everything in its members, now and after later changes. A
member added to the group later is readable with the token that already exists, and a member removed is
no longer readable. The web interface warns about this when you choose a group. This is the intended
behaviour. Nested groups are checked the same way, up to ten levels deep.

## Renaming and deleting a repository

Repositories cannot be renamed; the server refuses a rename. Deleting a repository removes its name from
every token that lists it, customer tokens and personal tokens alike, in the same transaction as the
delete. A token that is left with no repository is deleted. Each change is in the audit log: a deleted
token is recorded with the reason `repository-deleted`, and each customer whose tokens changed gets a
`tokens-updated` record with the number of tokens narrowed and deleted.

## Usage statistics

Usage is counted per customer and per token.

- The customer list shows, for the last 30 days, the downloads and the bytes, and the time of the last
  access.
- The customer page shows the usage per repository for 30, 90 or 365 days. The REST endpoint
  `/service/rest/v1/customers/<customer>/usage?days=` accepts 1 to 730 days, and the default is 30.
- The token list shows the time and the address of the last use of each token. The time has a
  granularity of 60 seconds.
- Downloads and bytes per token are in the CSV export, not in the token list.

What counts:

- A download is a request for the whole file. A ranged request that resumes a download adds bytes but
  no download.
- Bytes are the declared length of the response, so a download that is aborted counts in full.
- Each format decides which of its files count as downloads. For example, APT counts `.deb` files,
  Alpine counts `.apk` files and Go counts module zip files.
- Docker counts objects, not pulls. A single `docker pull` counts the manifest, the config and every
  layer it fetches.
- For a group, the usage repository is the member that served the file, not the group.

The CSV export is `GET /service/rest/v1/customers/usage.csv`, with the optional parameters `days`
(1 to 730, default 30) and `customer`. It needs `astrum:customers:read`. The header is
`customer,token,tokenHint,repository,day,downloads,bytes`. A cell that starts with `=`, `+`, `-`, `@`,
a tab or a carriage return gets a leading apostrophe, so a spreadsheet does not run it as a formula.
An export of more than 200000 rows is refused with a message to narrow the period or to choose one
customer; it is not cut short.

## Docker

Log in with the customer id as the user name and the token as the password:

```sh
docker login -u <customer> <host>
```

After the login, the token itself is used as the bearer credential for pulls.

## Expiry notices

The server can tell notification channels when a customer token is about to expire and when it has
expired. Nothing is sent to customers.

- **Setting**: the capability type `customers.expiryNotice`, stored under the setting key
  `customers.expiryNotice`. It is managed like the other capabilities, see [Capabilities](../configure/capabilities.md).
  Its fields are `days`, up to five day counts from 1 to 365 separated by commas (default 30, 7 and 1),
  and `channels`, the names of the notification channels that get the notices, separated by commas.
  With no channel set, no notices are sent.
- **Task**: a daily task `customers.tokens.expiry` runs at 08:00 UTC. The server creates it once, on the
  first start after this feature arrives, and it does not create it again if you delete it.
- **Notices**: a token that will expire within the largest day count gets one notice for the smallest
  count that covers it, so a token 29 days from expiry gets the 30-day notice, and a token 6 days from
  expiry gets the 7-day notice. A token that expired in the last seven days gets one "expired" notice.
  A notice is sent once per token and threshold, so reruns do not repeat it. Tokens of disabled
  customers are not announced.
- **Link**: when a base URL is set, the notice ends with a link to `<base>/admin/customers`.

## Operating notes

- **Rolling back**: a server older than this feature cannot work with the customer tables and the
  narrowed tokens. Before you start such a server, run the down migration of the customers schema. It
  deletes every customer token and every personal token that is limited to repositories, drops the
  customers and their usage history, and cannot be undone. Take a backup first, see
  [Upgrade](../operate/upgrade.md).
- **Audit**: the initiator of an action made with a customer token is the owner of the token, not the
  customer.
- **MCP**: no MCP tool issues tokens. The customer tools only list customers, read usage and revoke
  tokens; see [Agents over MCP](mcp.md).
