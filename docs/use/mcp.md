# Agents over MCP

The server speaks the Model Context Protocol at `/mcp`, so an AI agent such as Claude Code, Claude
Desktop or VS Code can search artifacts, read small files, look at blob stores and run tasks without a
browser. The agent works with the rights of the user whose token it presents: it sees the repositories
that user sees and is refused what that user is refused, and its changes are in the audit log under that
user.

## Connect

Create an [access token](../configure/authentication.md#access-tokens) under **Account menu → Access
Tokens**. A token with the scope **Read** gives the agent only the reading tools; **Write** adds the
tools that change the server.

Claude Code:

```sh
claude mcp add --transport http astrum https://astrum.example.org/mcp \
  --header "Authorization: Bearer astrum_..."
```

Any client that speaks the Streamable HTTP transport connects the same way: `POST` requests to
`https://<server>/mcp` with the header `Authorization: Bearer <token>`. The server is stateless: it gives
no session id and keeps no event stream, so `GET /mcp` answers `405`. It speaks the protocol versions
`2025-06-18` and `2025-03-26`, and refuses requests whose `Origin` is not the server's own host.

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

The tools that change the server are only offered to Write tokens, and every tool is only listed when the
user holds the rights it needs; the server checks those rights again on every call. Nothing deletes
components, repositories, blob stores or users, and nothing touches security settings.

The server also offers two prompts: `diagnose_space`, which walks the agent through finding where the
space went and what to do about it, and `find_artifact`, which finds a package and how to use its latest
version.
