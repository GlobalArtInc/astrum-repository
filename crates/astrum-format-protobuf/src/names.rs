use std::collections::BTreeMap;

use crate::connect::{Reply, Status};
use crate::proto;
use crate::proto::{label_ref, module_ref, owner_ref, resource_ref, scoped_label_ref};

pub const MAX_OWNER: usize = 48;
pub const MIN_MODULE: usize = 2;
pub const MAX_MODULE: usize = 100;
pub const MAX_LABEL: usize = 250;
pub const MAX_PATH: usize = 4096;
pub const MAX_FILE: usize = 4 * 1024 * 1024;
pub const MAX_FILES: usize = 20_000;
pub const LICENSE: &str = "LICENSE";
pub const DOCUMENTS: [&str; 3] = ["buf.md", "README.md", "README.markdown"];

pub fn valid_owner(name: &str) -> bool {
    let bytes = name.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= MAX_OWNER
        && bytes[0].is_ascii_lowercase()
        && bytes[bytes.len() - 1] != b'-'
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

pub fn valid_module(name: &str) -> bool {
    let bytes = name.as_bytes();
    (MIN_MODULE..=MAX_MODULE).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && (bytes[bytes.len() - 1].is_ascii_lowercase() || bytes[bytes.len() - 1].is_ascii_digit())
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))
}

pub fn commit_id(text: &str) -> Option<String> {
    (text.len() == 32 && text.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| text.to_ascii_lowercase())
}

pub fn valid_label(name: &str) -> Reply<()> {
    if name.is_empty() || name.chars().count() > MAX_LABEL {
        return Err(Status::invalid(format!(
            "a label name has 1 to {MAX_LABEL} characters"
        )));
    }
    if name.chars().any(char::is_control) {
        return Err(Status::invalid("a label name has no control characters"));
    }
    if commit_id(name).is_some() {
        return Err(Status::invalid("a label cannot be named like a commit id"));
    }
    Ok(())
}

