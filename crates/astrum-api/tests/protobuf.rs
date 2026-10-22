mod support;

use std::io::Write;
use std::sync::Arc;

use astrum_format_protobuf::digest;
use astrum_format_protobuf::proto::{
    self, DigestType, ModuleVisibility, create_modules_request, create_or_update_labels_request,
    download_request, graph, label_ref, module_ref, owner_ref, resource_ref, scoped_label_ref,
    update_modules_request, upload_request,
};
use astrum_security::Scope;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use flate2::Compression;
use flate2::write::GzEncoder;
use prost::Message;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const REPOSITORY: &str = "protos";
const MODULES: &str = "buf.registry.module.v1.ModuleService";
const UPLOADS: &str = "buf.registry.module.v1.UploadService";
const COMMITS: &str = "buf.registry.module.v1.CommitService";
const GRAPHS: &str = "buf.registry.module.v1.GraphService";
const DOWNLOADS: &str = "buf.registry.module.v1.DownloadService";
const LABELS: &str = "buf.registry.module.v1.LabelService";
const RESOURCES: &str = "buf.registry.module.v1.ResourceService";
const OWNERS: &str = "buf.registry.owner.v1.OwnerService";
const AUTHN: &str = "buf.alpha.registry.v1alpha1.AuthnService";
const GREETER: &str =
    "syntax = \"proto3\";\n\npackage acme.greeter.v1;\n\nmessage Hello {\n  string name = 1;\n}\n";
const TYPES: &str = "syntax = \"proto3\";\n\npackage acme.types.v1;\n\nimport \"acme/greeter/v1/greeter.proto\";\n\nmessage Wrapper {\n  acme.greeter.v1.Hello hello = 1;\n}\n";
const LICENSE: &str = "MIT\n";

fn storage(policy: &str) -> Value {
    json!({"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy})
}

async fn ready_with(policy: &str, domain: Option<&str>) -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_protobuf::Protobuf));
    let mut body = json!({"name": REPOSITORY, "online": true, "storage": storage(policy)});
    if let Some(host) = domain {
        body["domain"] = json!({"host": host});
    }
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/protobuf/hosted",
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    fixture
}

async fn ready() -> Fixture {
    ready_with("ALLOW", None).await
}

async fn token(fixture: &Fixture, scope: Scope) -> String {
    fixture
        .state
        .security()
        .create_token("admin", "protobuf", scope, None)
        .await
        .unwrap()
        .1
}

async fn post(
    fixture: &Fixture,
    service: &str,
    method: &str,
    token: Option<&str>,
    body: Vec<u8>,
    extra: &[(&str, &str)],
) -> Reply {
    let authorization = token.map(|token| format!("Bearer {token}"));
    let mut headers = vec![("connect-protocol-version", "1")];
    if !extra.iter().any(|(name, _)| *name == "content-type") {
        headers.push(("content-type", "application/proto"));
    }
    if let Some(value) = authorization.as_deref() {
        headers.push(("authorization", value));
    }
    headers.extend_from_slice(extra);
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{REPOSITORY}/{service}/{method}"),
            Auth::None,
            Body::from(body),
            &headers,
        )
        .await
}

async fn rpc<Q: Message, R: Message + Default>(
    fixture: &Fixture,
    service: &str,
    method: &str,
    token: &str,
    request: &Q,
) -> R {
    let reply = post(
        fixture,
        service,
        method,
        Some(token),
        request.encode_to_vec(),
        &[],
    )
    .await;
    assert_eq!(
        reply.status,
        StatusCode::OK,
        "{service}/{method}: {}",
        reply.text()
    );
    assert_eq!(
        reply
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/proto")
    );
    R::decode(reply.body.as_slice()).unwrap()
}

