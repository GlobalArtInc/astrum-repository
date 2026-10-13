mod support;

use std::collections::BTreeSet;

use axum::http::{Method, StatusCode};
use serde_json::{Map, Value, json};
use support::{Auth, Fixture};

const REFERENCE: &str = include_str!("data/openapi-reference.json");
const SERVED: &str = include_str!("../src/routes/support/openapi.json");
const METHODS: [&str; 5] = ["get", "put", "post", "delete", "head"];

fn words(segment: &str) -> String {
    match segment {
        "blobstores" => "blob stores".to_owned(),
        "ssl" => "SSL".to_owned(),
        "ldap" => "LDAP".to_owned(),
        "http" => "HTTP".to_owned(),
        "s3" => "S3".to_owned(),
        "supportzip" => "support ZIP".to_owned(),
        "supportzippath" => "support ZIP file".to_owned(),
        "ssrf-protection" => "SSRF protection".to_owned(),
        "gcp" => "Google Cloud".to_owned(),
        other => other.replace('-', " "),
    }
}

fn summary(method: &str, path: &str) -> String {
    let parts: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .skip(1)
        .collect();
    let ends_with_param = parts.last().is_some_and(|part| part.starts_with('{'));
    let fixed: Vec<String> = parts
        .iter()
        .filter(|part| !part.starts_with('{'))
        .map(|part| words(part))
        .collect();
    let action_after_param = !ends_with_param
        && parts.len() >= 2
        && parts[parts.len() - 2].starts_with('{')
        && method == "post";
    if action_after_param {
        let (action, resource) = fixed.split_last().expect("an action has a resource");
        let mut action = action.clone();
        action[..1].make_ascii_uppercase();
        return format!("{action} for {}", resource.join(" "));
    }
    let resource = fixed.join(" ");
    let verb = match method {
        "get" if ends_with_param => "Get",
        "get" => "List",
        "put" => "Update",
        "delete" => "Delete",
        "head" => "Check",
        _ => "Create",
    };
    format!("{verb} {resource}")
}

fn operation_id(method: &str, path: &str) -> String {
    let mut id = match method {
        "get" => "get".to_owned(),
        "put" => "update".to_owned(),
        "delete" => "delete".to_owned(),
        "head" => "check".to_owned(),
        _ => "post".to_owned(),
    };
    for part in path
        .split(['/', '-', '{', '}'])
        .filter(|part| !part.is_empty())
    {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            id.push(first.to_ascii_uppercase());
            id.extend(chars);
        }
    }
    id
}

fn probe_path(path: &str) -> String {
    let filled: Vec<String> = path
        .split('/')
        .map(|part| match part.starts_with('{') {
            true => format!("probe-{}", part.trim_matches(['{', '}'])),
            false => part.to_owned(),
        })
        .collect();
    format!("/service/rest{}", filled.join("/"))
}

async fn routed(fixture: &Fixture, method: &str, path: &str) -> bool {
    let method = Method::from_bytes(method.to_ascii_uppercase().as_bytes()).unwrap();
    let body = matches!(method, Method::POST | Method::PUT).then(|| json!({}));
    let reply = fixture
        .call(method, &probe_path(path), Auth::Admin, body)
        .await;
    reply.status != StatusCode::METHOD_NOT_ALLOWED
        && !(reply.status == StatusCode::NOT_FOUND && reply.body.is_empty())
}

fn collect_refs(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                match (key.as_str(), value) {
                    ("$ref", Value::String(target)) => {
                        if let Some(name) = target.strip_prefix("#/components/schemas/") {
                            found.insert(name.to_owned());
                        }
                    }
                    _ => collect_refs(value, found),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_refs(item, found)),
        _ => {}
    }
}

