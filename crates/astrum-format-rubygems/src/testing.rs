use std::io::Write;

pub fn spec_yaml(
    name: &str,
    version: &str,
    platform: &str,
    dependencies: &[(&str, &str)],
) -> String {
    let mut yaml = format!(
        "--- !ruby/object:Gem::Specification\nname: {name}\nversion: !ruby/object:Gem::Version\n  version: {version}\nplatform: {platform}\nauthors:\n- Jane Doe\ndate: 2026-10-13 00:00:00.000000000 Z\ndependencies:\n"
    );
    if dependencies.is_empty() {
        yaml = yaml.replace("dependencies:\n", "dependencies: []\n");
    }
    for (dependency, requirement) in dependencies {
        let (operator, wanted) = requirement.split_once(' ').unwrap_or(("=", requirement));
        yaml.push_str(&format!(
            "- !ruby/object:Gem::Dependency\n  name: {dependency}\n  requirement: !ruby/object:Gem::Requirement\n    requirements:\n    - - \"{operator}\"\n      - !ruby/object:Gem::Version\n        version: '{wanted}'\n  type: :runtime\n  prerelease: false\n"
        ));
    }
    yaml.push_str(&format!(
        "description: The {name} gem for tests\nemail: jane@example.org\nhomepage: https://example.org/{name}\nlicenses:\n- MIT\nrequired_ruby_version: !ruby/object:Gem::Requirement\n  requirements:\n  - - \">=\"\n    - !ruby/object:Gem::Version\n      version: '0'\nrubygems_version: 3.6.7\nspecification_version: 4\nsummary: {name} summary\n"
    ));
    yaml
}

fn gzip(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder
        .write_all(data)
        .expect("writing to memory never fails");
    encoder.finish().expect("writing to memory never fails")
}

pub fn gem(name: &str, version: &str, platform: &str, dependencies: &[(&str, &str)]) -> Vec<u8> {
    let metadata = gzip(spec_yaml(name, version, platform, dependencies).as_bytes());
    let mut inner = tar::Builder::new(Vec::new());
    let source = format!("puts '{name} {version}'\n");
    let mut header = tar::Header::new_gnu();
    header.set_size(source.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    inner
        .append_data(&mut header, format!("lib/{name}.rb"), source.as_bytes())
        .expect("writing to memory never fails");
    let data = gzip(&inner.into_inner().expect("writing to memory never fails"));
    let mut outer = tar::Builder::new(Vec::new());
    for (path, bytes) in [("metadata.gz", metadata), ("data.tar.gz", data)] {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o444);
        header.set_cksum();
        outer
            .append_data(&mut header, path, bytes.as_slice())
            .expect("writing to memory never fails");
    }
    outer.into_inner().expect("writing to memory never fails")
}
