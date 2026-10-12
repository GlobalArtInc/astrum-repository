use serde_json::{Value, json};

use super::*;

const PGP: &str =
    "-----BEGIN PGP PRIVATE KEY BLOCK-----\n\nlQ\n-----END PGP PRIVATE KEY BLOCK-----\n";

fn seal(plain: &str) -> String {
    format!("astrum:v1:{}", plain.len())
}

fn minimal(spec: Spec, name: &str) -> Value {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
    });
    match spec.kind {
        RepositoryType::Hosted => body["storage"]["writePolicy"] = json!("ALLOW_ONCE"),
        RepositoryType::Proxy => {
            body["proxy"] = json!({"remoteUrl": "https://upstream.example.org/", "contentMaxAge": 1440, "metadataMaxAge": 1440});
            body["negativeCache"] = json!({"enabled": true, "timeToLive": 1440});
            body["httpClient"] = json!({"blocked": false, "autoBlock": true});
        }
        RepositoryType::Group => body["group"] = json!({"memberNames": ["member"]}),
    }
    if spec.presence(Section::Maven).is_some() {
        body["maven"] = json!({"versionPolicy": "RELEASE", "layoutPolicy": "STRICT"});
    }
    if spec.format.id == "apt" {
        body["apt"] = match spec.kind {
            RepositoryType::Hosted => json!({"distribution": "bookworm"}),
            _ => json!({"flat": false}),
        };
    }
    if spec.presence(Section::AptSigning) == Some(Presence::Required) {
        body["aptSigning"] = json!({"keypair": PGP});
    }
    body
}

fn prepare(
    spec: Spec,
    body: Value,
    stored: Option<&Repository>,
) -> Result<Repository, Vec<String>> {
    let request: Request = serde_json::from_value(body).unwrap();
    Repository::prepare(spec, request, stored, &seal).map_err(|problems| {
        problems
            .into_iter()
            .map(|problem| format!("{}: {}", problem.id, problem.message))
            .collect()
    })
}

fn spec(format: &str, kind: &str) -> Spec {
    Spec::from_api(format, kind).unwrap()
}

#[test]
fn section_keys_are_the_json_names() {
    let mut attributes = Attributes::default();
    for section in Section::ALL {
        attributes.fill(*section);
    }
    let value = serde_json::to_value(&attributes).unwrap();
    let keys: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    for section in Section::ALL {
        assert!(keys.contains(&section.key()), "{section:?}");
    }
    assert_eq!(keys.len(), Section::ALL.len());
}

#[test]
fn names_follow_the_documented_pattern() {
    for good in ["maven-releases", "-x", "a.b_c", "A1", &"x".repeat(200)] {
        assert!(
            prepare(
                spec("raw", "hosted"),
                minimal(spec("raw", "hosted"), good),
                None
            )
            .is_ok(),
            "{good}"
        );
    }
    for bad in ["", "_x", ".x", "bad name", "a/b", "ä", &"x".repeat(201)] {
        let problems = prepare(
            spec("raw", "hosted"),
            minimal(spec("raw", "hosted"), bad),
            None,
        )
        .unwrap_err();
        assert!(
            problems[0].starts_with("PARAMETER name: "),
            "{bad}: {problems:?}"
        );
    }
}

#[test]
fn every_format_and_type_round_trips_through_its_document() {
    for spec in Spec::all() {
        let first = prepare(spec, minimal(spec, "first"), None)
            .unwrap_or_else(|problems| panic!("{spec:?}: {problems:?}"));
        let document = first.document("http://localhost:8081");
        assert_eq!(document["format"], spec.format.id);
        assert_eq!(document["type"], spec.kind.as_str());
        assert_eq!(document["url"], "http://localhost:8081/repository/first");
        let mut again = document.clone();
        again["name"] = json!("second");
        if spec.presence(Section::AptSigning) == Some(Presence::Required) {
            again["aptSigning"]["keypair"] = json!(PGP);
        }
        let second =
            prepare(spec, again, None).unwrap_or_else(|problems| panic!("{spec:?}: {problems:?}"));
        let mut expected = document.clone();
        expected["name"] = json!("second");
        expected["url"] = json!("http://localhost:8081/repository/second");
        assert_eq!(
            second.document("http://localhost:8081"),
            expected,
            "{spec:?}"
        );
    }
}