async fn generate(fixture: &Fixture) -> Value {
    let reference: Value = serde_json::from_str(REFERENCE).unwrap();
    let mut paths = Map::new();
    let mut tags = BTreeSet::new();
    for (path, item) in reference["paths"].as_object().unwrap() {
        let mut kept = Map::new();
        for (method, operation) in item.as_object().unwrap() {
            if !METHODS.contains(&method.as_str()) {
                kept.insert(method.clone(), operation.clone());
                continue;
            }
            if !routed(fixture, method, path).await {
                continue;
            }
            let mut operation = operation.clone();
            let object = operation.as_object_mut().unwrap();
            object.insert("operationId".into(), json!(operation_id(method, path)));
            object.insert("summary".into(), json!(summary(method, path)));
            for tag in object
                .get("tags")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                tags.insert(tag.as_str().unwrap().to_owned());
            }
            kept.insert(method.clone(), operation);
        }
        if kept.keys().any(|key| METHODS.contains(&key.as_str())) {
            paths.insert(path.clone(), Value::Object(kept));
        }
    }
    let all = &reference["components"]["schemas"];
    let mut needed = BTreeSet::new();
    collect_refs(&Value::Object(paths.clone()), &mut needed);
    loop {
        let mut more = needed.clone();
        for name in &needed {
            collect_refs(&all[name], &mut more);
        }
        if more == needed {
            break;
        }
        needed = more;
    }
    let schemas: Map<String, Value> = needed
        .into_iter()
        .filter_map(|name| all.get(&name).map(|schema| (name, schema.clone())))
        .collect();
    json!({
        "openapi": "3.0.1",
        "info": { "title": "Astrum Repository REST API", "version": "" },
        "servers": [{ "url": "/service/rest" }],
        "tags": tags.into_iter().map(|name| json!({ "name": name })).collect::<Vec<_>>(),
        "paths": paths,
        "components": { "schemas": schemas },
    })
}

#[tokio::test]
async fn the_document_lists_exactly_the_routed_operations() {
    let fixture = Fixture::new().await;
    let generated = generate(&fixture).await;
    let text = serde_json::to_string_pretty(&generated).unwrap() + "\n";
    if std::env::var_os("ASTRUM_WRITE_OPENAPI").is_some() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/routes/support/openapi.json"
        );
        std::fs::write(path, &text).unwrap();
        return;
    }
    let served: Value = serde_json::from_str(SERVED).unwrap();
    let listed = |document: &Value| -> BTreeSet<String> {
        document["paths"]
            .as_object()
            .unwrap()
            .iter()
            .flat_map(|(path, item)| {
                item.as_object()
                    .unwrap()
                    .keys()
                    .filter(|method| METHODS.contains(&method.as_str()))
                    .map(move |method| format!("{} {path}", method.to_uppercase()))
            })
            .collect()
    };
    let (want, have) = (listed(&generated), listed(&served));
    assert!(
        want == have && generated == served,
        "openapi.json is out of date (missing: {:?}, not routed: {:?}); regenerate it with \
         ASTRUM_WRITE_OPENAPI=1 cargo test --workspace --test openapi",
        want.difference(&have).collect::<Vec<_>>(),
        have.difference(&want).collect::<Vec<_>>(),
    );
}

#[tokio::test]
async fn the_document_is_public_and_names_this_version() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(Method::GET, "/service/rest/swagger.json", Auth::None, None)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.headers["content-type"], "application/json");
    let document = reply.json();
    assert_eq!(document["openapi"], "3.0.1");
    assert_eq!(document["info"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(document["servers"][0]["url"], "/service/rest");
    assert!(document["paths"]["/v1/status"]["get"].is_object());
    assert!(document["paths"]["/v1/repositories"]["get"]["summary"].is_string());
}

#[test]
fn summaries_read_like_sentences() {
    assert_eq!(summary("get", "/v1/blobstores"), "List blob stores");
    assert_eq!(
        summary("get", "/v1/blobstores/file/{name}"),
        "Get blob stores file"
    );
    assert_eq!(summary("post", "/v1/tasks/{id}/run"), "Run for tasks");
    assert_eq!(summary("delete", "/v1/assets/{id}"), "Delete assets");
    assert_eq!(
        operation_id("get", "/v1/blobstores/file/{name}"),
        "getV1BlobstoresFileName"
    );
}
