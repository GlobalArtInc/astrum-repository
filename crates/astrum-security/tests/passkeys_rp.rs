use astrum_security::two_factor::RelyingPartyPolicy;
use astrum_security::two_factor::passkeys::{RelyingParty, RelyingPartyError};

type Accepted<'a> = (
    Option<&'a str>,
    &'a [&'a str],
    Option<&'a str>,
    &'a str,
    &'a [&'a str],
);
type Refused<'a> = (
    Option<&'a str>,
    &'a [&'a str],
    Option<&'a str>,
    RelyingPartyError,
);

fn resolve(
    id: Option<&str>,
    origins: &[&str],
    base: Option<&str>,
) -> Result<RelyingParty, RelyingPartyError> {
    RelyingParty::resolve(
        &RelyingPartyPolicy {
            id: id.map(str::to_owned),
            origins: origins.iter().map(|origin| (*origin).to_owned()).collect(),
        },
        base,
    )
}

#[test]
fn the_relying_party_is_resolved_from_the_base_url_and_the_policy() {
    let accepted: Vec<Accepted> = vec![
        (
            None,
            &[],
            Some("https://astrum.example.com"),
            "astrum.example.com",
            &["https://astrum.example.com/"],
        ),
        (
            None,
            &[],
            Some("https://Astrum.Example.com:443/ui/?x=1#y"),
            "astrum.example.com",
            &["https://astrum.example.com/"],
        ),
        (
            None,
            &[],
            Some("https://astrum.example.com:8443"),
            "astrum.example.com",
            &["https://astrum.example.com:8443/"],
        ),
        (
            Some("example.com"),
            &[],
            Some("https://astrum.example.com"),
            "example.com",
            &["https://astrum.example.com/"],
        ),
        (
            Some("Astrum.Example.com"),
            &[],
            Some("https://astrum.example.com"),
            "astrum.example.com",
            &["https://astrum.example.com/"],
        ),
        (
            None,
            &[
                "https://astrum.example.com",
                "https://alt.astrum.example.com",
            ],
            Some("https://astrum.example.com"),
            "astrum.example.com",
            &[
                "https://astrum.example.com/",
                "https://alt.astrum.example.com/",
            ],
        ),
        (
            Some("example.com"),
            &["https://example.com", "https://other.example.com:9443"],
            Some("https://astrum.example.com"),
            "example.com",
            &[
                "https://astrum.example.com/",
                "https://example.com/",
                "https://other.example.com:9443/",
            ],
        ),
        (
            None,
            &[],
            Some("http://localhost:8081"),
            "localhost",
            &["http://localhost:8081/"],
        ),
        (
            None,
            &["http://localhost:3000"],
            Some("http://localhost:8081"),
            "localhost",
            &["http://localhost:8081/", "http://localhost:3000/"],
        ),
        (
            Some("example.com"),
            &["https://example.com"],
            None,
            "example.com",
            &["https://example.com/"],
        ),
        (
            Some("localhost"),
            &["http://localhost:5173"],
            None,
            "localhost",
            &["http://localhost:5173/"],
        ),
    ];
    for (id, origins, base, expected_id, expected_origins) in accepted {
        let rp =
            resolve(id, origins, base).unwrap_or_else(|error| panic!("{base:?} {id:?}: {error}"));
        assert_eq!(rp.id(), expected_id, "{base:?} {id:?}");
        let got: Vec<&str> = rp.origins().iter().map(|origin| origin.as_str()).collect();
        assert_eq!(got, expected_origins, "{base:?} {id:?}");
    }

    let base = Some("https://astrum.example.com");
    let tld = || RelyingPartyError::Id("a top level domain cannot be used");
    let plain = RelyingPartyError::Id("it must be a plain ASCII domain name");
    let outside = "its host is neither the relying party id nor a subdomain of it";
    let scheme = "it is neither https nor http://localhost";
    let origin = |value: &str, reason: &'static str| RelyingPartyError::Origin {
        origin: value.to_owned(),
        reason,
    };
    let refused: Vec<Refused> = vec![
        (None, &[], None, RelyingPartyError::Unavailable),
        (
            None,
            &["https://astrum.example.com"],
            None,
            RelyingPartyError::Unavailable,
        ),
        (
            Some("example.com"),
            &[],
            None,
            RelyingPartyError::Unavailable,
        ),
        (Some("other.com"), &[], base, RelyingPartyError::IdNotSuffix),
        (
            Some("xample.com"),
            &[],
            base,
            RelyingPartyError::IdNotSuffix,
        ),
        (
            Some("sub.astrum.example.com"),
            &[],
            base,
            RelyingPartyError::IdNotSuffix,
        ),
        (Some("com"), &[], base, tld()),
        (Some("com"), &["https://example.com"], None, tld()),
        (Some("exa mple.com"), &[], base, plain),
        (
            Some("127.0.0.1"),
            &[],
            None,
            RelyingPartyError::Id("an IP address cannot be used"),
        ),
        (
            None,
            &["https://other.com"],
            base,
            origin("https://other.com/", outside),
        ),
        (
            None,
            &["https://badastrum.example.com"],
            base,
            origin("https://badastrum.example.com/", outside),
        ),
        (
            None,
            &["http://astrum.example.com"],
            base,
            origin("http://astrum.example.com/", scheme),
        ),
        (
            None,
            &["https://astrum.example.com/path"],
            base,
            origin("https://astrum.example.com/path", "it is not a bare origin"),
        ),
        (
            None,
            &["https://user@astrum.example.com"],
            base,
            origin("https://user@astrum.example.com", "it carries credentials"),
        ),
        (
            None,
            &["https://astrum.example.com?x=1"],
            base,
            origin("https://astrum.example.com?x=1", "it is not a bare origin"),
        ),
        (
            None,
            &["not a url"],
            base,
            origin("not a url", "it is not a URL"),
        ),
        (
            None,
            &[],
            Some("http://astrum.example.com"),
            origin("http://astrum.example.com/", scheme),
        ),
        (
            None,
            &[],
            Some("https://127.0.0.1:8081"),
            RelyingPartyError::BaseUrl("its host is an IP address"),
        ),
        (
            None,
            &[],
            Some("ftp://astrum.example.com"),
            RelyingPartyError::BaseUrl("it is not an http or https URL"),
        ),
    ];
    for (id, origins, base, expected) in refused {
        let error = resolve(id, origins, base).expect_err(&format!("{base:?} {id:?} {origins:?}"));
        assert_eq!(error, expected, "{base:?} {id:?} {origins:?}");
    }
    assert_eq!(
        resolve(None, &[], Some("not a url")).unwrap_err(),
        RelyingPartyError::BaseUrl("it is not a URL")
    );
    assert_eq!(
        resolve(None, &[], Some("https://astrum.example.com.")).unwrap_err(),
        RelyingPartyError::Id("it must be a plain ASCII domain name")
    );
}
