use std::cmp::Ordering;

use astrum_mcp::{
    Annotations, Handler, Prompt, PromptArgument, Tool, ToolError, required_text, schema,
    text_argument,
};
use astrum_security::{Scope, Subject};
use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde_json::{Map, Value, json};

use super::loopback::{Loopback, refusal};

const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');
const TEXT_DEFAULT: usize = 64 * 1024;
const TEXT_MAX: usize = 256 * 1024;
const VERSION_PAGES: usize = 6;
const TAG_PAGES: usize = 10;

#[derive(Clone, Copy)]
enum Need {
    SignedIn,
    Permissions(&'static [&'static str]),
}

struct Def {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    schema: fn() -> Value,
    need: Need,
    write: bool,
    idempotent: bool,
}

fn string(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

fn defs() -> Vec<Def> {
    vec![
        Def {
            name: "get_status",
            title: "Server status",
            description: "Whether the server reads and writes, who you are signed in as, and blob stores that are unavailable or past their soft quota.",
            schema: || schema(json!({}), &[]),
            need: Need::SignedIn,
            write: false,
            idempotent: true,
        },
        Def {
            name: "list_repositories",
            title: "List repositories",
            description: "The repositories you can see, with format, type (hosted, proxy, group), URL and size in bytes.",
            schema: || {
                schema(
                    json!({
                        "format": string("Only this format, such as maven2, npm, docker, helm, pypi, raw"),
                        "type": string("Only hosted, proxy or group"),
                        "name": string("Only names that contain this text"),
                    }),
                    &[],
                )
            },
            need: Need::SignedIn,
            write: false,
            idempotent: true,
        },
        Def {
            name: "get_repository",
            title: "Repository settings",
            description: "The settings of one repository: storage, proxy remote, group members, cleanup and format settings.",
            schema: || schema(json!({ "name": string("Repository name") }), &["name"]),
            need: Need::SignedIn,
            write: false,
            idempotent: true,
        },
        Def {
            name: "search_components",
            title: "Search components",
            description: "Search packages, artifacts and images. Each result has its assets with download URLs and checksums. Pass continuationToken from a previous answer for the next page.",
            schema: || {
                schema(
                    json!({
                        "q": string("Words to look for in names, versions and paths"),
                        "repository": string("Repository name"),
                        "format": string("Format, such as maven2, npm, docker"),
                        "group": string("Group: Maven groupId, npm scope without @"),
                        "name": string("Name, * matches any text"),
                        "version": string("Version, * matches any text"),
                        "sort": string("name, version, group or repository"),
                        "direction": string("asc or desc"),
                        "continuationToken": string("Token of the next page"),
                    }),
                    &[],
                )
            },
            need: Need::SignedIn,
            write: false,
            idempotent: true,
        },
        Def {
            name: "list_versions",
            title: "Versions of a component",
            description: "Every version of one package, artifact or image, newest first, and the latest one.",
            schema: || {
                schema(
                    json!({
                        "name": string("Component name, such as phoenix-ui or my-app"),
                        "group": string("Group: Maven groupId, npm scope without @"),
                        "repository": string("Repository name"),
                        "format": string("Format"),
                    }),
                    &["name"],
                )
            },
            need: Need::SignedIn,
            write: false,
            idempotent: true,
        },
        Def {
            name: "search_assets",
            title: "Search assets",
            description: "Search single files (assets) with their download URL, size, checksums and last download. Pass continuationToken for the next page.",
            schema: || {
                schema(
                    json!({
                        "q": string("Words to look for in paths and names"),
                        "repository": string("Repository name"),
                        "format": string("Format"),
                        "group": string("Group"),
                        "name": string("Component name"),
                        "version": string("Component version"),
                        "sha256": string("SHA-256 of the file"),
                        "sha1": string("SHA-1 of the file"),
                        "continuationToken": string("Token of the next page"),
                    }),
                    &[],
                )
            },
            need: Need::SignedIn,
            write: false,
            idempotent: true,
        },
        Def {
            name: "read_text_asset",
            title: "Read a text file",
            description: "Read a small text file of a repository, such as a pom.xml, package.json or Chart.yaml. Binary files are refused; long files are cut at maxBytes.",
            schema: || {
                schema(
                    json!({
                        "repository": string("Repository name"),
                        "path": string("Path of the file in the repository, as search_assets shows it"),
                        "maxBytes": { "type": "integer", "description": "At most this many bytes, up to 262144; 65536 by default" },
                    }),
                    &["repository", "path"],
                )
            },
            need: Need::SignedIn,
            write: false,
            idempotent: true,
        },
        Def {
            name: "list_docker_tags",
            title: "Docker tags of an image",
            description: "The tags of one Docker image in a repository, newest first, with size and the reference to pull.",
            schema: || {
                schema(
                    json!({
                        "repository": string("Docker repository name, such as docker-prod"),
                        "image": string("Image name, such as ownlate/api"),
                    }),
                    &["repository", "image"],
                )
            },
            need: Need::SignedIn,
            write: false,
            idempotent: true,
        },
        Def {
            name: "list_blob_stores",
            title: "Blob stores and space",
            description: "Every blob store with its type, blob count, size used, free space, soft quota state, the repositories that use it and the group it belongs to.",
            schema: || schema(json!({}), &[]),
            need: Need::Permissions(&["astrum:blobstores:read"]),
            write: false,
            idempotent: true,
        },
        Def {
            name: "list_tasks",
            title: "List tasks",
            description: "Scheduled and manual tasks with their state, last run and next run.",
            schema: || {
                schema(
                    json!({ "type": string("Only tasks of this type id, such as blobstore.compact") }),
                    &[],
                )
            },
            need: Need::Permissions(&["astrum:tasks:read"]),
            write: false,
            idempotent: true,
        },
        Def {
            name: "get_task",
            title: "Task details",
            description: "One task with its settings, schedule, state and last result.",
            schema: || schema(json!({ "id": string("Task id") }), &["id"]),
            need: Need::Permissions(&["astrum:tasks:read"]),
            write: false,
            idempotent: true,
        },
        Def {
            name: "run_task",
            title: "Run a task",
            description: "Start a task now, such as a compaction or a cleanup.",
            schema: || schema(json!({ "id": string("Task id") }), &["id"]),
            need: Need::Permissions(&["astrum:tasks:start"]),
            write: true,
            idempotent: false,
        },
        Def {
            name: "stop_task",
            title: "Stop a task",
            description: "Stop a task that is running.",
            schema: || schema(json!({ "id": string("Task id") }), &["id"]),
            need: Need::Permissions(&["astrum:tasks:stop"]),
            write: true,
            idempotent: true,
        },
        Def {
            name: "create_remote_blob_store",
            title: "Create a remote blob store",
            description: "Add a blob store kept by the blob agent on another server. The server checks that the agent answers before it saves the store. Add it to a group to give a store more room.",
            schema: || {
                schema(
                    json!({
                        "name": string("Name of the new store"),
                        "url": string("Address of the agent, such as https://storage1.internal:8090"),
                        "token": string("Token the agent was started with, at least 32 characters"),
                        "certificate": string("PEM certificate of the agent or its CA, when it is not publicly trusted"),
                    }),
                    &["name", "url", "token"],
                )
            },
            need: Need::Permissions(&["astrum:blobstores:create"]),
            write: true,
            idempotent: false,
        },
        Def {
            name: "convert_blob_store_to_group",
            title: "Convert a blob store to a group",
            description: "Turn a blob store into a group of the same name with the store, renamed, as its only member. Repositories keep their store; add members to the group afterwards to give it room.",
            schema: || {
                schema(
                    json!({
                        "name": string("Store to convert, such as default"),
                        "newNameForOriginal": string("New name of the current store, such as default-disk"),
                    }),
                    &["name", "newNameForOriginal"],
                )
            },
            need: Need::Permissions(&["astrum:blobstores:create", "astrum:blobstores:update"]),
            write: true,
            idempotent: false,
        },
        Def {
            name: "update_group_blob_store",
            title: "Change a group blob store",
            description: "Set the members or the fill policy of a group blob store. A member that holds blobs cannot be removed.",
            schema: || {
                schema(
                    json!({
                        "name": string("Group name"),
                        "members": { "type": "array", "items": { "type": "string" }, "description": "All members in order; omit to keep them" },
                        "fillPolicy": string("writeToFirst, roundRobin or mostFreeSpace; omit to keep it"),
                    }),
                    &["name"],
                )
            },
            need: Need::Permissions(&["astrum:blobstores:update"]),
            write: true,
            idempotent: true,
        },
        Def {
            name: "set_repository_online",
            title: "Take a repository online or offline",
            description: "An offline repository refuses downloads and uploads until it is online again.",
            schema: || {
                schema(
                    json!({
                        "name": string("Repository name"),
                        "online": { "type": "boolean", "description": "true for online, false for offline" },
                    }),
                    &["name", "online"],
                )
            },
            need: Need::SignedIn,
            write: true,
            idempotent: true,
        },
        Def {
            name: "list_users",
            title: "List users",
            description: "Users with their source, status and roles. query matches any part of the user id, name or email.",
            schema: || {
                schema(
                    json!({
                        "query": string("Part of the user id, name or email"),
                        "source": string("User source: default for local users (the default), or LDAP"),
                    }),
                    &[],
                )
            },
            need: Need::Permissions(&["astrum:users:read"]),
            write: false,
            idempotent: true,
        },
        Def {
            name: "list_roles",
            title: "List roles",
            description: "Roles with the privileges and roles they hold. query matches any part of the id, name or description; privilege keeps the roles that hold that privilege.",
            schema: || {
                schema(
                    json!({
                        "query": string("Part of the role id, name or description"),
                        "privilege": string("Name of a privilege the role holds, such as as-repository-view-npm-npm-hosted-read"),
                    }),
                    &[],
                )
            },
            need: Need::Permissions(&["astrum:roles:read"]),
            write: false,
            idempotent: true,
        },
        Def {
            name: "list_privileges",
            title: "List privileges",
            description: "Privileges with their type, format, repository and actions. Every repository has view privileges named as-repository-view-<format>-<repository>-<action> for browse, read, edit, add and delete, and one for all actions (*); format-wide ones use * as the repository.",
            schema: || {
                schema(
                    json!({
                        "repository": string("Repository the privileges are about"),
                        "format": string("Format id, such as npm or protobuf"),
                        "query": string("Part of the privilege name or description"),
                    }),
                    &[],
                )
            },
            need: Need::Permissions(&["astrum:privileges:read"]),
            write: false,
            idempotent: true,
        },
        Def {
            name: "create_role",
            title: "Create a role",
            description: "Create a role that holds privileges and other roles. Give it to users with set_user_roles.",
            schema: || {
                schema(
                    json!({
                        "id": string("Role id"),
                        "name": string("Name shown in the interface; the id when omitted"),
                        "description": string("What the role is for"),
                        "privileges": { "type": "array", "items": { "type": "string" }, "description": "Privilege names" },
                        "roles": { "type": "array", "items": { "type": "string" }, "description": "Ids of roles it includes" },
                    }),
                    &["id"],
                )
            },
            need: Need::Permissions(&["astrum:roles:create"]),
            write: true,
            idempotent: false,
        },
        Def {
            name: "update_role",
            title: "Change a role",
            description: "Add privileges and included roles to a role or take them away, or change its name and description. Built-in roles cannot be changed.",
            schema: || {
                schema(
                    json!({
                        "id": string("Role id"),
                        "addPrivileges": { "type": "array", "items": { "type": "string" }, "description": "Privilege names to add" },
                        "removePrivileges": { "type": "array", "items": { "type": "string" }, "description": "Privilege names to take away" },
                        "addRoles": { "type": "array", "items": { "type": "string" }, "description": "Role ids to include" },
                        "removeRoles": { "type": "array", "items": { "type": "string" }, "description": "Role ids to stop including" },
                        "name": string("New name"),
                        "description": string("New description"),
                    }),
                    &["id"],
                )
            },
            need: Need::Permissions(&["astrum:roles:update"]),
            write: true,
            idempotent: true,
        },
        Def {
            name: "set_user_roles",
            title: "Give or take roles of a user",
            description: "Add roles to a user or take them away. Giving a role needs the right to grant it; the other details of the user stay as they are.",
            schema: || {
                schema(
                    json!({
                        "userId": string("User id"),
                        "source": string("User source: default for local users (the default), or LDAP"),
                        "add": { "type": "array", "items": { "type": "string" }, "description": "Role ids to give" },
                        "remove": { "type": "array", "items": { "type": "string" }, "description": "Role ids to take away" },
                    }),
                    &["userId"],
                )
            },
            need: Need::Permissions(&["astrum:users:update"]),
            write: true,
            idempotent: true,
        },
    ]
}

pub struct AstrumTools {
    loopback: Loopback,
    subject: Subject,
}

impl AstrumTools {
    pub fn new(loopback: Loopback, subject: Subject) -> Self {
        Self { loopback, subject }
    }

    fn allowed(&self, def: &Def) -> bool {
        if def.write && self.subject.scope == Scope::Read {
            return false;
        }
        match def.need {
            Need::SignedIn => true,
            Need::Permissions(permissions) => permissions
                .iter()
                .all(|permission| self.subject.is_permitted(permission)),
        }
    }
}

#[async_trait]
impl Handler for AstrumTools {
    fn tools(&self) -> Vec<Tool> {
        defs()
            .into_iter()
            .filter(|def| self.allowed(def))
            .map(|def| Tool {
                name: def.name.to_owned(),
                title: def.title.to_owned(),
                description: def.description.to_owned(),
                input_schema: (def.schema)(),
                annotations: Annotations {
                    title: def.title.to_owned(),
                    read_only_hint: !def.write,
                    destructive_hint: false,
                    idempotent_hint: def.idempotent,
                    open_world_hint: false,
                },
            })
            .collect()
    }

    async fn call(&self, name: &str, arguments: Map<String, Value>) -> Result<Value, ToolError> {
        let api = &self.loopback;
        match name {
            "get_status" => status(api, &self.subject).await,
            "list_repositories" => list_repositories(api, &arguments).await,
            "get_repository" => get_repository(api, &required_text(&arguments, "name")?).await,
            "search_components" => search_components(api, &arguments).await,
            "list_versions" => list_versions(api, &arguments).await,
            "search_assets" => search_assets(api, &arguments).await,
            "read_text_asset" => read_text_asset(api, &arguments).await,
            "list_docker_tags" => list_docker_tags(api, &arguments).await,
            "list_blob_stores" => list_blob_stores(api).await,
            "list_tasks" => list_tasks(api, &arguments).await,
            "get_task" => {
                let id = required_text(&arguments, "id")?;
                api.get(&format!("/service/rest/v1/tasks/{}", segment(&id)))
                    .await
            }
            "run_task" | "stop_task" => {
                let id = required_text(&arguments, "id")?;
                let action = match name {
                    "run_task" => "run",
                    _ => "stop",
                };
                api.json(
                    Method::POST,
                    &format!("/service/rest/v1/tasks/{}/{action}", segment(&id)),
                    None,
                )
                .await?;
                Ok(json!({ "id": id, "done": action }))
            }
            "create_remote_blob_store" => create_remote_blob_store(api, &arguments).await,
            "convert_blob_store_to_group" => {
                let store = required_text(&arguments, "name")?;
                let member = required_text(&arguments, "newNameForOriginal")?;
                api.json(
                    Method::POST,
                    &format!(
                        "/service/rest/v1/blobstores/group/convert/{}/{}",
                        segment(&store),
                        segment(&member)
                    ),
                    None,
                )
                .await?;
                api.get(&format!(
                    "/service/rest/v1/blobstores/group/{}",
                    segment(&store)
                ))
                .await
            }
            "update_group_blob_store" => update_group(api, &arguments).await,
            "set_repository_online" => set_online(api, &arguments).await,
            "list_users" => list_users(api, &arguments).await,
            "list_roles" => list_roles(api, &arguments).await,
            "list_privileges" => list_privileges(api, &arguments).await,
            "create_role" => create_role(api, &arguments).await,
            "update_role" => update_role(api, &arguments).await,
            "set_user_roles" => set_user_roles(api, &arguments).await,
            other => Err(ToolError::Unknown(other.to_owned())),
        }
    }

    fn prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt {
                name: "diagnose_space".to_owned(),
                title: "Find out where the space went".to_owned(),
                description: "Look at the blob stores and the biggest repositories and suggest how to free or add space".to_owned(),
                arguments: Vec::new(),
            },
            Prompt {
                name: "find_artifact".to_owned(),
                title: "Find an artifact".to_owned(),
                description: "Find a package, artifact or image and how to use its latest version".to_owned(),
                arguments: vec![PromptArgument {
                    name: "name".to_owned(),
                    description: "What to look for, such as phoenix-ui or ownlate/api".to_owned(),
                    required: true,
                }],
            },
        ]
    }

    fn prompt(&self, name: &str, arguments: &Map<String, Value>) -> Option<(String, String)> {
        match name {
            "diagnose_space" => Some((
                "Find out where the space of Astrum Repository went".to_owned(),
                "Find out where the space of this Astrum Repository went and what to do about it.\n\
                 1. Call list_blob_stores: note stores that are unavailable, past a soft quota or low on free space.\n\
                 2. Call list_repositories and rank the repositories by size; name the five biggest and their stores.\n\
                 3. For big Docker repositories call list_docker_tags on their largest images and count old tags.\n\
                 4. Call list_tasks and check that cleanup, Docker cleanup and compaction tasks exist and ran recently.\n\
                 5. Suggest, in this order: deleting what cleanup policies already allow, running the compaction, and adding room \
                 by converting the full store into a group and adding a remote blob store.\n\
                 Change nothing yourself; end with the exact tool calls you would make."
                    .to_owned(),
            )),
            "find_artifact" => {
                let wanted = arguments
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                Some((
                    format!("Find {wanted} in Astrum Repository"),
                    format!(
                        "Find \"{wanted}\" in this Astrum Repository.\n\
                         1. Call search_components with q set to it; if nothing comes back, try parts of the name.\n\
                         2. For the best match call list_versions and name the latest version and the repository.\n\
                         3. Show how to use it: the Maven, npm, pip or helm coordinates, or the docker pull reference, \
                         and the download URL of its main file."
                    ),
                ))
            }
            _ => None,
        }
    }
}