async fn refused<Q: Message>(
    fixture: &Fixture,
    service: &str,
    method: &str,
    token: Option<&str>,
    request: &Q,
) -> (StatusCode, String, String) {
    let reply = post(
        fixture,
        service,
        method,
        token,
        request.encode_to_vec(),
        &[],
    )
    .await;
    assert_ne!(reply.status, StatusCode::OK, "{service}/{method}");
    assert_eq!(
        reply
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
    let body = reply.json();
    (
        reply.status,
        body["code"].as_str().unwrap().to_owned(),
        body["message"].as_str().unwrap().to_owned(),
    )
}

fn module_by_name(owner: &str, module: &str) -> proto::ModuleRef {
    proto::ModuleRef {
        value: Some(module_ref::Value::Name(module_ref::Name {
            owner: owner.to_owned(),
            module: module.to_owned(),
        })),
    }
}

fn module_by_id(id: &str) -> proto::ModuleRef {
    proto::ModuleRef {
        value: Some(module_ref::Value::Id(id.to_owned())),
    }
}

fn resource(
    owner: &str,
    module: &str,
    child: Option<resource_ref::name::Child>,
) -> proto::ResourceRef {
    proto::ResourceRef {
        value: Some(resource_ref::Value::Name(resource_ref::Name {
            owner: owner.to_owned(),
            module: module.to_owned(),
            child,
        })),
    }
}

fn by_label(owner: &str, module: &str, label: &str) -> proto::ResourceRef {
    resource(
        owner,
        module,
        Some(resource_ref::name::Child::LabelName(label.to_owned())),
    )
}

fn by_ref(owner: &str, module: &str, reference: &str) -> proto::ResourceRef {
    resource(
        owner,
        module,
        Some(resource_ref::name::Child::Ref(reference.to_owned())),
    )
}

fn by_commit_id(id: &str) -> proto::ResourceRef {
    proto::ResourceRef {
        value: Some(resource_ref::Value::Id(id.to_owned())),
    }
}

fn label_by_name(owner: &str, module: &str, label: &str) -> proto::LabelRef {
    proto::LabelRef {
        value: Some(label_ref::Value::Name(label_ref::Name {
            owner: owner.to_owned(),
            module: module.to_owned(),
            label: label.to_owned(),
        })),
    }
}

fn file(path: &str, content: &str) -> proto::File {
    proto::File {
        path: path.to_owned(),
        content: content.as_bytes().to_vec(),
    }
}

fn greeter_files() -> Vec<proto::File> {
    vec![
        file("acme/greeter/v1/greeter.proto", GREETER),
        file("LICENSE", LICENSE),
        file("buf.yaml", "version: v2\n"),
    ]
}

fn types_files() -> Vec<proto::File> {
    vec![file("acme/types/v1/types.proto", TYPES)]
}

fn create_request(owner: &str, name: &str) -> proto::CreateModulesRequest {
    proto::CreateModulesRequest {
        values: vec![create_modules_request::Value {
            owner_ref: Some(proto::OwnerRef {
                value: Some(owner_ref::Value::Name(owner.to_owned())),
            }),
            name: name.to_owned(),
            visibility: ModuleVisibility::Public as i32,
            description: String::new(),
            url: String::new(),
            default_label_name: String::new(),
        }],
    }
}

fn upload_content(
    owner: &str,
    module: &str,
    files: Vec<proto::File>,
    labels: &[&str],
) -> upload_request::Content {
    upload_request::Content {
        module_ref: Some(module_by_name(owner, module)),
        files,
        scoped_label_refs: labels
            .iter()
            .map(|name| proto::ScopedLabelRef {
                value: Some(scoped_label_ref::Value::Name((*name).to_owned())),
            })
            .collect(),
        source_control_url: String::new(),
    }
}

async fn create_module(fixture: &Fixture, token: &str, owner: &str, name: &str) -> proto::Module {
    let created: proto::CreateModulesResponse = rpc(
        fixture,
        MODULES,
        "CreateModules",
        token,
        &create_request(owner, name),
    )
    .await;
    assert_eq!(created.modules.len(), 1);
    created.modules.into_iter().next().unwrap()
}

async fn upload(
    fixture: &Fixture,
    token: &str,
    contents: Vec<upload_request::Content>,
    deps: &[&str],
) -> Vec<proto::Commit> {
    let response: proto::UploadResponse = rpc(
        fixture,
        UPLOADS,
        "Upload",
        token,
        &proto::UploadRequest {
            contents,
            dep_commit_ids: deps.iter().map(|id| (*id).to_owned()).collect(),
        },
    )
    .await;
    response.commits
}

async fn download(
    fixture: &Fixture,
    token: &str,
    reference: proto::ResourceRef,
) -> proto::download_response::Content {
    let response: proto::DownloadResponse = rpc(
        fixture,
        DOWNLOADS,
        "Download",
        token,
        &proto::DownloadRequest {
            values: vec![download_request::Value {
                resource_ref: Some(reference),
                file_types: vec![],
                paths: vec![],
                paths_allow_not_exist: false,
            }],
        },
    )
    .await;
    response.contents.into_iter().next().unwrap()
}

fn digest_of(commit: &proto::Commit) -> digest::Value {
    let value = commit.digest.as_ref().unwrap();
    assert_eq!(value.kind, DigestType::B5 as i32);
    value.value.clone().try_into().unwrap()
}

fn recomputed(files: &[proto::File], dependencies: &[digest::Value]) -> digest::Value {
    digest::module_digest(
        files
            .iter()
            .map(|file| (file.path.as_str(), file.content.as_slice())),
        dependencies,
    )
}

#[tokio::test]
async fn a_module_is_created_uploaded_and_downloaded_with_a_digest_the_client_can_check() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;

    let module = create_module(&fixture, &writer, "acme", "greeter").await;
    assert_eq!(module.name, "greeter");
    assert_eq!(module.default_label_name, "main");
    assert_eq!(module.visibility, ModuleVisibility::Public as i32);
    assert_eq!(module.id.len(), 32);
    assert_eq!(module.owner_id.len(), 32);

    let duplicate = refused(
        &fixture,
        MODULES,
        "CreateModules",
        Some(&writer),
        &create_request("acme", "greeter"),
    )
    .await;
    assert_eq!(duplicate.0, StatusCode::CONFLICT);
    assert_eq!(duplicate.1, "already_exists");

    let fetched: proto::GetModulesResponse = rpc(
        &fixture,
        MODULES,
        "GetModules",
        &writer,
        &proto::GetModulesRequest {
            module_refs: vec![module_by_name("acme", "greeter"), module_by_id(&module.id)],
        },
    )
    .await;
    assert_eq!(fetched.modules, vec![module.clone(), module.clone()]);

    let missing = refused(
        &fixture,
        MODULES,
        "GetModules",
        Some(&writer),
        &proto::GetModulesRequest {
            module_refs: vec![module_by_name("acme", "absent")],
        },
    )
    .await;
    assert_eq!(missing.0, StatusCode::NOT_FOUND);
    assert_eq!(missing.1, "not_found");

    let commits = upload(
        &fixture,
        &writer,
        vec![upload_content("acme", "greeter", greeter_files(), &[])],
        &[],
    )
    .await;
    assert_eq!(commits.len(), 1);
    let commit = &commits[0];
    assert_eq!(commit.module_id, module.id);
    assert_eq!(commit.owner_id, module.owner_id);
    assert_eq!(commit.id.len(), 32);
    let module_files = vec![
        file("LICENSE", LICENSE),
        file("acme/greeter/v1/greeter.proto", GREETER),
    ];
    assert_eq!(digest_of(commit), recomputed(&module_files, &[]));

    for reference in [
        resource("acme", "greeter", None),
        by_label("acme", "greeter", "main"),
        by_ref("acme", "greeter", "main"),
        by_ref("acme", "greeter", &commit.id),
        by_commit_id(&commit.id),
    ] {
        let found: proto::GetCommitsResponse = rpc(
            &fixture,
            COMMITS,
            "GetCommits",
            &writer,
            &proto::GetCommitsRequest {
                resource_refs: vec![reference],
            },
        )
        .await;
        assert_eq!(&found.commits, &commits);
    }

    let content = download(&fixture, &writer, by_commit_id(&commit.id)).await;
    assert_eq!(content.commit.as_ref(), Some(commit));
    let mut paths: Vec<&str> = content
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    paths.sort_unstable();
    assert_eq!(paths, vec!["LICENSE", "acme/greeter/v1/greeter.proto"]);
    assert_eq!(digest_of(commit), recomputed(&content.files, &[]));
    let proto_file = content
        .files
        .iter()
        .find(|file| file.path.ends_with(".proto"))
        .unwrap();
    assert_eq!(proto_file.content, GREETER.as_bytes());

    let owners: proto::GetOwnersResponse = rpc(
        &fixture,
        OWNERS,
        "GetOwners",
        &writer,
        &proto::GetOwnersRequest {
            owner_refs: vec![
                proto::OwnerRef {
                    value: Some(owner_ref::Value::Name("acme".to_owned())),
                },
                proto::OwnerRef {
                    value: Some(owner_ref::Value::Id(module.owner_id.clone())),
                },
            ],
        },
    )
    .await;
    assert_eq!(owners.owners.len(), 2);
    for owner in owners.owners {
        let Some(proto::owner::Value::Organization(organization)) = owner.value else {
            panic!("an owner is an organization");
        };
        assert_eq!(organization.name, "acme");
        assert_eq!(organization.id, module.owner_id);
    }

    let listed: proto::ListModulesResponse = rpc(
        &fixture,
        MODULES,
        "ListModules",
        &writer,
        &proto::ListModulesRequest::default(),
    )
    .await;
    assert_eq!(listed.modules, vec![module.clone()]);

    let resources: proto::GetResourcesResponse = rpc(
        &fixture,
        RESOURCES,
        "GetResources",
        &writer,
        &proto::GetResourcesRequest {
            resource_refs: vec![
                resource("acme", "greeter", None),
                by_label("acme", "greeter", "main"),
                by_commit_id(&commit.id),
            ],
        },
    )
    .await;
    assert!(matches!(
        resources.resources[0].value,
        Some(proto::resource::Value::Module(_))
    ));
    assert!(matches!(
        resources.resources[1].value,
        Some(proto::resource::Value::Label(_))
    ));
    assert!(matches!(
        resources.resources[2].value,
        Some(proto::resource::Value::Commit(_))
    ));
}

