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
connecting and disconnecting an agent are audited as `oauth` events.

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
| `list_users`, `list_roles`, `list_privileges` | Users with their roles; roles with their privileges and the roles they include; privileges, filtered by repository, format or name | `astrum:users:read`, `astrum:roles:read`, `astrum:privileges:read` |
| `create_role`, `update_role` | Creates a role, or adds and takes away its privileges and included roles; built-in roles stay as they are | `astrum:roles:create`, `astrum:roles:update` |
| `set_user_roles` | Gives roles to a user or takes them away, keeping the other details of the user | `astrum:users:update`, and the right to grant the roles |

The tools that change the server are only offered to Write tokens, and every tool is only listed when the
user holds the rights it needs; the server checks those rights again on every call. Nothing deletes
components, repositories, blob stores or users, and nothing touches security settings.

The server also offers two prompts: `diagnose_space`, which walks the agent through finding where the
space went and what to do about it, and `find_artifact`, which finds a package and how to use its latest
version.
