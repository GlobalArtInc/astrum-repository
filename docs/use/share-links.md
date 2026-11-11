# Share links

A share link gives someone without an account a secret address to download one file, or every file under a
folder, of a hosted repository. The link works until it expires or until its download limit is used up, and
you can revoke it at any time.

Links work on hosted repositories of every format except Docker: a Docker repository's manifests and blobs are
of no use at a plain address. Proxy and group repositories take no links, because their files can be evicted
or come from several members.

## Create a link

Open a file in the browser of a repository and press **Share link**; in a folder press **Share folder**. The
button is shown only to users who hold the right to create share links, and only for hosted repositories that
are not Docker.

The dialog takes:

- **Lifetime**: 1 hour, 1 day, 7 days or a custom number of whole hours. The server refuses a lifetime longer
  than its limit, the setting `shareLinks.maxDays` (30 days by default). A limit of 0 switches links off.
- **Download limit** (files only): the total size the link may serve, counted in copies of the file. Leave it
  empty for no limit.
- **Note**: up to 500 characters, shown in the list of links. Control characters and text-direction characters
  are refused.

The address appears once, in the dialog, with a copy button. For a file, the dialog also shows a ready
`curl -fLO` line. The server keeps only a hash of the secret, so it cannot show the address again: if it is
lost, revoke the link and create another.

The same is possible over REST. `POST /service/rest/v1/share-links` takes `repository`, `path` and optionally
`prefix`, `expiresInHours`, `maxDownloads` and `note`, and answers `201` with the link and its `url`. The
answer carries `Cache-Control: no-store`. The list (`GET /service/rest/v1/share-links`) and the revoke call
(`DELETE /service/rest/v1/share-links/{id}`) never return the address. Agents can do the same with the tools
`create_share_link`, `list_share_links` and `revoke_share_link` ([Agents over MCP](mcp.md#tools)).

A path that ends with a slash makes a folder link. A folder link has no download limit.

## Address

A file link opens `https://<host>/service/share/<secret>/<file name>`. A folder link opens
`https://<host>/service/share/<secret>/` and shows a page that lists up to 1000 files of the folder, each with a
link that uses the same secret.

The host is the domain of the repository when the repository has its own [domain](../configure/reverse-proxy.md#a-domain-of-its-own-for-a-repository).
Otherwise it is the address the creating request was sent to, as the server sees it behind its reverse proxy.

## Who may do what

| Action | Needs |
|---|---|
| Create a link to one file | `astrum:sharelinks:create` and read access to that file |
| Create a folder link | `astrum:sharelinks:create` and read access to the whole repository |
| List your own links | being signed in |
| List everyone's links | `astrum:sharelinks:read` |
| Revoke your own links | being signed in |
| Revoke someone else's link | `astrum:sharelinks:delete` |

A token with the READ scope can neither create nor revoke links. The rights are checked when a link is
created: after that the link works until it expires, even if its creator loses the right. Deleting a user
deletes that user's links, and deleting a repository deletes its links. A user may hold at most 1000 active
links.

## Expiry, downloads and revoking

Once a link has expired, its address answers `410 Gone`.

The download limit is a budget of bytes: a link may serve at most as many bytes as the file's size times the
limit. A download that starts at the beginning of the file counts as one download in the list. A request for a
later part of the file, which a client sends when it resumes, uses up only the bytes it sends. When the budget
is spent, the address answers `410 Gone`.

To revoke a link, open **Settings → Share links** and revoke it there, or call the REST route above. The
address stops working at once and answers `404 Not Found`.

## What the address answers

| Status | When |
|---|---|
| `200`, `206` | the file, a part of it, or the folder page |
| `404` | the address is unknown or revoked, the path is missing, or the repository is offline, proxy, group or Docker |
| `410` | the link has expired or its download budget is spent |
| `405` | any method other than `GET` and `HEAD`; the answer lists `GET, HEAD` in `Allow` |

Files are sent as attachments. Every answer carries `Cache-Control: private, no-store`, `Referrer-Policy:
no-referrer`, `X-Content-Type-Options: nosniff`, `Content-Security-Policy: sandbox` and `X-Robots-Tag: noindex`,
so a shared file is not kept by browsers or search engines and does not run as a page.

## Logs and audit

Creating and revoking a link are audited in the domain `security.sharelink`, as `created` and `deleted`. The
audit entry names the link, its repository, path and lifetime. It never holds the address, the secret's hash
or the note.

The server's own request log writes the share address as `/service/share/<hidden>`. A reverse proxy in front
of the server writes the path as the client sent it, so it must mask the secret too:
see [Share link addresses in access logs](../configure/reverse-proxy.md#share-link-addresses-in-access-logs).