#[tokio::test]
async fn the_same_content_is_the_same_commit_and_a_change_moves_the_default_label() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    create_module(&fixture, &writer, "acme", "greeter").await;
    let first = upload(
        &fixture,
        &writer,
        vec![upload_content("acme", "greeter", greeter_files(), &[])],
        &[],
    )
    .await;
    let again = upload(
        &fixture,
        &writer,
        vec![upload_content("acme", "greeter", greeter_files(), &[])],
        &[],
    )
    .await;
    assert_eq!(first, again);

    let changed = vec![
        file(
            "acme/greeter/v1/greeter.proto",
            &format!("{GREETER}// v2\n"),
        ),
        file("LICENSE", LICENSE),
    ];
    let second = upload(
        &fixture,
        &writer,
        vec![upload_content("acme", "greeter", changed, &[])],
        &[],
    )
    .await;
    assert_ne!(first[0].id, second[0].id);
    assert_ne!(first[0].digest, second[0].digest);

    let head: proto::GetCommitsResponse = rpc(
        &fixture,
        COMMITS,
        "GetCommits",
        &writer,
        &proto::GetCommitsRequest {
            resource_refs: vec![resource("acme", "greeter", None)],
        },
    )
    .await;
    assert_eq!(head.commits, second);

    let listed: proto::ListCommitsResponse = rpc(
        &fixture,
        COMMITS,
        "ListCommits",
        &writer,
        &proto::ListCommitsRequest {
            resource_ref: Some(resource("acme", "greeter", None)),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(listed.commits.len(), 2);
    assert_eq!(listed.commits[0].id, second[0].id);
    assert!(listed.next_page_token.is_empty());
    let paged: proto::ListCommitsResponse = rpc(
        &fixture,
        COMMITS,
        "ListCommits",
        &writer,
        &proto::ListCommitsRequest {
            resource_ref: Some(resource("acme", "greeter", None)),
            page_size: 1,
            order: proto::list_commits_request::Order::CreateTimeAsc as i32,
            ..Default::default()
        },
    )
    .await;
    assert_eq!(paged.commits[0].id, first[0].id);
    assert_eq!(paged.next_page_token, "1");

    let history: proto::ListLabelHistoryResponse = rpc(
        &fixture,
        LABELS,
        "ListLabelHistory",
        &writer,
        &proto::ListLabelHistoryRequest {
            label_ref: Some(label_by_name("acme", "greeter", "main")),
            ..Default::default()
        },
    )
    .await;
    let ids: Vec<&str> = history
        .values
        .iter()
        .map(|value| value.commit.as_ref().unwrap().id.as_str())
        .collect();
    assert_eq!(ids, vec![second[0].id.as_str(), first[0].id.as_str()]);
    let earlier = download(&fixture, &writer, by_commit_id(&first[0].id)).await;
    assert_eq!(earlier.commit.as_ref(), Some(&first[0]));
}

#[tokio::test]
async fn a_module_depends_on_another_and_the_graph_and_the_digest_say_so() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    create_module(&fixture, &writer, "acme", "greeter").await;
    create_module(&fixture, &writer, "acme", "types").await;
    let base = upload(
        &fixture,
        &writer,
        vec![upload_content("acme", "greeter", greeter_files(), &[])],
        &[],
    )
    .await;
    let dependent = upload(
        &fixture,
        &writer,
        vec![upload_content("acme", "types", types_files(), &[])],
        &[&base[0].id],
    )
    .await;
    assert_eq!(
        digest_of(&dependent[0]),
        recomputed(&types_files(), &[digest_of(&base[0])])
    );
    assert_ne!(digest_of(&dependent[0]), recomputed(&types_files(), &[]));

    let response: proto::GetGraphResponse = rpc(
        &fixture,
        GRAPHS,
        "GetGraph",
        &writer,
        &proto::GetGraphRequest {
            resource_refs: vec![resource("acme", "types", None)],
        },
    )
    .await;
    let graph = response.graph.unwrap();
    let ids: Vec<&str> = graph
        .commits
        .iter()
        .map(|commit| commit.id.as_str())
        .collect();
    assert_eq!(ids, vec![dependent[0].id.as_str(), base[0].id.as_str()]);
    assert_eq!(
        graph.edges,
        vec![graph::Edge {
            from_node: Some(graph::Node {
                commit_id: dependent[0].id.clone()
            }),
            to_node: Some(graph::Node {
                commit_id: base[0].id.clone()
            }),
        }]
    );
    assert!(graph.registry_commit_ids.is_empty());

    let two_modules = refused(
        &fixture,
        GRAPHS,
        "GetGraph",
        Some(&writer),
        &proto::GetGraphRequest {
            resource_refs: vec![
                resource("acme", "types", None),
                by_label("acme", "types", "main"),
            ],
        },
    )
    .await;
    assert_eq!(two_modules.1, "invalid_argument");

    let kept = refused(
        &fixture,
        MODULES,
        "DeleteModules",
        Some(&writer),
        &proto::DeleteModulesRequest {
            module_refs: vec![module_by_name("acme", "greeter")],
        },
    )
    .await;
    assert_eq!(kept.1, "failed_precondition");
    assert!(kept.2.contains("acme/types"), "{}", kept.2);

    let unknown_dependency = refused(
        &fixture,
        UPLOADS,
        "Upload",
        Some(&writer),
        &proto::UploadRequest {
            contents: vec![upload_content("acme", "types", types_files(), &[])],
            dep_commit_ids: vec!["0".repeat(32)],
        },
    )
    .await;
    assert_eq!(unknown_dependency.1, "not_found");

    for module in ["types", "greeter"] {
        let _: proto::DeleteModulesResponse = rpc(
            &fixture,
            MODULES,
            "DeleteModules",
            &writer,
            &proto::DeleteModulesRequest {
                module_refs: vec![module_by_name("acme", module)],
            },
        )
        .await;
    }
    let gone = refused(
        &fixture,
        COMMITS,
        "GetCommits",
        Some(&writer),
        &proto::GetCommitsRequest {
            resource_refs: vec![by_commit_id(&base[0].id)],
        },
    )
    .await;
    assert_eq!(gone.1, "not_found");
    create_module(&fixture, &writer, "acme", "greeter").await;
}

