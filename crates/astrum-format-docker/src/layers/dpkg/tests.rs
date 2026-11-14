use super::*;

fn parsed(bytes: &[u8], database: Database) -> Parsed<DpkgPackage> {
    let limits = Limits::DEFAULT;
    parse(
        bytes,
        &limits,
        &mut Budget::new(limits.facts_budget),
        database,
    )
}

fn stanza(name: &str, status: Option<&str>) -> String {
    let status = status.map_or(String::new(), |status| format!("Status: {status}\n"));
    format!("Package: {name}\n{status}Version: 1\n\n")
}

#[test]
fn source_is_read_with_and_without_a_version() {
    let text = "Package: libc6\nStatus: install ok installed\nVersion: 2.36-9+b1\nSource: glibc (2.36-9)\nArchitecture: amd64\n\
Description: x\n y: not a field\n\n\
Package: bash\nStatus: install ok installed\nVersion: 5.2\nSource: bash-src\n\n\
Package: zsh\nStatus: install ok installed\nVersion: 5.9\n\n";
    let parsed = parsed(text.as_bytes(), Database::Status);
    assert_eq!(parsed.malformed, 0);
    let first = &parsed.packages[0];
    assert_eq!(first.source.as_deref(), Some("glibc"));
    assert_eq!(first.source_version.as_deref(), Some("2.36-9"));
    assert_eq!(parsed.packages[1].source.as_deref(), Some("bash-src"));
    assert_eq!(parsed.packages[1].source_version, None);
    assert_eq!(parsed.packages[2].source, None);
}

#[test]
fn every_status_but_not_installed_and_config_files_counts_as_installed() {
    let counted = [
        "install ok installed",
        "hold ok installed",
        "deinstall ok installed",
        "install ok half-installed",
        "install ok unpacked",
        "install ok half-configured",
        "install ok triggers-awaited",
        "install ok triggers-pending",
    ];
    let gone = ["deinstall ok config-files", "purge ok not-installed"];
    let mut text = String::new();
    for (index, status) in counted.iter().chain(&gone).enumerate() {
        text.push_str(&stanza(&format!("p{index}"), Some(status)));
    }
    let parsed = parsed(text.as_bytes(), Database::Status);
    assert_eq!(parsed.packages.len(), counted.len());
    assert_eq!(parsed.malformed, 0);
}

#[test]
fn a_missing_status_is_malformed_in_the_main_file_and_installed_in_a_status_d_file() {
    let text = stanza("distroless", None);
    let main = parsed(text.as_bytes(), Database::Status);
    assert!(main.packages.is_empty());
    assert_eq!(main.malformed, 1);
    let part = parsed(text.as_bytes(), Database::Part);
    assert_eq!(part.packages.len(), 1);
    let short = parsed(stanza("x", Some("install ok")).as_bytes(), Database::Part);
    assert_eq!(short.malformed, 1);
}

#[test]
fn a_field_over_one_kib_or_a_missing_version_makes_an_installed_stanza_malformed() {
    let long = "v".repeat(records::FIELD_LIMIT + 1);
    let text = format!(
        "Package: a\nStatus: install ok installed\nVersion: {long}\n\n\
Package: b\nStatus: install ok installed\n\n\
Package: gone\nStatus: purge ok not-installed\nVersion: {long}\n\n"
    );
    let parsed = parsed(text.as_bytes(), Database::Status);
    assert!(parsed.packages.is_empty());
    assert_eq!(parsed.malformed, 2);
    for garbage in [&b"\xff\x00\n\n: :\n"[..], b"", b"   \n\t\n", b"Status"] {
        let _ = self::parsed(garbage, Database::Status);
    }
}