fn settings_uri(summary: &Value, name: &str) -> Option<String> {
    let format = summary["format"].as_str()?;
    let kind = summary["type"].as_str()?;
    let api_name = astrum_repo::format::FORMATS
        .iter()
        .find(|known| known.id == format)
        .map_or(format, |known| known.api_name);
    Some(format!(
        "/service/rest/v1/repositories/{}/{}/{}",
        segment(api_name),
        segment(kind),
        segment(name)
    ))
}

fn segment(text: &str) -> String {
    utf8_percent_encode(text, SEGMENT).to_string()
}

fn encoded_path(path: &str) -> String {
    path.trim_start_matches('/')
        .split('/')
        .map(segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn query(pairs: &[(&str, String)]) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(key, value);
    }
    serializer.finish()
}

fn picked(arguments: &Map<String, Value>, keys: &[&'static str]) -> Vec<(&'static str, String)> {
    keys.iter()
        .filter_map(|key| text_argument(arguments, key).map(|value| (*key, value)))
        .collect()
}

async fn status(api: &Loopback, subject: &Subject) -> Result<Value, ToolError> {
    let readable = api
        .fetch(Method::GET, crate::STATUS, None, 1024)
        .await?
        .status
        == StatusCode::OK;
    let writable = api
        .fetch(Method::GET, crate::STATUS_WRITABLE, None, 1024)
        .await?
        .status
        == StatusCode::OK;
    let mut status = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "readable": readable,
        "writable": writable,
        "user": subject.user_id,
        "tokenScope": match subject.scope {
            Scope::Read => "read",
            Scope::Write => "write",
        },
    });
    if subject.is_permitted("astrum:blobstores:read")
        && let Ok(Value::Array(stores)) = api.get("/service/rest/internal/ui/blobstores").await
    {
        let attention: Vec<Value> = stores
            .iter()
            .filter(|store| {
                store["unavailable"] == true || store["quotaStatus"]["violation"] == true
            })
            .map(|store| {
                json!({
                    "name": store["name"],
                    "unavailable": store["unavailable"],
                    "quota": store["quotaStatus"]["message"],
                })
            })
            .collect();
        status["blobStoresNeedingAttention"] = Value::Array(attention);
    }
    Ok(status)
}