pub fn encode_segment(name: &str) -> String {
    let mut encoded = String::with_capacity(name.len());
    for byte in name.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                encoded.push(byte as char)
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

pub fn valid_file_path(path: &str) -> Reply<()> {
    if path.is_empty() || path.len() > MAX_PATH {
        return Err(Status::invalid(format!(
            "a file path has 1 to {MAX_PATH} bytes"
        )));
    }
    if path.contains('\\') {
        return Err(Status::invalid(format!(
            "the file path {path:?} has a backslash"
        )));
    }
    if path.chars().any(|c| c.is_control()) {
        return Err(Status::invalid(format!(
            "the file path {path:?} has control characters"
        )));
    }
    if path.contains('%') {
        return Err(Status::invalid(format!(
            "the file path {path:?} has a percent sign"
        )));
    }
    if path.starts_with('/') || path.ends_with('/') {
        return Err(Status::invalid(format!(
            "the file path {path:?} must be relative"
        )));
    }
    if path
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(Status::invalid(format!(
            "the file path {path:?} has an empty or dot segment"
        )));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Proto,
    Doc,
    License,
}

pub fn classify(path: &str) -> Option<Kind> {
    if path == LICENSE {
        return Some(Kind::License);
    }
    if DOCUMENTS.contains(&path) {
        return Some(Kind::Doc);
    }
    path.ends_with(".proto").then_some(Kind::Proto)
}

pub fn kind_of(file_type: i32) -> Option<Kind> {
    match proto::FileType::try_from(file_type).ok()? {
        proto::FileType::Proto => Some(Kind::Proto),
        proto::FileType::Doc => Some(Kind::Doc),
        proto::FileType::License => Some(Kind::License),
        proto::FileType::Unspecified => None,
    }
}

pub fn module_files(files: Vec<proto::File>) -> Reply<BTreeMap<String, Vec<u8>>> {
    if files.len() > MAX_FILES * 2 {
        return Err(Status::invalid("the upload has too many files"));
    }
    let mut kept = BTreeMap::new();
    let mut document: Option<(usize, proto::File)> = None;
    let mut seen = std::collections::BTreeSet::new();
    for file in files {
        valid_file_path(&file.path)?;
        if !seen.insert(file.path.clone()) {
            return Err(Status::invalid(format!(
                "the file {} is listed twice",
                file.path
            )));
        }
        if file.content.len() > MAX_FILE {
            return Err(Status::invalid(format!(
                "the file {} is larger than {MAX_FILE} bytes",
                file.path
            )));
        }
        match classify(&file.path) {
            Some(Kind::Doc) => {
                let rank = DOCUMENTS
                    .iter()
                    .position(|name| *name == file.path)
                    .unwrap_or(usize::MAX);
                if document.as_ref().is_none_or(|(best, _)| rank < *best) {
                    document = Some((rank, file));
                }
            }
            Some(_) => {
                kept.insert(file.path, file.content);
            }
            None => {}
        }
    }
    if let Some((_, file)) = document {
        kept.insert(file.path, file.content);
    }
    if kept.len() > MAX_FILES {
        return Err(Status::invalid("the upload has too many files"));
    }
    if !kept.keys().any(|path| classify(path) == Some(Kind::Proto)) {
        return Err(Status::invalid("a module needs at least one .proto file"));
    }
    Ok(kept)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ref {
    Id(String),
    Module {
        owner: String,
        module: String,
    },
    Label {
        owner: String,
        module: String,
        label: String,
    },
    Named {
        owner: String,
        module: String,
        reference: String,
    },
}

fn identifier(id: &str) -> Reply<String> {
    commit_id(id).ok_or_else(|| Status::invalid(format!("{id:?} is not a registry id")))
}

fn names(owner: &str, module: &str) -> Reply<()> {
    if !valid_owner(owner) {
        return Err(Status::invalid(format!("{owner:?} is not an owner name")));
    }
    if !valid_module(module) {
        return Err(Status::invalid(format!("{module:?} is not a module name")));
    }
    Ok(())
}

pub fn resource_ref(reference: Option<&proto::ResourceRef>) -> Reply<Ref> {
    let value = reference
        .and_then(|reference| reference.value.as_ref())
        .ok_or_else(|| Status::invalid("a resource reference is required"))?;
    match value {
        resource_ref::Value::Id(id) => Ok(Ref::Id(identifier(id)?)),
        resource_ref::Value::Name(name) => {
            names(&name.owner, &name.module)?;
            let owner = name.owner.clone();
            let module = name.module.clone();
            Ok(match &name.child {
                Some(resource_ref::name::Child::LabelName(label)) if !label.is_empty() => {
                    Ref::Label {
                        owner,
                        module,
                        label: label.clone(),
                    }
                }
                Some(resource_ref::name::Child::Ref(reference)) if !reference.is_empty() => {
                    Ref::Named {
                        owner,
                        module,
                        reference: reference.clone(),
                    }
                }
                _ => Ref::Module { owner, module },
            })
        }
    }
}

pub fn module_ref(reference: Option<&proto::ModuleRef>) -> Reply<Ref> {
    let value = reference
        .and_then(|reference| reference.value.as_ref())
        .ok_or_else(|| Status::invalid("a module reference is required"))?;
    match value {
        module_ref::Value::Id(id) => Ok(Ref::Id(identifier(id)?)),
        module_ref::Value::Name(name) => {
            names(&name.owner, &name.module)?;
            Ok(Ref::Module {
                owner: name.owner.clone(),
                module: name.module.clone(),
            })
        }
    }
}

pub fn label_ref(reference: Option<&proto::LabelRef>) -> Reply<Ref> {
    let value = reference
        .and_then(|reference| reference.value.as_ref())
        .ok_or_else(|| Status::invalid("a label reference is required"))?;
    match value {
        label_ref::Value::Id(id) => Ok(Ref::Id(identifier(id)?)),
        label_ref::Value::Name(name) => {
            names(&name.owner, &name.module)?;
            if name.label.is_empty() {
                return Err(Status::invalid("a label reference needs a label name"));
            }
            Ok(Ref::Label {
                owner: name.owner.clone(),
                module: name.module.clone(),
                label: name.label.clone(),
            })
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerKey {
    Id(String),
    Name(String),
}

pub fn owner_ref(reference: &proto::OwnerRef) -> Reply<OwnerKey> {
    match reference.value.as_ref() {
        Some(owner_ref::Value::Id(id)) => Ok(OwnerKey::Id(identifier(id)?)),
        Some(owner_ref::Value::Name(name)) if !name.is_empty() => Ok(OwnerKey::Name(name.clone())),
        _ => Err(Status::invalid("an owner reference is required")),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelKey {
    Id(String),
    Name(String),
}

pub fn scoped_label_ref(reference: &proto::ScopedLabelRef) -> Reply<LabelKey> {
    match reference.value.as_ref() {
        Some(scoped_label_ref::Value::Id(id)) => Ok(LabelKey::Id(identifier(id)?)),
        Some(scoped_label_ref::Value::Name(name)) => {
            valid_label(name)?;
            Ok(LabelKey::Name(name.clone()))
        }
        None => Err(Status::invalid("a label reference is required")),
    }
}

pub fn imports(source: &str) -> Vec<String> {
    #[derive(PartialEq)]
    enum Token {
        Word(String),
        Text(String),
        Punct(char),
    }
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let c = chars[index];
        if c.is_whitespace() {
            index += 1;
        } else if c == '/' && chars.get(index + 1) == Some(&'/') {
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
        } else if c == '/' && chars.get(index + 1) == Some(&'*') {
            index += 2;
            while index < chars.len()
                && !(chars[index] == '*' && chars.get(index + 1) == Some(&'/'))
            {
                index += 1;
            }
            index += 2;
        } else if c == '"' || c == '\'' {
            let quote = c;
            let mut text = String::new();
            index += 1;
            while index < chars.len() && chars[index] != quote {
                if chars[index] == '\\' && index + 1 < chars.len() {
                    index += 1;
                }
                text.push(chars[index]);
                index += 1;
            }
            index += 1;
            tokens.push(Token::Text(text));
        } else if c.is_alphanumeric() || c == '_' || c == '.' {
            let mut word = String::new();
            while index < chars.len()
                && (chars[index].is_alphanumeric() || chars[index] == '_' || chars[index] == '.')
            {
                word.push(chars[index]);
                index += 1;
            }
            tokens.push(Token::Word(word));
        } else {
            tokens.push(Token::Punct(c));
            index += 1;
        }
    }
    let mut found = Vec::new();
    let mut depth = 0usize;
    let mut position = 0;
    while position < tokens.len() {
        match &tokens[position] {
            Token::Punct('{') => depth += 1,
            Token::Punct('}') => depth = depth.saturating_sub(1),
            Token::Word(word) if word == "import" && depth == 0 => {
                let mut next = position + 1;
                if let Some(Token::Word(modifier)) = tokens.get(next)
                    && (modifier == "public" || modifier == "weak")
                {
                    next += 1;
                }
                if let (Some(Token::Text(path)), Some(Token::Punct(';'))) =
                    (tokens.get(next), tokens.get(next + 1))
                {
                    found.push(path.clone());
                    position = next + 1;
                }
            }
            _ => {}
        }
        position += 1;
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{ResourceRef, resource_ref};

    fn file(path: &str, content: &str) -> proto::File {
        proto::File {
            path: path.to_owned(),
            content: content.as_bytes().to_vec(),
        }
    }

    fn named(child: Option<resource_ref::name::Child>) -> ResourceRef {
        ResourceRef {
            value: Some(resource_ref::Value::Name(resource_ref::Name {
                owner: "acme".to_owned(),
                module: "greeter".to_owned(),
                child,
            })),
        }
    }

    #[test]
    fn owners_and_modules_follow_the_naming_rules() {
        assert!(valid_owner("acme"));
        assert!(valid_owner("a"));
        assert!(valid_owner("acme-2"));
        assert!(!valid_owner(""));
        assert!(!valid_owner("Acme"));
        assert!(!valid_owner("2acme"));
        assert!(!valid_owner("acme-"));
        assert!(!valid_owner(&"a".repeat(49)));
        assert!(valid_owner(&"a".repeat(48)));
        assert!(valid_module("greeter"));
        assert!(valid_module("ab"));
        assert!(valid_module("my_module-2"));
        assert!(!valid_module("a"));
        assert!(!valid_module("Greeter"));
        assert!(!valid_module("greeter-"));
        assert!(!valid_module("1greeter"));
        assert!(!valid_module(&"a".repeat(101)));
    }

    #[test]
    fn labels_cannot_look_like_commit_ids() {
        assert!(valid_label("main").is_ok());
        assert!(valid_label("release/v1.2").is_ok());
        assert!(valid_label("").is_err());
        assert!(valid_label("a\nb").is_err());
        assert!(valid_label(&"x".repeat(251)).is_err());
        assert!(valid_label("0123456789abcdef0123456789abcdef").is_err());
        assert_eq!(encode_segment("release/v1 2"), "release%2Fv1%202");
        assert_eq!(encode_segment("main"), "main");
    }

    #[test]
    fn file_paths_are_relative_and_clean() {
        for good in ["a.proto", "a/b/c.proto", "LICENSE", "a b/c.proto"] {
            assert!(valid_file_path(good).is_ok(), "{good}");
        }
        for bad in [
            "",
            "/a.proto",
            "a/",
            "a//b.proto",
            "./a.proto",
            "a/../b.proto",
            "a\\b.proto",
            "a%2Fb.proto",
            "a\u{7}.proto",
        ] {
            assert!(valid_file_path(bad).is_err(), "{bad:?}");
        }
        assert!(valid_file_path(&"a".repeat(MAX_PATH + 1)).is_err());
    }

    #[test]
    fn module_files_keep_protos_the_license_and_one_document() {
        let kept = module_files(vec![
            file("a/v1/a.proto", "x"),
            file("LICENSE", "MIT"),
            file("README.md", "readme"),
            file("buf.md", "buf"),
            file("README.markdown", "markdown"),
            file("buf.yaml", "version: v2"),
            file("docs/README.md", "nested"),
            file("sub/LICENSE", "nested"),
        ])
        .unwrap();
        assert_eq!(
            kept.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["LICENSE", "a/v1/a.proto", "buf.md"]
        );
        let kept = module_files(vec![
            file("a.proto", "x"),
            file("README.markdown", "m"),
            file("README.md", "r"),
        ])
        .unwrap();
        assert!(kept.contains_key("README.md") && !kept.contains_key("README.markdown"));
    }

    #[test]
    fn module_files_are_refused_when_wrong() {
        assert!(module_files(vec![file("LICENSE", "x")]).is_err());
        assert!(module_files(vec![]).is_err());
        assert!(module_files(vec![file("a.proto", "x"), file("a.proto", "y")]).is_err());
        assert!(module_files(vec![file("../a.proto", "x")]).is_err());
        let big = proto::File {
            path: "a.proto".to_owned(),
            content: vec![0; MAX_FILE + 1],
        };
        assert!(module_files(vec![big]).is_err());
    }

    #[test]
    fn resource_refs_resolve_by_id_or_name() {
        let id = "0123456789ABCDEF0123456789abcdef";
        let by_id = ResourceRef {
            value: Some(resource_ref::Value::Id(id.to_owned())),
        };
        assert_eq!(
            resource_ref(Some(&by_id)).unwrap(),
            Ref::Id(id.to_ascii_lowercase())
        );
        assert_eq!(
            resource_ref(Some(&named(None))).unwrap(),
            Ref::Module {
                owner: "acme".to_owned(),
                module: "greeter".to_owned()
            }
        );
        assert_eq!(
            resource_ref(Some(&named(Some(resource_ref::name::Child::LabelName(
                "main".to_owned()
            )))))
            .unwrap(),
            Ref::Label {
                owner: "acme".to_owned(),
                module: "greeter".to_owned(),
                label: "main".to_owned()
            }
        );
        assert_eq!(
            resource_ref(Some(&named(Some(resource_ref::name::Child::Ref(
                "v1".to_owned()
            )))))
            .unwrap(),
            Ref::Named {
                owner: "acme".to_owned(),
                module: "greeter".to_owned(),
                reference: "v1".to_owned()
            }
        );
    }

    #[test]
    fn an_empty_child_means_the_module() {
        for child in [
            resource_ref::name::Child::LabelName(String::new()),
            resource_ref::name::Child::Ref(String::new()),
        ] {
            assert!(matches!(
                resource_ref(Some(&named(Some(child)))).unwrap(),
                Ref::Module { .. }
            ));
        }
    }

    #[test]
    fn bad_refs_are_invalid_arguments() {
        assert!(resource_ref(None).is_err());
        assert!(resource_ref(Some(&ResourceRef { value: None })).is_err());
        let bad_id = ResourceRef {
            value: Some(resource_ref::Value::Id("nope".to_owned())),
        };
        assert!(resource_ref(Some(&bad_id)).is_err());
        let no_module = ResourceRef {
            value: Some(resource_ref::Value::Name(resource_ref::Name {
                owner: "acme".to_owned(),
                module: String::new(),
                child: None,
            })),
        };
        assert!(resource_ref(Some(&no_module)).is_err());
    }

    #[test]
    fn imports_are_found_at_the_top_level_only() {
        let line = "/".repeat(2);
        let open = "/*";
        let close = "*/";
        let source = [
            "syntax = \"proto3\";".to_owned(),
            format!("{line} import \"commented.proto\";"),
            "package acme.v1;".to_owned(),
            "import \"a/v1/a.proto\";".to_owned(),
            "import public 'b/v1/b.proto';".to_owned(),
            format!("{open} import \"block.proto\"; {close}"),
            "import weak \"c/v1/c.proto\";".to_owned(),
            "message M {".to_owned(),
            "  string import = 1;".to_owned(),
            "  string text = 2 [default = \"import \\\"inner.proto\\\";\"];".to_owned(),
            "}".to_owned(),
            "import \"d/v1/d.proto\" ;".to_owned(),
        ]
        .join("\n");
        assert_eq!(
            imports(&source),
            vec![
                "a/v1/a.proto",
                "b/v1/b.proto",
                "c/v1/c.proto",
                "d/v1/d.proto"
            ]
        );
        assert!(imports("syntax = \"proto3\";").is_empty());
    }
}