#[tokio::test]
async fn modules_of_one_upload_depend_on_each_other_through_their_imports() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    create_module(&fixture, &writer, "acme", "greeter").await;
    create_module(&fixture, &writer, "acme", "types").await;
    let commits = upload(
        &fixture,
        &writer,
        vec![
            upload_content("acme", "types", types_files(), &[]),
            upload_content("acme", "greeter", greeter_files(), &[]),
        ],
        &[],
    )
    .await;
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].owner_id, commits[1].owner_id);
    assert_ne!(commits[0].module_id, commits[1].module_id);
    let greeter_digest = digest_of(&commits[1]);
    assert_eq!(
        digest_of(&commits[0]),
        recomputed(&types_files(), &[greeter_digest])
    );
    let greeter_files: Vec<proto::File> = greeter_files()
        .into_iter()
        .filter(|file| file.path != "buf.yaml")
        .collect();
    assert_eq!(greeter_digest, recomputed(&greeter_files, &[]));
}

#[tokio::test]
async fn labels_are_created_moved_archived_and_unarchived() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    create_module(&fixture, &writer, "acme", "greeter").await;
    let first = upload(
        &fixture,
        &writer,
        vec![upload_content(
            "acme",
            "greeter",
            greeter_files(),
            &["release/v1", "main"],
        )],
        &[],
    )
    .await;
    let labelled: proto::ListLabelsResponse = rpc(
        &fixture,
        LABELS,
        "ListLabels",
        &writer,
        &proto::ListLabelsRequest {
            resource_ref: Some(resource("acme", "greeter", None)),
            order: proto::list_labels_request::Order::CreateTimeAsc as i32,
            ..Default::default()
        },
    )
    .await;
    let mut names: Vec<&str> = labelled
        .labels
        .iter()
        .map(|label| label.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["main", "release/v1"]);
    assert!(
        labelled
            .labels
            .iter()
            .all(|label| label.commit_id == first[0].id)
    );

    let by_commit: proto::ListLabelsResponse = rpc(
        &fixture,
        LABELS,
        "ListLabels",
        &writer,
        &proto::ListLabelsRequest {
            resource_ref: Some(by_commit_id(&first[0].id)),
            name_query: "RELEASE".to_owned(),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(by_commit.labels.len(), 1);
    assert_eq!(by_commit.labels[0].name, "release/v1");

    let second = upload(
        &fixture,
        &writer,
        vec![upload_content(
            "acme",
            "greeter",
            vec![file(
                "acme/greeter/v1/greeter.proto",
                &format!("{GREETER}// second\n"),
            )],
            &["main"],
        )],
        &[],
    )
    .await;
    let release: proto::GetLabelsResponse = rpc(
        &fixture,
        LABELS,
        "GetLabels",
        &writer,
        &proto::GetLabelsRequest {
            label_refs: vec![label_by_name("acme", "greeter", "release/v1")],
        },
    )
    .await;
    assert_eq!(release.labels[0].commit_id, first[0].id);
    assert!(release.labels[0].archive_time.is_none());

    let older = refused(
        &fixture,
        LABELS,
        "CreateOrUpdateLabels",
        Some(&writer),
        &proto::CreateOrUpdateLabelsRequest {
            values: vec![create_or_update_labels_request::Value {
                label_ref: Some(label_by_name("acme", "greeter", "main")),
                commit_id: first[0].id.clone(),
            }],
        },
    )
    .await;
    assert_eq!(older.1, "failed_precondition");
    let moved: proto::CreateOrUpdateLabelsResponse = rpc(
        &fixture,
        LABELS,
        "CreateOrUpdateLabels",
        &writer,
        &proto::CreateOrUpdateLabelsRequest {
            values: vec![create_or_update_labels_request::Value {
                label_ref: Some(label_by_name("acme", "greeter", "release/v1")),
                commit_id: second[0].id.clone(),
            }],
        },
    )
    .await;
    assert_eq!(moved.labels[0].commit_id, second[0].id);
    let created: proto::CreateOrUpdateLabelsResponse = rpc(
        &fixture,
        LABELS,
        "CreateOrUpdateLabels",
        &writer,
        &proto::CreateOrUpdateLabelsRequest {
            values: vec![create_or_update_labels_request::Value {
                label_ref: Some(label_by_name("acme", "greeter", "stable")),
                commit_id: first[0].id.clone(),
            }],
        },
    )
    .await;
    assert_eq!(created.labels[0].name, "stable");

    let by_name_ref: proto::GetCommitsResponse = rpc(
        &fixture,
        COMMITS,
        "GetCommits",
        &writer,
        &proto::GetCommitsRequest {
            resource_refs: vec![by_ref("acme", "greeter", "release/v1")],
        },
    )
    .await;
    assert_eq!(by_name_ref.commits, second);

    let default_label = refused(
        &fixture,
        LABELS,
        "ArchiveLabels",
        Some(&writer),
        &proto::ArchiveLabelsRequest {
            label_refs: vec![label_by_name("acme", "greeter", "main")],
        },
    )
    .await;
    assert_eq!(default_label.1, "failed_precondition");
    let _: proto::ArchiveLabelsResponse = rpc(
        &fixture,
        LABELS,
        "ArchiveLabels",
        &writer,
        &proto::ArchiveLabelsRequest {
            label_refs: vec![label_by_name("acme", "greeter", "stable")],
        },
    )
    .await;
    let archived: proto::GetLabelsResponse = rpc(
        &fixture,
        LABELS,
        "GetLabels",
        &writer,
        &proto::GetLabelsRequest {
            label_refs: vec![label_by_name("acme", "greeter", "stable")],
        },
    )
    .await;
    assert!(archived.labels[0].archive_time.is_some());
    let visible: proto::ListLabelsResponse = rpc(
        &fixture,
        LABELS,
        "ListLabels",
        &writer,
        &proto::ListLabelsRequest {
            resource_ref: Some(resource("acme", "greeter", None)),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(visible.labels.len(), 2);
    let everything: proto::ListLabelsResponse = rpc(
        &fixture,
        LABELS,
        "ListLabels",
        &writer,
        &proto::ListLabelsRequest {
            resource_ref: Some(resource("acme", "greeter", None)),
            archive_filter: proto::list_labels_request::ArchiveFilter::All as i32,
            ..Default::default()
        },
    )
    .await;
    assert_eq!(everything.labels.len(), 3);
    let _: proto::UnarchiveLabelsResponse = rpc(
        &fixture,
        LABELS,
        "UnarchiveLabels",
        &writer,
        &proto::UnarchiveLabelsRequest {
            label_refs: vec![label_by_name("acme", "greeter", "stable")],
        },
    )
    .await;
    let restored: proto::GetLabelsResponse = rpc(
        &fixture,
        LABELS,
        "GetLabels",
        &writer,
        &proto::GetLabelsRequest {
            label_refs: vec![label_by_name("acme", "greeter", "stable")],
        },
    )
    .await;
    assert!(restored.labels[0].archive_time.is_none());

    let changed_default: proto::UpdateModulesResponse = rpc(
        &fixture,
        MODULES,
        "UpdateModules",
        &writer,
        &proto::UpdateModulesRequest {
            values: vec![update_modules_request::Value {
                module_ref: Some(module_by_name("acme", "greeter")),
                visibility: Some(ModuleVisibility::Private as i32),
                state: Some(proto::ModuleState::Deprecated as i32),
                description: Some("Greets".to_owned()),
                url: None,
                default_label_name: Some("release/v1".to_owned()),
            }],
        },
    )
    .await;
    let updated = &changed_default.modules[0];
    assert_eq!(updated.default_label_name, "release/v1");
    assert_eq!(updated.state, proto::ModuleState::Deprecated as i32);
    assert_eq!(updated.description, "Greets");
    let head: proto::GetCommitsResponse = rpc(
        &fixture,
        COMMITS,
        "GetCommits",
        &writer,
        &proto::GetCommitsRequest {
            resource_refs: vec![resource("acme", "greeter", None)],
        },
    )
    .await;
    assert_eq!(head.commits, second);
}

#[tokio::test]
async fn download_filters_by_type_and_path() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    create_module(&fixture, &writer, "acme", "greeter").await;
    let files = vec![
        file("acme/greeter/v1/greeter.proto", GREETER),
        file("acme/other/v1/other.proto", "syntax = \"proto3\";\n"),
        file("LICENSE", LICENSE),
        file("README.md", "# readme\n"),
        file("buf.md", "# buf\n"),
    ];
    let commits = upload(
        &fixture,
        &writer,
        vec![upload_content("acme", "greeter", files, &[])],
        &[],
    )
    .await;
    let fetch = |file_types: Vec<i32>, paths: Vec<&str>, allow: bool| {
        let fixture = &fixture;
        let writer = writer.clone();
        let id = commits[0].id.clone();
        let paths: Vec<String> = paths.into_iter().map(str::to_owned).collect();
        async move {
            let reply = post(
                fixture,
                DOWNLOADS,
                "Download",
                Some(&writer),
                proto::DownloadRequest {
                    values: vec![download_request::Value {
                        resource_ref: Some(by_commit_id(&id)),
                        file_types,
                        paths,
                        paths_allow_not_exist: allow,
                    }],
                }
                .encode_to_vec(),
                &[],
            )
            .await;
            (reply.status, reply.body)
        }
    };
    let (status, body) = fetch(vec![], vec![], false).await;
    assert_eq!(status, StatusCode::OK);
    let all = proto::DownloadResponse::decode(body.as_slice()).unwrap();
    let mut paths: Vec<&str> = all.contents[0]
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        vec![
            "LICENSE",
            "acme/greeter/v1/greeter.proto",
            "acme/other/v1/other.proto",
            "buf.md"
        ]
    );
    let (status, body) = fetch(
        vec![proto::FileType::Proto as i32],
        vec!["acme/greeter"],
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let one = proto::DownloadResponse::decode(body.as_slice()).unwrap();
    assert_eq!(one.contents[0].files.len(), 1);
    assert_eq!(
        one.contents[0].files[0].path,
        "acme/greeter/v1/greeter.proto"
    );
    let (status, _) = fetch(vec![], vec!["acme/absent"], false).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, body) = fetch(vec![], vec!["acme/absent"], true).await;
    assert_eq!(status, StatusCode::OK);
    let none = proto::DownloadResponse::decode(body.as_slice()).unwrap();
    assert!(none.contents[0].files.is_empty());
    assert!(none.contents[0].commit.is_some());
}

#[tokio::test]
async fn connect_requests_may_be_gets_or_compressed_and_wrong_ones_get_connect_errors() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    let module = create_module(&fixture, &writer, "acme", "greeter").await;

    let message = proto::GetModulesRequest {
        module_refs: vec![module_by_name("acme", "greeter")],
    }
    .encode_to_vec();
    let authorization = format!("Bearer {writer}");
    let query = format!(
        "connect=v1&encoding=proto&base64=1&message={}",
        URL_SAFE_NO_PAD.encode(&message)
    );
    let by_get = fixture
        .send_body(
            Method::GET,
            &format!("/repository/{REPOSITORY}/{MODULES}/GetModules?{query}"),
            Auth::None,
            Body::empty(),
            &[("authorization", &authorization)],
        )
        .await;
    assert_eq!(by_get.status, StatusCode::OK, "{}", by_get.text());
    let decoded = proto::GetModulesResponse::decode(by_get.body.as_slice()).unwrap();
    assert_eq!(decoded.modules, vec![module.clone()]);

    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&message).unwrap();
    let packed = encoder.finish().unwrap();
    let compressed = post(
        &fixture,
        MODULES,
        "GetModules",
        Some(&writer),
        packed,
        &[("content-encoding", "gzip")],
    )
    .await;
    assert_eq!(compressed.status, StatusCode::OK, "{}", compressed.text());
    assert_eq!(
        proto::GetModulesResponse::decode(compressed.body.as_slice())
            .unwrap()
            .modules,
        vec![module.clone()]
    );

    let get_for_a_change = fixture
        .send_body(
            Method::GET,
            &format!("/repository/{REPOSITORY}/{MODULES}/CreateModules?{query}"),
            Auth::None,
            Body::empty(),
            &[("authorization", &authorization)],
        )
        .await;
    assert_eq!(get_for_a_change.status, StatusCode::METHOD_NOT_ALLOWED);

    let json_body = post(
        &fixture,
        MODULES,
        "GetModules",
        Some(&writer),
        b"{}".to_vec(),
        &[("content-type", "application/json")],
    )
    .await;
    assert_eq!(json_body.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let garbage = post(
        &fixture,
        MODULES,
        "GetModules",
        Some(&writer),
        vec![0xff, 0xff, 0xff],
        &[],
    )
    .await;
    assert_eq!(garbage.status, StatusCode::BAD_REQUEST);
    assert_eq!(garbage.json()["code"], "invalid_argument");

    let legacy = post(
        &fixture,
        "buf.registry.module.v1beta1.UploadService",
        "Upload",
        Some(&writer),
        vec![],
        &[],
    )
    .await;
    assert_eq!(legacy.status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(legacy.json()["code"], "unimplemented");
    let unknown = post(&fixture, MODULES, "Frobnicate", Some(&writer), vec![], &[]).await;
    assert_eq!(unknown.status, StatusCode::NOT_IMPLEMENTED);

    for (owner, name) in [
        ("Acme", "greeter"),
        ("acme", "G"),
        ("acme", "a"),
        ("-acme", "greeter"),
    ] {
        let bad = refused(
            &fixture,
            MODULES,
            "CreateModules",
            Some(&writer),
            &create_request(owner, name),
        )
        .await;
        assert_eq!(bad.0, StatusCode::BAD_REQUEST, "{owner}/{name}");
        assert_eq!(bad.1, "invalid_argument");
    }
    let no_protos = refused(
        &fixture,
        UPLOADS,
        "Upload",
        Some(&writer),
        &proto::UploadRequest {
            contents: vec![upload_content(
                "acme",
                "greeter",
                vec![file("LICENSE", "x")],
                &[],
            )],
            dep_commit_ids: vec![],
        },
    )
    .await;
    assert_eq!(no_protos.1, "invalid_argument");
    let bad_path = refused(
        &fixture,
        UPLOADS,
        "Upload",
        Some(&writer),
        &proto::UploadRequest {
            contents: vec![upload_content(
                "acme",
                "greeter",
                vec![file("../escape.proto", "x")],
                &[],
            )],
            dep_commit_ids: vec![],
        },
    )
    .await;
    assert_eq!(bad_path.1, "invalid_argument");
    let missing_module = refused(
        &fixture,
        UPLOADS,
        "Upload",
        Some(&writer),
        &proto::UploadRequest {
            contents: vec![upload_content("acme", "absent", greeter_files(), &[])],
            dep_commit_ids: vec![],
        },
    )
    .await;
    assert_eq!(missing_module.1, "not_found");
    let nothing_yet = refused(
        &fixture,
        COMMITS,
        "GetCommits",
        Some(&writer),
        &proto::GetCommitsRequest {
            resource_refs: vec![resource("acme", "greeter", None)],
        },
    )
    .await;
    assert_eq!(nothing_yet.1, "not_found");
}

#[tokio::test]
async fn permissions_follow_the_roles_of_the_caller() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    let reader_token = token(&fixture, Scope::Read).await;
    create_module(&fixture, &writer, "acme", "greeter").await;
    let commits = upload(
        &fixture,
        &writer,
        vec![upload_content("acme", "greeter", greeter_files(), &[])],
        &[],
    )
    .await;

    let anonymous_read = refused(
        &fixture,
        MODULES,
        "GetModules",
        None,
        &proto::GetModulesRequest {
            module_refs: vec![module_by_name("acme", "greeter")],
        },
    )
    .await;
    assert_eq!(anonymous_read.0, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_read.1, "unauthenticated");
    let anonymous_write = refused(
        &fixture,
        MODULES,
        "CreateModules",
        None,
        &create_request("acme", "other"),
    )
    .await;
    assert_eq!(anonymous_write.1, "unauthenticated");

    let read_only_write = refused(
        &fixture,
        MODULES,
        "CreateModules",
        Some(&reader_token),
        &create_request("acme", "other"),
    )
    .await;
    assert_eq!(read_only_write.0, StatusCode::FORBIDDEN);
    assert_eq!(read_only_write.1, "permission_denied");
    let read_only_upload = refused(
        &fixture,
        UPLOADS,
        "Upload",
        Some(&reader_token),
        &proto::UploadRequest {
            contents: vec![upload_content("acme", "greeter", types_files(), &[])],
            dep_commit_ids: vec![],
        },
    )
    .await;
    assert_eq!(read_only_upload.1, "permission_denied");
    let read_only_same = refused(
        &fixture,
        UPLOADS,
        "Upload",
        Some(&reader_token),
        &proto::UploadRequest {
            contents: vec![upload_content("acme", "greeter", greeter_files(), &[])],
            dep_commit_ids: vec![],
        },
    )
    .await;
    assert_eq!(read_only_same.1, "permission_denied");
    let read_only_delete = refused(
        &fixture,
        MODULES,
        "DeleteModules",
        Some(&reader_token),
        &proto::DeleteModulesRequest {
            module_refs: vec![module_by_name("acme", "greeter")],
        },
    )
    .await;
    assert_eq!(read_only_delete.1, "permission_denied");
    let content = download(&fixture, &reader_token, by_commit_id(&commits[0].id)).await;
    assert_eq!(content.commit.as_ref(), Some(&commits[0]));

    fixture
        .add_user("reader", "reader-password-1", &["as-anonymous"])
        .await;
    let basic = fixture
        .send_body(
            Method::POST,
            &format!("/repository/{REPOSITORY}/{COMMITS}/GetCommits"),
            Auth::Basic("reader", "reader-password-1"),
            Body::from(
                proto::GetCommitsRequest {
                    resource_refs: vec![resource("acme", "greeter", None)],
                }
                .encode_to_vec(),
            ),
            &[("content-type", "application/proto")],
        )
        .await;
    assert_eq!(basic.status, StatusCode::OK, "{}", basic.text());
    let denied = fixture
        .send_body(
            Method::POST,
            &format!("/repository/{REPOSITORY}/{MODULES}/CreateModules"),
            Auth::Basic("reader", "reader-password-1"),
            Body::from(create_request("acme", "other").encode_to_vec()),
            &[("content-type", "application/proto")],
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_read_only_repository_takes_no_modules() {
    let fixture = ready_with("DENY", None).await;
    let writer = token(&fixture, Scope::Write).await;
    let refusal = refused(
        &fixture,
        MODULES,
        "CreateModules",
        Some(&writer),
        &create_request("acme", "greeter"),
    )
    .await;
    assert_eq!(refusal.1, "failed_precondition");
    assert!(refusal.2.contains("read-only"), "{}", refusal.2);
}

#[tokio::test]
async fn a_root_domain_serves_the_calls_without_a_path_prefix() {
    let fixture = ready_with("ALLOW", Some("registry.example.test")).await;
    let writer = token(&fixture, Scope::Write).await;
    let authorization = format!("Bearer {writer}");
    let created = fixture
        .send_body(
            Method::POST,
            &format!("/{MODULES}/CreateModules"),
            Auth::None,
            Body::from(create_request("acme", "greeter").encode_to_vec()),
            &[
                ("host", "registry.example.test:8443"),
                ("content-type", "application/proto"),
                ("authorization", &authorization),
            ],
        )
        .await;
    assert_eq!(created.status, StatusCode::OK, "{}", created.text());
    let uploaded = fixture
        .send_body(
            Method::POST,
            &format!("/{UPLOADS}/Upload"),
            Auth::None,
            Body::from(
                proto::UploadRequest {
                    contents: vec![upload_content("acme", "greeter", greeter_files(), &[])],
                    dep_commit_ids: vec![],
                }
                .encode_to_vec(),
            ),
            &[
                ("host", "registry.example.test"),
                ("content-type", "application/proto"),
                ("authorization", &authorization),
            ],
        )
        .await;
    assert_eq!(uploaded.status, StatusCode::OK, "{}", uploaded.text());
    let commit = proto::UploadResponse::decode(uploaded.body.as_slice())
        .unwrap()
        .commits
        .remove(0);

    let stored = fixture
        .send_body(
            Method::GET,
            &format!(
                "/acme/greeter/commits/{}/acme/greeter/v1/greeter.proto",
                commit.id
            ),
            Auth::None,
            Body::empty(),
            &[
                ("host", "registry.example.test"),
                ("authorization", &authorization),
            ],
        )
        .await;
    assert_eq!(stored.status, StatusCode::OK);
    assert_eq!(stored.body, GREETER.as_bytes());

    let page = fixture
        .send_body(
            Method::GET,
            "/",
            Auth::None,
            Body::empty(),
            &[("host", "registry.example.test")],
        )
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("registry.example.test"));
}

#[tokio::test]
async fn the_stored_files_are_browsable_assets_of_a_commit_component() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    create_module(&fixture, &writer, "acme", "greeter").await;
    let commits = upload(
        &fixture,
        &writer,
        vec![upload_content(
            "acme",
            "greeter",
            greeter_files(),
            &["main"],
        )],
        &[],
    )
    .await;
    let listing = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/components?repository={REPOSITORY}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    let items = listing.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["group"], "acme");
    assert_eq!(items[0]["name"], "greeter");
    assert_eq!(items[0]["version"], commits[0].id);
    let paths: Vec<String> = items[0]["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["path"].as_str().unwrap().to_owned())
        .collect();
    assert!(paths.iter().any(|path| path.ends_with("/commit.json")));
    assert!(paths.iter().any(|path| path.ends_with("/LICENSE")));
}