#[test]
fn documents_hold_exactly_the_sections_of_their_kind() {
    let keys = |format: &str, kind: &str| -> Vec<String> {
        let spec = spec(format, kind);
        let repository = prepare(spec, minimal(spec, "x"), None).unwrap();
        let mut keys: Vec<String> = repository
            .document("http://h")
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        keys.sort();
        keys
    };
    assert_eq!(
        keys("maven", "hosted"),
        [
            "cleanup",
            "component",
            "format",
            "maven",
            "name",
            "online",
            "storage",
            "type",
            "url"
        ]
    );
    assert_eq!(
        keys("npm", "proxy"),
        [
            "cleanup",
            "firewall",
            "format",
            "httpClient",
            "name",
            "negativeCache",
            "npm",
            "online",
            "proxy",
            "replication",
            "routingRuleName",
            "storage",
            "type",
            "url"
        ]
    );
    assert_eq!(
        keys("pypi", "group"),
        [
            "format", "group", "name", "online", "storage", "type", "url"
        ]
    );
    assert_eq!(
        keys("yum", "hosted"),
        [
            "cleanup",
            "component",
            "format",
            "name",
            "online",
            "storage",
            "type",
            "url",
            "yum"
        ]
    );
    assert_eq!(
        keys("alpine", "group"),
        [
            "alpineSigning",
            "format",
            "group",
            "name",
            "online",
            "storage",
            "type",
            "url"
        ]
    );
    assert_eq!(
        keys("docker", "proxy"),
        [
            "cleanup",
            "docker",
            "dockerProxy",
            "firewall",
            "format",
            "httpClient",
            "name",
            "negativeCache",
            "online",
            "proxy",
            "replication",
            "routingRuleName",
            "storage",
            "type",
            "url"
        ]
    );
}

#[test]
fn documents_show_defaults_and_nulls_like_the_reference() {
    let proxy = prepare(
        spec("raw", "proxy"),
        minimal(spec("raw", "proxy"), "x"),
        None,
    )
    .unwrap();
    let document = proxy.document("http://h");
    assert_eq!(document["cleanup"], Value::Null);
    assert_eq!(document["firewall"], Value::Null);
    assert_eq!(document["routingRuleName"], Value::Null);
    assert_eq!(
        document["replication"],
        json!({"preemptivePullEnabled": false, "assetPathRegex": null})
    );
    assert_eq!(
        document["storage"],
        json!({"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"})
    );
    assert_eq!(document["httpClient"]["authentication"], Value::Null);
    assert_eq!(document["proxy"]["preserveEncodedCharacters"], json!(false));
    let hosted = prepare(
        spec("docker", "hosted"),
        minimal(spec("docker", "hosted"), "x"),
        None,
    )
    .unwrap()
    .document("http://h");
    assert_eq!(hosted["storage"]["latestPolicy"], Value::Null);
    assert_eq!(
        hosted["docker"],
        json!({"v1Enabled": false, "forceBasicAuth": true})
    );
    assert_eq!(hosted["component"], json!({"proprietaryComponents": false}));
}

#[test]
fn maven_proxies_show_an_unset_preemptive_flag_as_null() {
    let spec = spec("maven", "proxy");
    let mut body = minimal(spec, "x");
    body["httpClient"]["authentication"] =
        json!({"type": "username", "username": "u", "password": "p"});
    let document = prepare(spec, body.clone(), None)
        .unwrap()
        .document("http://h");
    assert_eq!(
        document["httpClient"]["authentication"],
        json!({"type": "username", "username": "u", "ntlmHost": null, "ntlmDomain": null, "preemptive": null})
    );
    body["httpClient"]["authentication"]["preemptive"] = json!(false);
    let document = prepare(spec, body, None).unwrap().document("http://h");
    assert_eq!(
        document["httpClient"]["authentication"]["preemptive"],
        json!(false)
    );
    let raw = Spec::from_api("raw", "proxy").unwrap();
    let mut body = minimal(raw, "x");
    body["httpClient"]["authentication"] =
        json!({"type": "username", "username": "u", "password": "p", "preemptive": true});
    let document = prepare(raw, body, None).unwrap().document("http://h");
    assert!(
        document["httpClient"]["authentication"]
            .get("preemptive")
            .is_none()
    );
}

#[test]
fn missing_and_misplaced_sections() {
    let problems = prepare(spec("raw", "proxy"), json!({}), None).unwrap_err();
    assert_eq!(
        problems,
        [
            "PARAMETER name: must not be empty",
            "PARAMETER online: must not be null",
            "PARAMETER storage: must not be null",
            "PARAMETER proxy: must not be null",
            "PARAMETER negativeCache: must not be null",
            "PARAMETER httpClient: must not be null",
        ]
    );
    let problems = prepare(
        spec("maven", "hosted"),
        json!({"name": "x", "online": true}),
        None,
    )
    .unwrap_err();
    assert!(problems.contains(&"PARAMETER maven: must not be null".to_owned()));
    let mut body = minimal(spec("raw", "hosted"), "x");
    body["maven"] = json!({"versionPolicy": "RELEASE"});
    body["proxy"] = json!({"remoteUrl": "ftp://nowhere"});
    body["group"] = json!({});
    let repository = prepare(spec("raw", "hosted"), body, None).unwrap();
    assert_eq!(repository.attributes.maven, None);
    assert_eq!(repository.attributes.proxy, None);
    assert_eq!(repository.attributes.group, None);
}

