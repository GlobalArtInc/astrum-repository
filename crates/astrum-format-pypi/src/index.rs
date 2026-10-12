use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::name;

pub type Hashes = BTreeMap<String, String>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Project {
    pub name: String,
    pub files: Vec<File>,
    pub versions: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct File {
    pub filename: String,
    pub url: String,
    pub version: Option<String>,
    pub hashes: Hashes,
    pub requires_python: Option<String>,
    pub yanked: Option<String>,
    pub core_metadata: Option<Hashes>,
    pub size: Option<u64>,
    pub upload_time: Option<String>,
}

impl File {
    pub fn fragment(&self) -> Option<(&str, &str)> {
        ["sha256", "sha512", "sha384", "sha224", "sha1", "md5"]
            .iter()
            .find_map(|algorithm| {
                self.hashes
                    .get_key_value(*algorithm)
                    .map(|(name, value)| (name.as_str(), value.as_str()))
            })
            .or_else(|| {
                self.hashes
                    .iter()
                    .next()
                    .map(|(name, value)| (name.as_str(), value.as_str()))
            })
    }
}

fn version_from_url(project: &str, url: &str, filename: &str) -> Option<String> {
    let url = url::Url::parse(url).ok()?;
    let segments: Vec<String> = url
        .path_segments()?
        .map(|segment| {
            percent_encoding::percent_decode_str(segment)
                .decode_utf8_lossy()
                .into_owned()
        })
        .collect();
    match segments.as_slice() {
        [.., packages, name, version, file]
            if packages == "packages" && file == filename && name::normalize(name) == project =>
        {
            Some(version.clone())
        }
        _ => None,
    }
}

impl File {
    pub fn local_path(&self, project: &str) -> String {
        let version = self
            .version
            .clone()
            .or_else(|| name::version_of(project, &self.filename))
            .or_else(|| version_from_url(project, &self.url, &self.filename))
            .filter(|version| name::is_safe_segment(version))
            .unwrap_or_else(|| "unknown".to_owned());
        format!("/packages/{project}/{version}/{}", self.filename)
    }
}

impl Project {
    pub fn localize(mut self) -> Self {
        let project = name::normalize(&self.name);
        for file in &mut self.files {
            file.url = file.local_path(&project);
            if file.version.is_none() {
                file.version = file.url.split('/').nth(3).map(str::to_owned);
            }
        }
        self
    }

    pub fn find(&self, filename: &str) -> Option<&File> {
        self.files.iter().find(|file| file.filename == filename)
    }

    pub fn all_versions(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.versions
            .iter()
            .cloned()
            .chain(self.files.iter().filter_map(|file| file.version.clone()))
            .filter(|version| seen.insert(version.clone()))
            .collect()
    }
}

pub fn merge(pages: Vec<Project>) -> Option<Project> {
    let mut pages = pages.into_iter();
    let mut merged = pages.next()?;
    let mut seen: HashSet<String> = merged
        .files
        .iter()
        .map(|file| file.filename.clone())
        .collect();
    for page in pages {
        merged.versions.extend(page.versions);
        for file in page.files {
            if seen.insert(file.filename.clone()) {
                merged.files.push(file);
            }
        }
    }
    merged
        .files
        .sort_by(|left, right| left.filename.cmp(&right.filename));
    let mut versions = HashSet::new();
    merged
        .versions
        .retain(|version| versions.insert(version.clone()));
    Some(merged)
}

pub fn merge_names(lists: Vec<Vec<String>>) -> Vec<String> {
    let mut merged = BTreeSet::new();
    for list in lists {
        merged.extend(list.into_iter().map(|project| name::normalize(&project)));
    }
    merged.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(filename: &str, sha: &str) -> File {
        File {
            filename: filename.to_owned(),
            url: format!("https://files.example/{filename}"),
            hashes: [("sha256".to_owned(), sha.to_owned())].into(),
            ..Default::default()
        }
    }

    #[test]
    fn the_first_page_wins_on_duplicates() {
        let hosted = Project {
            name: "six".to_owned(),
            files: vec![
                file("six-1.17.0.tar.gz", "aa"),
                file("six-99.0.tar.gz", "bb"),
            ],
            versions: vec![],
        };
        let remote = Project {
            name: "six".to_owned(),
            files: vec![
                file("six-1.0.0.tar.gz", "cc"),
                file("six-1.17.0.tar.gz", "dd"),
            ],
            versions: vec!["1.0.0".to_owned(), "1.17.0".to_owned()],
        };
        let merged = merge(vec![hosted, remote]).unwrap();
        let names: Vec<_> = merged.files.iter().map(|f| f.filename.as_str()).collect();
        assert_eq!(
            names,
            ["six-1.0.0.tar.gz", "six-1.17.0.tar.gz", "six-99.0.tar.gz"]
        );
        assert_eq!(
            merged.find("six-1.17.0.tar.gz").unwrap().hashes["sha256"],
            "aa"
        );
        assert_eq!(merged.versions, ["1.0.0", "1.17.0"]);
        assert_eq!(merge(vec![]), None);
    }

    #[test]
    fn localized_files_point_into_the_repository() {
        let project = Project {
            name: "Foo_Bar".to_owned(),
            files: vec![file("foo_bar-1.0-py3-none-any.whl", "aa")],
            versions: vec![],
        }
        .localize();
        assert_eq!(
            project.files[0].url,
            "/packages/foo-bar/1.0/foo_bar-1.0-py3-none-any.whl"
        );
        assert_eq!(project.all_versions(), ["1.0"]);
    }

    #[test]
    fn versions_fall_back_to_the_upstream_layout() {
        let odd = File {
            filename: "other-9.9.tar.gz".to_owned(),
            url: "http://h/repository/r/packages/Foo_Bar/2.0/other-9.9.tar.gz".to_owned(),
            ..Default::default()
        };
        assert_eq!(
            odd.local_path("foo-bar"),
            "/packages/foo-bar/2.0/other-9.9.tar.gz"
        );
        let unknown = File {
            url: "https://files.example/ab/cd/other-9.9.tar.gz".to_owned(),
            ..odd.clone()
        };
        assert_eq!(
            unknown.local_path("foo-bar"),
            "/packages/foo-bar/unknown/other-9.9.tar.gz"
        );
        let wheel = File {
            filename: "six-1.0-py3-none-any.whl".to_owned(),
            url: "https://files.example/packages/six/9.9/six-1.0-py3-none-any.whl".to_owned(),
            ..Default::default()
        };
        assert_eq!(
            wheel.local_path("six"),
            "/packages/six/1.0/six-1.0-py3-none-any.whl"
        );
        let localized = Project {
            name: "Foo_Bar".to_owned(),
            files: vec![odd],
            versions: vec![],
        }
        .localize();
        assert_eq!(localized.files[0].version.as_deref(), Some("2.0"));
    }

    #[test]
    fn fragments_prefer_sha256() {
        let mut both = file("a-1.tar.gz", "aa");
        both.hashes.insert("md5".to_owned(), "mm".to_owned());
        assert_eq!(both.fragment(), Some(("sha256", "aa")));
        let mut md5 = File::default();
        md5.hashes.insert("md5".to_owned(), "mm".to_owned());
        assert_eq!(md5.fragment(), Some(("md5", "mm")));
        assert_eq!(File::default().fragment(), None);
    }

    #[test]
    fn names_merge_normalized_and_sorted() {
        assert_eq!(
            merge_names(vec![
                vec!["Six".to_owned(), "b".to_owned()],
                vec!["six".to_owned(), "A_x".to_owned()]
            ]),
            ["a-x", "b", "six"]
        );
    }
}
