pub const DEFAULT_REVISION: &str = "main";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepoType {
    Model,
    Dataset,
    Space,
}

impl RepoType {
    pub fn kind(self) -> &'static str {
        match self {
            RepoType::Model => "model",
            RepoType::Dataset => "dataset",
            RepoType::Space => "space",
        }
    }

    pub fn plural(self) -> &'static str {
        match self {
            RepoType::Model => "models",
            RepoType::Dataset => "datasets",
            RepoType::Space => "spaces",
        }
    }

    pub fn url_prefix(self) -> &'static str {
        match self {
            RepoType::Model => "",
            RepoType::Dataset => "datasets/",
            RepoType::Space => "spaces/",
        }
    }

    pub fn from_plural(value: &str) -> Option<Self> {
        [RepoType::Model, RepoType::Dataset, RepoType::Space]
            .into_iter()
            .find(|kind| kind.plural() == value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Info {
        kind: RepoType,
        repo: String,
        revision: Option<String>,
    },
    Tree {
        kind: RepoType,
        repo: String,
        revision: String,
        directory: String,
    },
    PathsInfo {
        kind: RepoType,
        repo: String,
        revision: String,
    },
    File {
        kind: RepoType,
        repo: String,
        revision: String,
        file: String,
    },
    Stored,
}

pub fn is_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn dots(segment: &str) -> bool {
    segment == "." || segment == ".."
}

fn repo_id(segments: &[&str]) -> bool {
    !segments.is_empty()
        && segments.len() <= 2
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment.len() <= 96
                && !dots(segment)
                && segment
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'-'))
        })
}

fn clean(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && value.split('/').all(|segment| {
            !segment.is_empty() && !dots(segment) && !segment.chars().any(char::is_control)
        })
}

fn revision_length(segments: &[&str]) -> usize {
    match segments {
        ["refs", "pr" | "convert", _, ..] => 3,
        _ => 1,
    }
}

fn keyword(segments: &[&str], keywords: &[&str]) -> Option<usize> {
    [2, 1]
        .into_iter()
        .find(|&at| segments.len() > at && keywords.contains(&segments[at]))
}

pub fn parse(path: &str) -> Option<Request> {
    let trimmed = path.trim_start_matches('/').trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let segments: Vec<&str> = trimmed.split('/').collect();
    if segments.contains(&"") {
        return None;
    }
    if segments[0] == "api" {
        return parse_api(&segments[1..]);
    }
    parse_resolve(&segments).or_else(|| Some(parse_stored(&segments)))
}

fn parse_api(rest: &[&str]) -> Option<Request> {
    if rest.len() < 2 {
        return None;
    }
    let kind = RepoType::from_plural(rest[0])?;
    let tail = &rest[1..];
    let Some(at) = keyword(tail, &["revision", "tree", "paths-info"]) else {
        return (tail.len() <= 2 && repo_id(tail)).then(|| Request::Info {
            kind,
            repo: tail.join("/"),
            revision: None,
        });
    };
    let (repo, after) = (&tail[..at], &tail[at + 1..]);
    if !repo_id(repo) || after.is_empty() {
        return None;
    }
    let repo = repo.join("/");
    let consumed = revision_length(after).min(after.len());
    let revision = after[..consumed].join("/");
    if !clean(&revision, 255) {
        return None;
    }
    match tail[at] {
        "revision" => {
            let revision = after.join("/");
            clean(&revision, 255).then_some(Request::Info {
                kind,
                repo,
                revision: Some(revision),
            })
        }
        "tree" => {
            let directory = after[consumed..].join("/");
            (directory.is_empty() || clean(&directory, 1024)).then_some(Request::Tree {
                kind,
                repo,
                revision,
                directory,
            })
        }
        _ => (consumed == after.len()).then_some(Request::PathsInfo {
            kind,
            repo,
            revision,
        }),
    }
}