#[test]
fn secrets_are_sealed_and_never_shown() {
    let spec = spec("yum", "proxy");
    let mut body = minimal(spec, "x");
    body["httpClient"]["authentication"] =
        json!({"type": "username", "username": "reader", "password": "secret-password"});
    body["yumSigning"] = json!({"keypair": PGP, "passphrase": "secret-passphrase"});
    let repository = prepare(spec, body, None).unwrap();
    let authentication = repository
        .attributes
        .http_client
        .as_ref()
        .unwrap()
        .authentication
        .as_ref()
        .unwrap();
    assert_eq!(authentication.password.as_deref(), Some("astrum:v1:15"));
    let document = repository.document("http://h");
    let text = document.to_string();
    for secret in ["secret-password", "secret-passphrase", "PGP", "astrum:v1:"] {
        assert!(!text.contains(secret), "{secret} in {text}");
    }
    assert_eq!(
        document["httpClient"]["authentication"],
        json!({"type": "username", "username": "reader", "ntlmHost": null, "ntlmDomain": null})
    );
    assert_eq!(document["yumSigning"], json!({}));
    let stored = repository.stored_attributes().to_string();
    assert!(stored.contains("astrum:v1:15"));
    assert!(!stored.contains("secret-password"));
}

#[test]
fn an_update_without_secrets_keeps_the_stored_ones() {
    let spec = spec("terraform", "hosted");
    let mut body = minimal(spec, "x");
    body["terraformSigning"] = json!({"keypair": PGP, "passphrase": "p"});
    let stored = prepare(spec, body, None).unwrap();
    let document = stored.document("http://h");
    let updated = prepare(spec, document.clone(), Some(&stored)).unwrap();
    assert_eq!(
        updated.attributes.terraform_signing,
        stored.attributes.terraform_signing
    );
    let mut removed = document;
    removed.as_object_mut().unwrap().remove("terraformSigning");
    let updated = prepare(spec, removed, Some(&stored)).unwrap();
    assert_eq!(updated.attributes.terraform_signing, None);
    let other = Spec::from_api("yum", "hosted").unwrap();
    let mut body = minimal(other, "x");
    body["yumSigning"] = json!({});
    let fresh = prepare(other, body, Some(&stored)).unwrap();
    assert_eq!(
        fresh.attributes.yum_signing, None,
        "secrets never cross formats"
    );
}

#[test]
fn apt_hosted_needs_a_signing_key() {
    let spec = spec("apt", "hosted");
    let mut body = minimal(spec, "x");
    body.as_object_mut().unwrap().remove("aptSigning");
    assert_eq!(
        prepare(spec, body.clone(), None).unwrap_err(),
        ["PARAMETER aptSigning: must not be null"]
    );
    body["aptSigning"] = json!({});
    assert_eq!(
        prepare(spec, body.clone(), None).unwrap_err(),
        ["PARAMETER keypair: must not be empty"]
    );
    let stored = prepare(spec, minimal(spec, "x"), None).unwrap();
    let kept = prepare(spec, body, Some(&stored)).unwrap();
    assert_eq!(kept.attributes.apt_signing, stored.attributes.apt_signing);
    let proxy = Spec::from_api("apt", "proxy").unwrap();
    let mut body = minimal(proxy, "x");
    body["aptSigning"] = json!({});
    let repository = prepare(proxy, body, None).unwrap();
    assert_eq!(repository.document("http://h")["aptSigning"], Value::Null);
}

#[test]
fn stored_attributes_read_back_the_same() {
    for spec in Spec::all() {
        let repository = prepare(spec, minimal(spec, "x"), None).unwrap();
        let stored = repository.stored_attributes();
        assert!(stored["storage"].get("blobStoreName").is_none());
        let back = Repository::from_stored(
            "x".to_owned(),
            spec.format.id,
            spec.kind.as_str(),
            true,
            Some("default".to_owned()),
            stored,
        )
        .unwrap();
        assert_eq!(back, repository, "{spec:?}");
    }
    assert!(
        Repository::from_stored("x".to_owned(), "pub", "hosted", true, None, json!({})).is_err()
    );
}

#[test]
fn summaries_show_the_remote_of_proxies_only() {
    let proxy = prepare(
        spec("npm", "proxy"),
        minimal(spec("npm", "proxy"), "npmjs"),
        None,
    )
    .unwrap();
    assert_eq!(
        proxy.summary("https://repo.example.org/"),
        json!({
            "name": "npmjs", "format": "npm", "type": "proxy",
            "url": "https://repo.example.org/repository/npmjs", "online": true,
            "attributes": {"proxy": {"remoteUrl": "https://upstream.example.org/"}}
        })
    );
    let hosted = prepare(
        spec("npm", "hosted"),
        minimal(spec("npm", "hosted"), "x"),
        None,
    )
    .unwrap();
    assert_eq!(hosted.summary("http://h")["attributes"], json!({}));
}