async fn list_repositories(
    api: &Loopback,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let format = text_argument(arguments, "format");
    let kind = text_argument(arguments, "type");
    let name = text_argument(arguments, "name").map(|name| name.to_lowercase());
    let Value::Array(repositories) = api.get("/service/rest/v1/repositories").await? else {
        return Err(ToolError::failed("the repository list was not a list"));
    };
    let kept: Vec<Value> = repositories
        .into_iter()
        .filter(|repository| {
            format
                .as_deref()
                .is_none_or(|format| repository["format"].as_str() == Some(format))
                && kind
                    .as_deref()
                    .is_none_or(|kind| repository["type"].as_str() == Some(kind))
                && name.as_deref().is_none_or(|name| {
                    repository["name"]
                        .as_str()
                        .is_some_and(|found| found.to_lowercase().contains(name))
                })
        })
        .collect();
    Ok(json!({ "count": kept.len(), "repositories": kept }))
}

async fn get_repository(api: &Loopback, name: &str) -> Result<Value, ToolError> {
    let summary = api
        .get(&format!("/service/rest/v1/repositories/{}", segment(name)))
        .await?;
    let Some(uri) = settings_uri(&summary, name) else {
        return Ok(summary);
    };
    match api.get(&uri).await {
        Ok(settings) => Ok(settings),
        Err(_) => Ok(summary),
    }
}