fn parse_resolve(segments: &[&str]) -> Option<Request> {
    let (kind, rest) = match segments[0] {
        "datasets" => (RepoType::Dataset, &segments[1..]),
        "spaces" => (RepoType::Space, &segments[1..]),
        "models" => return None,
        _ => (RepoType::Model, segments),
    };
    let at = keyword(rest, &["resolve"])?;
    let (repo, after) = (&rest[..at], &rest[at + 1..]);
    if !repo_id(repo) || after.len() < 2 {
        return None;
    }
    let consumed = revision_length(after).min(after.len() - 1);
    let revision = after[..consumed].join("/");
    let file = after[consumed..].join("/");
    (clean(&revision, 255) && clean(&file, 1024)).then(|| Request::File {
        kind,
        repo: repo.join("/"),
        revision,
        file,
    })
}

fn parse_stored(segments: &[&str]) -> Request {
    if let Some(kind) = RepoType::from_plural(segments[0]) {
        let rest = &segments[1..];
        for commit in [2, 1] {
            if rest.len() > commit + 1 && is_commit(rest[commit]) && repo_id(&rest[..commit]) {
                let file = rest[commit + 1..].join("/");
                if clean(&file, 1024) {
                    return Request::File {
                        kind,
                        repo: rest[..commit].join("/"),
                        revision: rest[commit].to_owned(),
                        file,
                    };
                }
            }
        }
    }
    Request::Stored
}

pub fn revision_path(kind: RepoType, repo: &str, revision: &str) -> String {
    format!("/{}/{repo}/{revision}.json", kind.plural())
}

pub fn file_path(kind: RepoType, repo: &str, commit: &str, file: &str) -> String {
    format!("/{}/{repo}/{commit}/{file}", kind.plural())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hub_paths_are_recognised() {
        assert_eq!(
            parse("/api/models/gpt2"),
            Some(Request::Info {
                kind: RepoType::Model,
                repo: "gpt2".to_owned(),
                revision: None
            })
        );
        assert_eq!(
            parse("/api/datasets/acme/set/revision/v1.0"),
            Some(Request::Info {
                kind: RepoType::Dataset,
                repo: "acme/set".to_owned(),
                revision: Some("v1.0".to_owned())
            })
        );
        assert_eq!(
            parse("/api/models/acme/bert/tree/main/sub/dir"),
            Some(Request::Tree {
                kind: RepoType::Model,
                repo: "acme/bert".to_owned(),
                revision: "main".to_owned(),
                directory: "sub/dir".to_owned()
            })
        );
        assert_eq!(
            parse("/api/models/acme/bert/paths-info/refs/pr/1"),
            Some(Request::PathsInfo {
                kind: RepoType::Model,
                repo: "acme/bert".to_owned(),
                revision: "refs/pr/1".to_owned()
            })
        );
        assert_eq!(
            parse("/acme/bert/resolve/main/onnx/model.onnx"),
            Some(Request::File {
                kind: RepoType::Model,
                repo: "acme/bert".to_owned(),
                revision: "main".to_owned(),
                file: "onnx/model.onnx".to_owned()
            })
        );
        assert_eq!(
            parse("/datasets/acme/set/resolve/refs/convert/parquet/a.parquet"),
            Some(Request::File {
                kind: RepoType::Dataset,
                repo: "acme/set".to_owned(),
                revision: "refs/convert/parquet".to_owned(),
                file: "a.parquet".to_owned()
            })
        );
        let commit = "f171d7baecaf37b5da5a3616d8833b9969753535";
        assert_eq!(
            parse(&format!("/models/acme/bert/{commit}/config.json")),
            Some(Request::File {
                kind: RepoType::Model,
                repo: "acme/bert".to_owned(),
                revision: commit.to_owned(),
                file: "config.json".to_owned()
            })
        );
        assert_eq!(parse("/models/acme/bert/main.json"), Some(Request::Stored));
        assert_eq!(parse("/api/models/../x"), None);
        assert_eq!(parse("/api/unknown/x"), None);
    }
}
