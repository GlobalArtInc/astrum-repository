use std::fmt;

use url::Url;

pub const DEFAULT_SCHEMA: &str = "astrum";
pub const DEFAULT_MAX_CONNECTIONS: u32 = 20;
pub const MIN_CONNECTIONS: u32 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub url: DatabaseUrl,
    pub schema: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn new(url: DatabaseUrl) -> Self {
        Self {
            url,
            schema: DEFAULT_SCHEMA.to_owned(),
            max_connections: DEFAULT_MAX_CONNECTIONS,
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("the database address is not a URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("the database address must start with postgres:// or postgresql://, not {0}://")]
    Scheme(String),
    #[error("\"{0}\" is not a schema name: use lower-case letters, digits and underscores")]
    Schema(String),
    #[error("the connection pool needs at least {MIN_CONNECTIONS} connections, not {0}")]
    Pool(u32),
}

#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseUrl(Url);

impl DatabaseUrl {
    pub fn parse(value: &str) -> Result<Self, ConfigError> {
        let url = Url::parse(value.trim())?;
        match url.scheme() {
            "postgres" | "postgresql" => Ok(Self(url)),
            other => Err(ConfigError::Scheme(other.to_owned())),
        }
    }

    pub fn expose(&self) -> &str {
        self.0.as_str()
    }

    pub fn database(&self) -> &str {
        self.0.path().trim_start_matches('/')
    }

    pub fn with_database(&self, name: &str) -> Self {
        let mut url = self.0.clone();
        url.set_path(&format!("/{name}"));
        Self(url)
    }
}

impl fmt::Display for DatabaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut shown = self.0.clone();
        if shown.password().is_some() {
            let _ = shown.set_password(Some("***"));
        }
        let pairs: Vec<(String, String)> = shown
            .query_pairs()
            .map(|(key, value)| {
                let value = if key.eq_ignore_ascii_case("password") {
                    "***".to_owned()
                } else {
                    value.into_owned()
                };
                (key.into_owned(), value)
            })
            .collect();
        if !pairs.is_empty() {
            shown.query_pairs_mut().clear().extend_pairs(pairs);
        }
        f.write_str(shown.as_str())
    }
}

impl fmt::Debug for DatabaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatabaseUrl({self})")
    }
}

pub fn check_schema(name: &str) -> Result<(), ConfigError> {
    let mut chars = name.chars();
    let valid = matches!(chars.next(), Some('a'..='z' | '_'))
        && chars.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_'))
        && name.len() <= 63
        && !name.starts_with("pg_");
    match valid {
        true => Ok(()),
        false => Err(ConfigError::Schema(name.to_owned())),
    }
}

pub fn check_pool(max_connections: u32) -> Result<(), ConfigError> {
    match max_connections >= MIN_CONNECTIONS {
        true => Ok(()),
        false => Err(ConfigError::Pool(max_connections)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_password_never_shows() {
        let url = DatabaseUrl::parse("postgres://astrum:s3cret@db:5432/astrum").unwrap();
        assert_eq!(url.to_string(), "postgres://astrum:***@db:5432/astrum");
        assert_eq!(
            format!("{url:?}"),
            "DatabaseUrl(postgres://astrum:***@db:5432/astrum)"
        );
        assert!(url.expose().contains("s3cret"));
    }

    #[test]
    fn a_password_in_the_query_never_shows() {
        let url = DatabaseUrl::parse("postgres://db/astrum?user=astrum&password=s3cret").unwrap();
        assert_eq!(
            url.to_string(),
            "postgres://db/astrum?user=astrum&password=***"
        );
    }

    #[test]
    fn an_address_without_a_password_shows_as_it_is() {
        let url = DatabaseUrl::parse("postgresql://astrum@db/astrum").unwrap();
        assert_eq!(url.to_string(), "postgresql://astrum@db/astrum");
    }

    #[test]
    fn only_postgres_addresses_are_accepted() {
        assert_eq!(
            DatabaseUrl::parse("mysql://db/astrum").unwrap_err(),
            ConfigError::Scheme("mysql".to_owned())
        );
        assert!(DatabaseUrl::parse("not a url").is_err());
    }

    #[test]
    fn another_database_on_the_same_server() {
        let url =
            DatabaseUrl::parse("postgres://astrum:pw@db:5432/postgres?sslmode=disable").unwrap();
        let other = url.with_database("astrum_test_1");
        assert_eq!(other.database(), "astrum_test_1");
        assert_eq!(
            other.expose(),
            "postgres://astrum:pw@db:5432/astrum_test_1?sslmode=disable"
        );
    }

    #[test]
    fn schema_names_are_plain_identifiers() {
        for good in ["astrum", "astrum_2", "_private"] {
            assert!(check_schema(good).is_ok(), "{good}");
        }
        for bad in ["", "Astrum", "astrum-two", "2astrum", "a\"b", "pg_astrum"] {
            assert!(check_schema(bad).is_err(), "{bad}");
        }
        assert!(check_schema(&"a".repeat(64)).is_err());
    }
}
