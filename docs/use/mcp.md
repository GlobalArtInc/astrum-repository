# Agents over MCP

The server speaks the Model Context Protocol at `/mcp`, so an AI agent such as Claude Code, Claude
Desktop or VS Code can search artifacts, read small files, look at blob stores and run tasks without a
browser. The agent works with the rights of the user whose token it presents: it sees the repositories
that user sees and is refused what that user is refused, and its changes are in the audit log under that
user.

## Connect

### With OAuth

Clients that sign in through OAuth (Claude on the web and Claude Desktop as a custom connector, Claude
Code, VS Code) need nothing but the address `https://<server>/mcp`. The client finds the rest itself:
`/mcp` answers `401` with a link to `/.well-known/oauth-protected-resource/mcp` (RFC 9728), which names
the server as its own authorization server (`/.well-known/oauth-authorization-server`, RFC 8414). The
client registers itself (RFC 7591), and the browser opens the server's sign-in and then the page
**Connect an Agent**, where you allow or deny it. Allowing gives the agent the reading tools; tick
**Allow changes as well** to add the tools that change the server.

Claude Code:

```sh
claude mcp add --transport http astrum https://astrum.example.org/mcp
```

Then run `/mcp` in Claude Code and choose **Authenticate**.

The server issues its own tokens, bound to `/mcp`: they are refused everywhere else on the server.
Access tokens last an hour and the client renews them with a refresh token that lasts 30 days and is
replaced on every use. Only public clients with PKCE (`S256`) are accepted. Redirects must go to
`https` addresses, to `http` on the client's own computer (`127.0.0.1`, `[::1]`, `localhost`, any port)
or to an app's own scheme.

Every connected agent is listed under **Account menu → Access Tokens → Connected Agents**, where
**Disconnect** cuts it off at once. Deleting a user disconnects their agents too.

### With an access token