fn trimmed_asset(asset: &Value) -> Value {
    let mut kept = json!({
        "path": asset["path"],
        "downloadUrl": asset["downloadUrl"],
        "contentType": asset["contentType"],
        "fileSize": asset["fileSize"],
        "lastModified": asset["lastModified"],
        "lastDownloaded": asset["lastDownloaded"],
        "downloadCount": asset["downloadCount"],
        "sha256": asset["checksum"]["sha256"],
        "sha1": asset["checksum"]["sha1"],
        "blobStore": asset["blobStoreName"],
    });
    if !asset["registryUrl"].is_null() {
        kept["registryUrl"] = asset["registryUrl"].clone();
    }
    kept
}

fn trimmed_component(component: &Value) -> Value {
    json!({
        "id": component["id"],
        "repository": component["repository"],
        "format": component["format"],
        "group": component["group"],
        "name": component["name"],
        "version": component["version"],
        "assets": component["assets"]
            .as_array()
            .map(|assets| assets.iter().map(trimmed_asset).collect::<Vec<_>>())
            .unwrap_or_default(),
    })
}

fn page(found: &Value, items: Vec<Value>) -> Value {
    json!({
        "count": items.len(),
        "items": items,
        "continuationToken": found["continuationToken"],
    })
}

async fn search_components(
    api: &Loopback,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let pairs = picked(
        arguments,
        &[
            "q",
            "repository",
            "format",
            "group",
            "name",
            "version",
            "sort",
            "direction",
            "continuationToken",
        ],
    );
    let found = api
        .get(&format!("/service/rest/v1/search?{}", query(&pairs)))
        .await?;
    let items = found["items"]
        .as_array()
        .map(|items| items.iter().map(trimmed_component).collect())
        .unwrap_or_default();
    Ok(page(&found, items))
}

