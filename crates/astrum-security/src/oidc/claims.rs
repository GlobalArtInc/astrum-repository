use std::collections::BTreeMap;

use serde_json::{Map, Value};

use super::OidcError;

pub const DEFAULT_USERNAME: &str = "preferred_username";
pub const DEFAULT_FIRST_NAME: &str = "given_name";
pub const DEFAULT_LAST_NAME: &str = "family_name";
pub const DEFAULT_EMAIL: &str = "email";

pub fn find<'a>(claims: &'a Map<String, Value>, path: &str) -> Option<&'a Value> {
    let name = path.trim();
    if name.is_empty() {
        return None;
    }
    if let Some(value) = claims.get(name) {
        return Some(value);
    }
    let mut parts = name.split('.');
    let mut current = claims.get(parts.next()?)?;
    for part in parts {
        current = current.as_object()?.get(part)?;
    }
    Some(current)
}

fn scalar(value: &Value) -> Option<String> {
    let text = match value {
        Value::String(text) => text.trim().to_owned(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        _ => return None,
    };
    (!text.is_empty()).then_some(text)
}

pub fn string(claims: &Map<String, Value>, path: &str) -> Option<String> {
    find(claims, path).and_then(scalar)
}

pub fn strings(claims: &Map<String, Value>, path: &str) -> Vec<String> {
    match find(claims, path) {
        Some(Value::Array(items)) => items.iter().filter_map(scalar).collect(),
        Some(value) => scalar(value).into_iter().collect(),
        None => Vec::new(),
    }
}

pub fn claim_or_default(
    claims: &Map<String, Value>,
    configured: Option<&str>,
    fallback: &str,
) -> Option<String> {
    match configured.map(str::trim).filter(|claim| !claim.is_empty()) {
        Some(claim) => string(claims, claim),
        None => string(claims, fallback),
    }
}

pub fn user_id(claims: &Map<String, Value>, configured: Option<&str>) -> Result<String, OidcError> {
    if let Some(claim) = configured.map(str::trim).filter(|claim| !claim.is_empty()) {
        return string(claims, claim).ok_or_else(|| {
            OidcError::denied(format!(
                "The identity provider did not return the username claim '{claim}'"
            ))
        });
    }
    string(claims, DEFAULT_USERNAME)
        .or_else(|| string(claims, "sub"))
        .ok_or_else(|| OidcError::failed("The identity provider did not return a subject"))
}

pub fn check_exact_match(
    claims: &Map<String, Value>,
    required: &BTreeMap<String, String>,
) -> Result<(), OidcError> {
    for (name, wanted) in required {
        let matches = match find(claims, name) {
            Some(Value::Array(items)) => {
                items.iter().filter_map(scalar).any(|item| item == *wanted)
            }
            Some(value) => scalar(value).is_some_and(|actual| actual == *wanted),
            None => false,
        };
        if !matches {
            return Err(OidcError::denied(format!(
                "Access denied: claim '{name}' does not have the required value"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn claims() -> Map<String, Value> {
        json!({
            "sub": "u-1",
            "preferred_username": " rita ",
            "realm_access": {"roles": ["devs", "ops", {"nested": true}]},
            "groups": "single",
            "hd": "example.com",
            "flags": [1, true],
            "dotted.name": "direct"
        })
        .as_object()
        .unwrap()
        .clone()
    }

    #[test]
    fn claims_resolve_by_name_or_path() {
        let claims = claims();
        assert_eq!(
            string(&claims, "preferred_username").as_deref(),
            Some("rita")
        );
        assert_eq!(strings(&claims, "realm_access.roles"), ["devs", "ops"]);
        assert_eq!(strings(&claims, "groups"), ["single"]);
        assert_eq!(strings(&claims, "flags"), ["1", "true"]);
        assert_eq!(string(&claims, "dotted.name").as_deref(), Some("direct"));
        assert_eq!(string(&claims, "realm_access"), None);
        assert!(strings(&claims, "missing").is_empty());
    }

    #[test]
    fn the_user_id_falls_back_to_the_subject() {
        let claims = claims();
        assert_eq!(user_id(&claims, None).unwrap(), "rita");
        assert_eq!(user_id(&claims, Some("sub")).unwrap(), "u-1");
        assert!(user_id(&claims, Some("email")).is_err());
        let bare = json!({"sub": "only"}).as_object().unwrap().clone();
        assert_eq!(user_id(&bare, Some(" ")).unwrap(), "only");
    }

    #[test]
    fn exact_matches_accept_one_of_many() {
        let claims = claims();
        let mut required = BTreeMap::new();
        required.insert("hd".to_owned(), "example.com".to_owned());
        required.insert("realm_access.roles".to_owned(), "ops".to_owned());
        check_exact_match(&claims, &required).unwrap();
        required.insert("hd".to_owned(), "other.org".to_owned());
        assert!(check_exact_match(&claims, &required).is_err());
    }
}