Create an [access token](../configure/authentication.md#access-tokens) under **Account menu → Access
Tokens**. A token with the scope **Read** gives the agent only the reading tools; **Write** adds the
tools that change the server.

```sh
claude mcp add --transport http astrum https://astrum.example.org/mcp \
  --header "Authorization: Bearer astrum_..."
```

### Transport

Any client that speaks the Streamable HTTP transport connects the same way: `POST` requests to
`https://<server>/mcp` with `Authorization: Bearer <token>`. The server is stateless: it gives no session
id and keeps no event stream, so `GET /mcp` answers `405`. It speaks the protocol versions `2025-11-25`,
`2025-06-18` and `2025-03-26`, and refuses requests whose `Origin` is not the server's own host.

### Limits and audit

An agent may make 120 calls at once and 4 a second after that, counted per user; above that `/mcp`
answers `429` with `Retry-After`. Client registration is limited to 10 an hour per address, and the
other OAuth endpoints to 30 at once and one every two seconds after that. Changes made through MCP are
in the audit log under the user, with `via: mcp` and the name of the client in the event's attributes;
connecting and disconnecting an agent are audited as `oauth` events. An upload leaves two
`repository.upload` events: `ticket-issued` when the address is handed out and `uploaded` when the file
is stored. Presenting an upload address is limited to 30 attempts at once and one a second after that,
counted per client address.

## Tools

| Tool | What it does | Needs |
|---|---|---|
| `get_status` | Whether the server reads and writes, the signed-in user, blob stores that are unavailable or past a soft quota | a token |
| `list_repositories`, `get_repository` | Repositories with format, type, URL and size; the settings of one | a token |
| `search_components`, `search_assets` | Search, 50 results a page, with download URLs and checksums | a token |
| `list_versions` | Every version of a component, newest first, and the latest release | a token |
| `read_text_asset` | Reads a text file such as a `pom.xml` or `Chart.yaml`, up to 256 KiB; refuses binary files | a token |
| `list_docker_tags` | Tags of an image, newest first, with the reference to pull | a token |
| `list_blob_stores` | Blob stores with blobs, space used and free, quota state, repositories and groups | `astrum:blobstores:read` |
| `list_tasks`, `get_task` | Tasks with state, last and next run | `astrum:tasks:read` |
| `run_task`, `stop_task` | Starts or stops a task | `astrum:tasks:start`, `astrum:tasks:stop` |
| `create_remote_blob_store` | Adds a [remote blob store](../configure/blob-stores.md#remote-blob-stores) | `astrum:blobstores:create` |
| `convert_blob_store_to_group` | Turns a store into a group of the same name | `astrum:blobstores:create` and `update` |
| `update_group_blob_store` | Sets the members or the fill policy of a group | `astrum:blobstores:update` |
| `set_repository_online` | Takes a repository offline or back online | edit rights on the repository |
| `request_upload` | Gives a one-time `PUT` address for a file; the agent then sends the file with `curl` ([Upload a package](#upload-a-package)) | the right to add to some repository |
| `upload_component` | Uploads a file of up to 64 KiB given inline as base64 | the right to add to some repository |
| `list_users`, `list_roles`, `list_privileges` | Users with their roles; roles with their privileges and the roles they include; privileges, filtered by repository, format or name | `astrum:users:read`, `astrum:roles:read`, `astrum:privileges:read` |
| `create_role`, `update_role` | Creates a role, or adds and takes away its privileges and included roles; built-in roles stay as they are | `astrum:roles:create`, `astrum:roles:update` |
| `set_user_roles` | Gives roles to a user or takes them away, keeping the other details of the user | `astrum:users:update`, and the right to grant the roles |

The tools that change the server are only offered to Write tokens, and every tool is only listed when the
user holds the rights it needs; the server checks those rights again on every call. The only tools that
add components are the two upload tools; nothing deletes components, repositories, blob stores or users,
and nothing touches security settings.

The server also offers two prompts: `diagnose_space`, which walks the agent through finding where the
space went and what to do about it, and `find_artifact`, which finds a package and how to use its latest
version.

## Upload a package

A package must not travel through the conversation: a 600 KB tarball in base64 would cost about 800 KB
of model tokens. So the agent asks for an address and sends the file with `curl` from the machine where
the file is.

1. The agent calls `request_upload` with the repository, the name of the file and, when the format needs
   them, the fields of its upload form. The tool is offered to Write tokens of users who may add to some
   repository, and it refuses a repository the user may not add to.
2. The answer holds `uploadUrl`, a `PUT` address that works once and for five minutes, and the `curl`
   command to run.
3. The agent runs the command. The server stores the body through the code of the REST upload
   (`POST /service/rest/v1/components`), so validation, the write policy and the rights of the user are
   the ones of REST. The answer is `201` with the repository, format, file name and size.
4. The agent checks the result with `list_versions` or `search_components`.

`request_upload` takes:

| Argument | |
|---|---|
| `repository` | A hosted repository whose format takes uploads. |
| `fileName` | The name of the file, without a path: not empty, at most 255 bytes, no `/`, `\` or control characters, not `.` or `..`. For a raw repository it is the name in the repository, unless `fields` sets `filename`. |
| `fields` | The fields of the upload form of the format, as in [Upload a component](rest-api.md#upload-a-component) but without the file: `directory` or `raw.directory`, `groupId`, `extension`. `GET /service/rest/v1/formats/<format>/upload-specs` lists them. |
| `sizeBytes` | The size of the file, when known. A larger body is refused. Up to 512 MiB, which is also the limit when the argument is left out. |

An npm tarball needs no fields: the name and the version come from the `package.json` inside it. When the
upload form of the format asks for fields that were not set, the answer lists them in `fieldsNotSet` and
still issues the address, because some formats read them from the file (Maven reads the coordinates from
a POM); if the upload is then refused, the agent asks for a new address with the fields.

```text
request_upload {"repository": "npm-hosted", "fileName": "phoenix-ui-1.4.0.tgz", "sizeBytes": 612345}
```

```json
{
  "repository": "npm-hosted",
  "format": "npm",
  "method": "PUT",
  "uploadUrl": "https://astrum.example.org/mcp/upload/astrum_up_...",
  "curl": "curl -sS -w '\\nHTTP %{http_code}\\n' -T <path-to-file> 'https://astrum.example.org/mcp/upload/astrum_up_...'",
  "expiresInSeconds": 300,
  "maxBytes": 612345,
  "singleUse": true
}
```

```sh
curl -sS -w '\nHTTP %{http_code}\n' -T phoenix-ui-1.4.0.tgz 'https://astrum.example.org/mcp/upload/astrum_up_...'
```

```text
{"repository":"npm-hosted","format":"npm","fileName":"phoenix-ui-1.4.0.tgz","size":612345}
HTTP 201
```

Any other status comes with a JSON `message` that says why.

```text
list_versions {"name": "phoenix-ui", "group": "globalart", "repository": "npm-hosted"}
```

A raw file and a Maven jar differ only in `fields`:

```text
request_upload {"repository": "raw-hosted", "fileName": "tool-1.4.0.tar.gz", "fields": {"directory": "/tools"}}
request_upload {"repository": "maven-releases", "fileName": "my-library-1.0.0.jar",
  "fields": {"groupId": "com.example", "artifactId": "my-library", "version": "1.0.0",
             "extension": "jar", "generate-pom": "true"}}
```

An address is a secret: whoever holds it can store the one file it was issued for, as the user who asked
for it, so the agent keeps it out of logs and shell history it shares. What binds it:

- It names the user, the repository, the form fields and the file field when it is issued, and the
  address has nothing else to choose, so it cannot be turned to another repository or path.
- It is used once. The first request that presents it spends it, whether the upload then succeeds or not;
  after a refusal the agent asks for a new one. A body larger than `maxBytes` is cut off and refused with
  `413`.
- It expires after five minutes. An unknown, expired and used address all answer `404`.
- The rights of the user are read again when the file arrives, so a role taken away in the meantime
  stops the upload with `403`. The write policy of the repository applies as for any upload: with
  "Disable Redeploy" the second upload of the same file is refused.
- A user may hold about 20 unused addresses at a time.
- Disconnecting the agent, deleting its token or deleting the user does not cancel an address that was
  already issued; it stops working within five minutes, and at once for a deleted user.
- The address is part of the URL, so a reverse proxy or gateway that logs request paths logs it too. That
  does no harm once it is used or expired, but do not copy a fresh one into places other people read.

A file of up to 64 KiB can go inline instead, with `upload_component` and the same arguments plus
`contentBase64`. It is meant for small text files; use `request_upload` for anything else.