async fn search_assets(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let pairs = picked(
        arguments,
        &[
            "q",
            "repository",
            "format",
            "group",
            "name",
            "version",
            "sha256",
            "sha1",
            "continuationToken",
        ],
    );
    let found = api
        .get(&format!("/service/rest/v1/search/assets?{}", query(&pairs)))
        .await?;
    let items = found["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|asset| {
                    let mut kept = trimmed_asset(asset);
                    kept["repository"] = asset["repository"].clone();
                    kept["format"] = asset["format"].clone();
                    kept
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(page(&found, items))
}

async fn components(
    api: &Loopback,
    mut pairs: Vec<(&'static str, String)>,
    pages: usize,
) -> Result<(Vec<Value>, bool), ToolError> {
    let mut all = Vec::new();
    for _ in 0..pages {
        let found = api
            .get(&format!("/service/rest/v1/search?{}", query(&pairs)))
            .await?;
        if let Some(items) = found["items"].as_array() {
            all.extend(items.iter().cloned());
        }
        match found["continuationToken"].as_str() {
            Some(token) => {
                pairs.retain(|(key, _)| *key != "continuationToken");
                pairs.push(("continuationToken", token.to_owned()));
            }
            None => return Ok((all, false)),
        }
    }
    Ok((all, true))
}

async fn list_versions(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = required_text(arguments, "name")?;
    let mut pairs = picked(arguments, &["group", "repository", "format"]);
    pairs.push(("name", name.clone()));
    let (found, more) = components(api, pairs, VERSION_PAGES).await?;
    let mut versions: Vec<Value> = found
        .iter()
        .map(|component| {
            json!({
                "version": component["version"],
                "repository": component["repository"],
                "format": component["format"],
                "group": component["group"],
                "lastModified": newest(component),
            })
        })
        .collect();
    versions.sort_by(|left, right| {
        compare_versions(
            left["version"].as_str().unwrap_or_default(),
            right["version"].as_str().unwrap_or_default(),
        )
        .reverse()
    });
    let latest = versions
        .iter()
        .find(|version| {
            version["version"]
                .as_str()
                .is_some_and(|text| !prerelease(text))
        })
        .or(versions.first())
        .map(|version| version["version"].clone())
        .unwrap_or(Value::Null);
    Ok(json!({
        "name": name,
        "latest": latest,
        "count": versions.len(),
        "incomplete": more,
        "versions": versions,
    }))
}

fn newest(component: &Value) -> Value {
    component["assets"]
        .as_array()
        .and_then(|assets| {
            assets
                .iter()
                .filter_map(|asset| asset["lastModified"].as_str())
                .max()
        })
        .map(|time| Value::String(time.to_owned()))
        .unwrap_or(Value::Null)
}

fn prerelease(version: &str) -> bool {
    let lower = version.to_lowercase();
    lower.contains('-')
        || ["snapshot", "alpha", "beta", "rc", "dev", "pre"]
            .iter()
            .any(|word| lower.contains(word))
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let parts = |text: &str| -> Vec<(bool, String)> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut digits = false;
        for character in text.chars() {
            let is_digit = character.is_ascii_digit();
            if !character.is_alphanumeric() {
                if !current.is_empty() {
                    parts.push((digits, std::mem::take(&mut current)));
                }
                continue;
            }
            if !current.is_empty() && is_digit != digits {
                parts.push((digits, std::mem::take(&mut current)));
            }
            digits = is_digit;
            current.push(character);
        }
        if !current.is_empty() {
            parts.push((digits, current));
        }
        parts
    };
    let (left, right) = (parts(left), parts(right));
    for (a, b) in left.iter().zip(right.iter()) {
        let order = match (a.0, b.0) {
            (true, true) => {
                let (x, y) = (a.1.trim_start_matches('0'), b.1.trim_start_matches('0'));
                x.len().cmp(&y.len()).then_with(|| x.cmp(y))
            }
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => a.1.to_lowercase().cmp(&b.1.to_lowercase()),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    match (left.len().cmp(&right.len()), left.len() > right.len()) {
        (Ordering::Equal, _) => Ordering::Equal,
        (_, true) => match left[right.len()].0 {
            true => Ordering::Greater,
            false => Ordering::Less,
        },
        (_, false) => match right[left.len()].0 {
            true => Ordering::Less,
            false => Ordering::Greater,
        },
    }
}

async fn read_text_asset(
    api: &Loopback,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let repository = required_text(arguments, "repository")?;
    let path = required_text(arguments, "path")?;
    let limit = arguments
        .get("maxBytes")
        .and_then(Value::as_u64)
        .map_or(TEXT_DEFAULT, |limit| (limit as usize).clamp(1, TEXT_MAX));
    let uri = format!(
        "/repository/{}/{}",
        segment(&repository),
        encoded_path(&path)
    );
    let fetched = api.fetch(Method::GET, &uri, None, limit).await?;
    if !fetched.status.is_success() {
        return Err(refusal(&fetched));
    }
    let content_type = fetched
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let size = fetched
        .headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    let mut body = fetched.body;
    if fetched.truncated {
        while !body.is_empty() && std::str::from_utf8(&body).is_err() {
            body.pop();
        }
    }
    let binary = body.contains(&0);
    let text = match (binary, String::from_utf8(body)) {
        (false, Ok(text)) => text,
        _ => {
            return Err(ToolError::failed(format!(
                "{path} is not a text file ({content_type}); download it from its URL instead"
            )));
        }
    };
    Ok(json!({
        "repository": repository,
        "path": path,
        "contentType": content_type,
        "size": size,
        "truncated": fetched.truncated,
        "text": text,
    }))
}

async fn list_docker_tags(
    api: &Loopback,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let repository = required_text(arguments, "repository")?;
    let image = required_text(arguments, "image")?;
    let pairs = vec![
        ("format", "docker".to_owned()),
        ("repository", repository.clone()),
        ("name", image.clone()),
    ];
    let (found, more) = components(api, pairs, TAG_PAGES).await?;
    let mut tags: Vec<Value> = found
        .iter()
        .filter(|component| component["name"].as_str() == Some(image.as_str()))
        .map(|component| {
            let size: u64 = component["assets"]
                .as_array()
                .map(|assets| {
                    assets
                        .iter()
                        .filter_map(|asset| asset["fileSize"].as_u64())
                        .sum()
                })
                .unwrap_or(0);
            let registry = component["assets"]
                .as_array()
                .and_then(|assets| {
                    assets
                        .iter()
                        .find_map(|asset| asset["registryUrl"].as_str())
                })
                .map(str::to_owned);
            let tag = component["version"].as_str().unwrap_or_default();
            json!({
                "tag": tag,
                "lastModified": newest(component),
                "manifestBytes": size,
                "pull": registry.map(|registry| format!("{registry}/{image}:{tag}")),
            })
        })
        .collect();
    tags.sort_by(|left, right| {
        right["lastModified"]
            .as_str()
            .cmp(&left["lastModified"].as_str())
    });
    Ok(json!({
        "repository": repository,
        "image": image,
        "count": tags.len(),
        "incomplete": more,
        "tags": tags,
    }))
}

async fn list_blob_stores(api: &Loopback) -> Result<Value, ToolError> {
    let Value::Array(stores) = api.get("/service/rest/internal/ui/blobstores").await? else {
        return Err(ToolError::failed("the blob store list was not a list"));
    };
    let stores: Vec<Value> = stores
        .iter()
        .map(|store| {
            let mut kept = json!({
                "name": store["name"],
                "type": store["type"],
                "unavailable": store["unavailable"],
                "blobCount": store["blobCount"],
                "usedBytes": store["totalSize"],
                "availableBytes": store["availableSpace"],
                "softQuota": store["softQuota"],
                "quotaViolation": store["quotaStatus"]["violation"],
                "repositories": store["repositories"],
                "repositoryCount": store["repositoryCount"],
                "groups": store["groups"],
            });
            for key in [
                "path",
                "bucket",
                "region",
                "prefix",
                "url",
                "members",
                "fillPolicy",
            ] {
                if !store[key].is_null() {
                    kept[key] = store[key].clone();
                }
            }
            kept
        })
        .collect();
    Ok(json!({ "count": stores.len(), "blobStores": stores }))
}

async fn list_tasks(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let pairs = picked(arguments, &["type"]);
    let uri = match pairs.is_empty() {
        true => "/service/rest/v1/tasks".to_owned(),
        false => format!("/service/rest/v1/tasks?{}", query(&pairs)),
    };
    api.get(&uri).await
}

async fn create_remote_blob_store(
    api: &Loopback,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let name = required_text(arguments, "name")?;
    let body = json!({
        "name": name,
        "url": required_text(arguments, "url")?,
        "token": required_text(arguments, "token")?,
        "certificate": text_argument(arguments, "certificate"),
    });
    api.json(
        Method::POST,
        "/service/rest/v1/blobstores/remote",
        Some(&body),
    )
    .await?;
    api.get(&format!(
        "/service/rest/v1/blobstores/remote/{}",
        segment(&name)
    ))
    .await
}

async fn update_group(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = required_text(arguments, "name")?;
    let uri = format!("/service/rest/v1/blobstores/group/{}", segment(&name));
    let mut group = api.get(&uri).await?;
    match arguments.get("members") {
        None | Some(Value::Null) => {}
        Some(Value::Array(members)) if members.iter().all(Value::is_string) => {
            group["members"] = Value::Array(members.clone());
        }
        Some(_) => return Err(ToolError::invalid("members must be a list of store names")),
    }
    if let Some(policy) = text_argument(arguments, "fillPolicy") {
        group["fillPolicy"] = Value::String(policy);
    }
    api.json(Method::PUT, &uri, Some(&group)).await?;
    api.get(&uri).await
}

async fn set_online(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = required_text(arguments, "name")?;
    let Some(online) = arguments.get("online").and_then(Value::as_bool) else {
        return Err(ToolError::invalid(
            "the argument online must be true or false",
        ));
    };
    let summary = api
        .get(&format!("/service/rest/v1/repositories/{}", segment(&name)))
        .await?;
    let Some(uri) = settings_uri(&summary, &name) else {
        return Err(ToolError::failed(format!(
            "the repository {name} has no format or type"
        )));
    };
    let mut settings = api.get(&uri).await?;
    settings["online"] = Value::Bool(online);
    api.json(Method::PUT, &uri, Some(&settings)).await?;
    let saved = api.get(&uri).await?;
    Ok(json!({ "name": name, "online": saved["online"] }))
}

const USERS: &str = "/service/rest/v1/security/users";
const ROLES: &str = "/service/rest/v1/security/roles";
const PRIVILEGES: &str = "/service/rest/v1/security/privileges";

fn list_argument(arguments: &Map<String, Value>, key: &str) -> Result<Vec<String>, ToolError> {
    match arguments.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| ToolError::invalid(format!("{key} must be a list of names")))
            })
            .collect(),
        Some(_) => Err(ToolError::invalid(format!("{key} must be a list of names"))),
    }
}