#[tokio::test]
async fn the_current_user_is_the_owner_of_the_token_so_that_buf_registry_login_works() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    let response: proto::GetCurrentUserResponse = rpc(
        &fixture,
        AUTHN,
        "GetCurrentUser",
        &writer,
        &proto::GetCurrentUserRequest {},
    )
    .await;
    let user = response.user.unwrap();
    assert_eq!(user.username, "admin");
    assert_eq!(user.id.len(), 32);
    let subject: proto::GetCurrentUserSubjectResponse = rpc(
        &fixture,
        AUTHN,
        "GetCurrentUserSubject",
        &writer,
        &proto::GetCurrentUserSubjectRequest {},
    )
    .await;
    assert_eq!(subject.subject, "admin");

    let anonymous = refused(
        &fixture,
        AUTHN,
        "GetCurrentUser",
        None,
        &proto::GetCurrentUserRequest {},
    )
    .await;
    assert_eq!(anonymous.0, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous.1, "unauthenticated");
    let wrong = refused(
        &fixture,
        AUTHN,
        "GetCurrentUser",
        Some("astrum_0000000000000000000000000000000000000"),
        &proto::GetCurrentUserRequest {},
    )
    .await;
    assert_eq!(wrong.0, StatusCode::UNAUTHORIZED);
}
