use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;

use crate::hosted::{MAX_CONTROL_PREFIX, describe_package};
use crate::{DEB, METADATA, deb};

const PACKAGE_SUFFIXES: [&str; 3] = [".deb", ".udeb", ".ddeb"];

fn generated(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or_default();
    path.starts_with("/dists/")
        || path.contains("/by-hash/")
        || matches!(file, "Release" | "InRelease" | "Release.gpg")
        || file.starts_with("Packages")
}

pub fn plan(_repository: RepositoryType, path: &str, kind: &str) -> Plan {
    if kind == METADATA {
        return Plan::Derived;
    }
    if PACKAGE_SUFFIXES.iter().any(|suffix| path.ends_with(suffix)) {
        return match kind == DEB {
            true => Plan::Import,
            false => Plan::Skip(format!(
                "{path} is a package file of kind {kind}, not {DEB}"
            )),
        };
    }
    match generated(path) {
        true => Plan::Derived,
        false => {
            Plan::Skip("the path is neither a Debian package nor generated metadata".to_owned())
        }
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    if file.cached() {
        return Err("only hosted repositories take Debian packages".to_owned());
    }
    let data = file
        .head(MAX_CONTROL_PREFIX.min(file.size))
        .await
        .map_err(|error| error.to_string())?;
    let control = tokio::task::spawn_blocking(move || deb::read_control(&data))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let expected = deb::pool_path(&control);
    if expected != file.path {
        return Err(format!(
            "the bundle path {} does not match {expected}, where the upload would store the package",
            file.path
        ));
    }
    let mut put = describe_package(file.put(expected), &control);
    put.check_content_type = false;
    Ok(Some(Described::source(put)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packages_are_imported_and_generated_files_are_derived() {
        let hosted = RepositoryType::Hosted;
        assert_eq!(
            plan(hosted, "/pool/h/hello/hello_1_all.deb", DEB),
            Plan::Import
        );
        assert_eq!(
            plan(hosted, "/pool/h/hello/hello_1_all.udeb", DEB),
            Plan::Import
        );
        for path in [
            "/dists/bookworm/Release",
            "/dists/bookworm/InRelease",
            "/dists/bookworm/Release.gpg",
            "/dists/bookworm/main/binary-amd64/Packages.xz",
            "/dists/bookworm/main/binary-amd64/by-hash/SHA256/abc",
            "/Packages.gz",
        ] {
            assert_eq!(plan(hosted, path, "other"), Plan::Derived, "{path}");
        }
        assert_eq!(
            plan(hosted, "/pool/h/hello/hello_1_all.deb", METADATA),
            Plan::Derived
        );
        assert!(matches!(
            plan(hosted, "/pool/h/hello/hello_1_all.deb", "other"),
            Plan::Skip(_)
        ));
        assert!(matches!(plan(hosted, "/notes.txt", "other"), Plan::Skip(_)));
    }
}