fn names_of(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn changed(current: Vec<String>, add: &[String], remove: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = current
        .into_iter()
        .filter(|name| !remove.contains(name))
        .collect();
    for name in add {
        if !kept.contains(name) {
            kept.push(name.clone());
        }
    }
    kept
}

fn mentions(fields: &[&Value], wanted: &str) -> bool {
    let wanted = wanted.to_lowercase();
    fields.iter().any(|field| {
        field
            .as_str()
            .is_some_and(|text| text.to_lowercase().contains(&wanted))
    })
}

fn source_of(arguments: &Map<String, Value>) -> String {
    text_argument(arguments, "source").unwrap_or_else(|| "default".to_owned())
}

fn user_view(user: &Value) -> Value {
    json!({
        "userId": user["userId"],
        "firstName": user["firstName"],
        "lastName": user["lastName"],
        "emailAddress": user["emailAddress"],
        "source": user["source"],
        "status": user["status"],
        "roles": user["roles"],
    })
}

fn role_view(role: &Value) -> Value {
    json!({
        "id": role["id"],
        "name": role["name"],
        "description": role["description"],
        "readOnly": role["readOnly"],
        "privileges": role["privileges"],
        "roles": role["roles"],
    })
}

async fn list_users(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let uri = format!("{USERS}?{}", query(&[("source", source_of(arguments))]));
    let Value::Array(users) = api.get(&uri).await? else {
        return Err(ToolError::failed("the user list was not a list"));
    };
    let wanted = text_argument(arguments, "query");
    let users: Vec<Value> = users
        .iter()
        .filter(|user| {
            wanted.as_deref().is_none_or(|wanted| {
                mentions(
                    &[
                        &user["userId"],
                        &user["firstName"],
                        &user["lastName"],
                        &user["emailAddress"],
                    ],
                    wanted,
                )
            })
        })
        .map(user_view)
        .collect();
    Ok(json!({ "users": users }))
}

async fn list_roles(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let Value::Array(roles) = api.get(ROLES).await? else {
        return Err(ToolError::failed("the role list was not a list"));
    };
    let wanted = text_argument(arguments, "query");
    let privilege = text_argument(arguments, "privilege");
    let roles: Vec<Value> = roles
        .iter()
        .filter(|role| {
            wanted.as_deref().is_none_or(|wanted| {
                mentions(&[&role["id"], &role["name"], &role["description"]], wanted)
            })
        })
        .filter(|role| {
            privilege.as_deref().is_none_or(|privilege| {
                names_of(&role["privileges"])
                    .iter()
                    .any(|held| held == privilege)
            })
        })
        .map(role_view)
        .collect();
    Ok(json!({ "roles": roles }))
}

async fn list_privileges(
    api: &Loopback,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let Value::Array(privileges) = api.get(PRIVILEGES).await? else {
        return Err(ToolError::failed("the privilege list was not a list"));
    };
    let repository = text_argument(arguments, "repository");
    let format = text_argument(arguments, "format");
    let wanted = text_argument(arguments, "query");
    let privileges: Vec<Value> = privileges
        .iter()
        .filter(|privilege| {
            repository
                .as_deref()
                .is_none_or(|repository| privilege["repository"].as_str() == Some(repository))
        })
        .filter(|privilege| {
            format
                .as_deref()
                .is_none_or(|format| privilege["format"].as_str() == Some(format))
        })
        .filter(|privilege| {
            wanted.as_deref().is_none_or(|wanted| {
                mentions(&[&privilege["name"], &privilege["description"]], wanted)
            })
        })
        .map(|privilege| {
            json!({
                "name": privilege["name"],
                "type": privilege["type"],
                "format": privilege["format"],
                "repository": privilege["repository"],
                "actions": privilege["actions"],
                "description": privilege["description"],
            })
        })
        .collect();
    Ok(json!({ "privileges": privileges }))
}

async fn create_role(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let id = required_text(arguments, "id")?;
    let body = json!({
        "id": id,
        "name": text_argument(arguments, "name").unwrap_or_else(|| id.clone()),
        "description": text_argument(arguments, "description").unwrap_or_default(),
        "privileges": list_argument(arguments, "privileges")?,
        "roles": list_argument(arguments, "roles")?,
    });
    api.json(Method::POST, ROLES, Some(&body)).await?;
    let saved = api.get(&format!("{ROLES}/{}", segment(&id))).await?;
    Ok(role_view(&saved))
}

async fn update_role(api: &Loopback, arguments: &Map<String, Value>) -> Result<Value, ToolError> {
    let id = required_text(arguments, "id")?;
    let uri = format!("{ROLES}/{}", segment(&id));
    let role = api.get(&uri).await?;
    if role["readOnly"].as_bool() == Some(true) {
        return Err(ToolError::invalid(format!(
            "{id} is a built-in role and cannot be changed"
        )));
    }
    let privileges = changed(
        names_of(&role["privileges"]),
        &list_argument(arguments, "addPrivileges")?,
        &list_argument(arguments, "removePrivileges")?,
    );
    let roles = changed(
        names_of(&role["roles"]),
        &list_argument(arguments, "addRoles")?,
        &list_argument(arguments, "removeRoles")?,
    );
    let body = json!({
        "id": id,
        "name": text_argument(arguments, "name").map_or_else(|| role["name"].clone(), Value::String),
        "description": text_argument(arguments, "description")
            .map_or_else(|| role["description"].clone(), Value::String),
        "privileges": privileges,
        "roles": roles,
    });
    api.json(Method::PUT, &uri, Some(&body)).await?;
    Ok(role_view(&api.get(&uri).await?))
}

async fn set_user_roles(
    api: &Loopback,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let user_id = required_text(arguments, "userId")?;
    let source = source_of(arguments);
    let listed = format!(
        "{USERS}?{}",
        query(&[("userId", user_id.clone()), ("source", source.clone())])
    );
    let find = |users: Value| -> Option<Value> {
        users
            .as_array()?
            .iter()
            .find(|user| user["userId"].as_str() == Some(user_id.as_str()))
            .cloned()
    };
    let Some(user) = find(api.get(&listed).await?) else {
        return Err(ToolError::failed(format!(
            "there is no user {user_id} in the source {source}"
        )));
    };
    let roles = changed(
        names_of(&user["roles"]),
        &list_argument(arguments, "add")?,
        &list_argument(arguments, "remove")?,
    );
    let body = json!({
        "userId": user["userId"],
        "firstName": user["firstName"],
        "lastName": user["lastName"],
        "emailAddress": user["emailAddress"],
        "source": user["source"],
        "status": user["status"],
        "roles": roles,
    });
    api.json(
        Method::PUT,
        &format!("{USERS}/{}", segment(&user_id)),
        Some(&body),
    )
    .await?;
    let Some(saved) = find(api.get(&listed).await?) else {
        return Err(ToolError::failed(format!("the user {user_id} is gone")));
    };
    Ok(user_view(&saved))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_sort_by_their_numbers() {
        let mut versions = vec![
            "1.10.0",
            "1.2.0",
            "1.9.3",
            "2.0.0-rc.1",
            "2.0.0",
            "1.2",
            "0.9",
        ];
        versions.sort_by(|left, right| compare_versions(left, right).reverse());
        assert_eq!(
            versions,
            [
                "2.0.0",
                "2.0.0-rc.1",
                "1.10.0",
                "1.9.3",
                "1.2.0",
                "1.2",
                "0.9"
            ]
        );
        assert!(prerelease("2.0.0-rc.1"));
        assert!(prerelease("1.0-SNAPSHOT"));
        assert!(!prerelease("20261019031946"));
        assert_eq!(compare_versions("v1.0", "V1.0"), Ordering::Equal);
    }

    #[test]
    fn paths_and_names_are_encoded_once() {
        assert_eq!(segment("a b/c"), "a%20b%2Fc");
        assert_eq!(encoded_path("/org/x y/1.0/x.pom"), "org/x%20y/1.0/x.pom");
        assert_eq!(
            query(&[("q", "a&b".to_owned()), ("name", "x y".to_owned())]),
            "q=a%26b&name=x+y"
        );
    }

    #[test]
    fn lists_change_by_adding_and_taking_away() {
        let current = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
        assert_eq!(
            changed(
                current,
                &["d".to_owned(), "a".to_owned()],
                &["b".to_owned()]
            ),
            ["a", "c", "d"]
        );
        assert!(mentions(&[&json!("ci-globalart")], "GLOBAL"));
        assert!(!mentions(&[&Value::Null], "x"));
    }

    #[test]
    fn read_tokens_see_no_writing_tools() {
        let writes: Vec<&str> = defs()
            .iter()
            .filter(|def| def.write)
            .map(|def| def.name)
            .collect();
        assert_eq!(
            writes,
            [
                "run_task",
                "stop_task",
                "create_remote_blob_store",
                "convert_blob_store_to_group",
                "update_group_blob_store",
                "set_repository_online",
                "create_role",
                "update_role",
                "set_user_roles"
            ]
        );
    }
}
