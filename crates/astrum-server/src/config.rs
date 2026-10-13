use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;

use astrum_db::{
    DEFAULT_MAX_CONNECTIONS, DEFAULT_SCHEMA, DatabaseConfig, DatabaseUrl, check_pool, check_schema,
};
use serde::Deserialize;
use tracing_subscriber::EnvFilter;

use crate::cli::Cli;

pub const DEFAULT_LISTEN: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8081);
pub const DEFAULT_DATA_DIR: &str = "/astrum-data";
pub const DEFAULT_LOG_FILTER: &str = "info,sqlx=warn";
pub const CONFIG_FILE: &str = "etc/astrum.toml";

pub const ENV_CONFIG: &str = "ASTRUM_CONFIG";
pub const ENV_DATA_DIR: &str = "ASTRUM_DATA_DIR";
pub const ENV_LISTEN: &str = "ASTRUM_LISTEN";
pub const ENV_DATABASE_URL: &str = "ASTRUM_DATABASE_URL";
pub const ENV_DATABASE_SCHEMA: &str = "ASTRUM_DATABASE_SCHEMA";
pub const ENV_DATABASE_MAX_CONNECTIONS: &str = "ASTRUM_DATABASE_MAX_CONNECTIONS";
pub const ENV_LOG: &str = "ASTRUM_LOG";
pub const ENV_LOG_FORMAT: &str = "ASTRUM_LOG_FORMAT";
pub const ENV_SECRETS_KEY_FILE: &str = "ASTRUM_SECRETS_KEY_FILE";
pub const ENV_WEB_DIR: &str = "ASTRUM_WEB_DIR";
pub const ENV_LDAP_CACHE_TTL: &str = "ASTRUM_LDAP_CACHE_TTL";
pub const ENV_CONNECTOR_TLS_CERTIFICATE: &str = "ASTRUM_CONNECTOR_TLS_CERTIFICATE";
pub const ENV_CONNECTOR_TLS_KEY: &str = "ASTRUM_CONNECTOR_TLS_KEY";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub listen: SocketAddr,
    pub data_dir: PathBuf,
    pub database: Option<DatabaseConfig>,
    pub log: LogConfig,
    pub secrets_key: PathBuf,
    pub web_dir: PathBuf,
    pub ldap_cache_ttl: Duration,
    pub connectors: ConnectorConfig,
    pub file: PathBuf,
    pub file_found: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConnectorConfig {
    pub tls_certificate: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogConfig {
    pub filter: String,
    pub format: LogFormat,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            filter: DEFAULT_LOG_FILTER.to_owned(),
            format: LogFormat::Text,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path} is not a valid configuration file: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("{name}: {message}")]
    Value { name: String, message: String },
    #[error("no database is configured: set {ENV_DATABASE_URL} or database.url in {}", file.display())]
    NoDatabase { file: PathBuf },
}

impl Config {
    pub fn database(&self) -> Result<&DatabaseConfig, ConfigError> {
        self.database
            .as_ref()
            .ok_or_else(|| ConfigError::NoDatabase {
                file: self.file.clone(),
            })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    listen: Option<String>,
    data_dir: Option<PathBuf>,
    secrets_key_file: Option<PathBuf>,
    #[serde(default)]
    database: FileDatabase,
    #[serde(default)]
    log: FileLog,
    #[serde(default)]
    web: FileWeb,
    #[serde(default)]
    ldap: FileLdap,
    #[serde(default)]
    connectors: FileConnectors,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConnectors {
    tls_certificate: Option<PathBuf>,
    tls_key: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileLdap {
    cache_ttl: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileWeb {
    dir: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileDatabase {
    url: Option<String>,
    schema: Option<String>,
    max_connections: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileLog {
    filter: Option<String>,
    format: Option<String>,
}

enum Origin<'a> {
    Env(&'a str),
    File(&'a str),
}

impl Origin<'_> {
    fn fail(&self, message: impl ToString) -> ConfigError {
        let name = match self {
            Origin::Env(name) => (*name).to_owned(),
            Origin::File(key) => format!("{key} in the configuration file"),
        };
        ConfigError::Value {
            name,
            message: message.to_string(),
        }
    }
}

pub fn load(cli: &Cli, env: &dyn Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
    let env = |name: &str| env(name).filter(|value| !value.trim().is_empty());

    let hinted_data_dir = cli
        .data_dir
        .clone()
        .or_else(|| env(ENV_DATA_DIR).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR));
    let (file, required) = match cli
        .config
        .clone()
        .or_else(|| env(ENV_CONFIG).map(PathBuf::from))
    {
        Some(path) => (path, true),
        None => (hinted_data_dir.join(CONFIG_FILE), false),
    };
    let parsed = read_file(&file, required)?;
    let file_found = parsed.is_some();
    let parsed = parsed.unwrap_or_default();

    let listen = match (cli.listen, env(ENV_LISTEN), parsed.listen) {
        (Some(listen), _, _) => listen,
        (None, Some(value), _) => parse_listen(&value, Origin::Env(ENV_LISTEN))?,
        (None, None, Some(value)) => parse_listen(&value, Origin::File("listen"))?,
        (None, None, None) => DEFAULT_LISTEN,
    };

    let data_dir = cli
        .data_dir
        .clone()
        .or_else(|| env(ENV_DATA_DIR).map(PathBuf::from))
        .or(parsed.data_dir)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR));

    let secrets_key = env(ENV_SECRETS_KEY_FILE)
        .map(PathBuf::from)
        .or(parsed.secrets_key_file)
        .unwrap_or_else(|| data_dir.join(astrum_security::secrets::KEY_FILE));

    let web_dir = cli
        .web_dir
        .clone()
        .or_else(|| env(ENV_WEB_DIR).map(PathBuf::from))
        .or(parsed.web.dir)
        .unwrap_or_else(|| PathBuf::from(crate::web::DEFAULT_DIR));

    let database = database(&env, parsed.database)?;

    let ldap_cache_ttl = match (env(ENV_LDAP_CACHE_TTL), parsed.ldap.cache_ttl) {
        (Some(value), _) => parse_duration(&value, Origin::Env(ENV_LDAP_CACHE_TTL))?,
        (None, Some(value)) => parse_duration(&value, Origin::File("ldap.cache_ttl"))?,
        (None, None) => astrum_security::ldap::DEFAULT_CACHE_TTL,
    };

    let connectors = ConnectorConfig {
        tls_certificate: env(ENV_CONNECTOR_TLS_CERTIFICATE)
            .map(PathBuf::from)
            .or(parsed.connectors.tls_certificate),
        tls_key: env(ENV_CONNECTOR_TLS_KEY)
            .map(PathBuf::from)
            .or(parsed.connectors.tls_key),
    };
    if connectors.tls_certificate.is_some() != connectors.tls_key.is_some() {
        return Err(Origin::File("connectors").fail(format!(
            "set both the certificate and the key ({ENV_CONNECTOR_TLS_CERTIFICATE} and {ENV_CONNECTOR_TLS_KEY}, or connectors.tls_certificate and connectors.tls_key) or neither"
        )));
    }

    let filter = match (env(ENV_LOG), parsed.log.filter) {
        (Some(value), _) => check_filter(value, Origin::Env(ENV_LOG))?,
        (None, Some(value)) => check_filter(value, Origin::File("log.filter"))?,
        (None, None) => DEFAULT_LOG_FILTER.to_owned(),
    };
    let format = match (env(ENV_LOG_FORMAT), parsed.log.format) {
        (Some(value), _) => parse_format(&value, Origin::Env(ENV_LOG_FORMAT))?,
        (None, Some(value)) => parse_format(&value, Origin::File("log.format"))?,
        (None, None) => LogFormat::Text,
    };

    Ok(Config {
        listen,
        data_dir,
        database,
        log: LogConfig { filter, format },
        secrets_key,
        web_dir,
        ldap_cache_ttl,
        connectors,
        file,
        file_found,
    })
}

fn read_file(path: &Path, required: bool) -> Result<Option<FileConfig>, ConfigError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if !required && error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ConfigError::Read {
                path: path.to_owned(),
                source,
            });
        }
    };
    toml::from_str(&text)
        .map(Some)
        .map_err(|source| ConfigError::Parse {
            path: path.to_owned(),
            source,
        })
}

fn database(
    env: &dyn Fn(&str) -> Option<String>,
    file: FileDatabase,
) -> Result<Option<DatabaseConfig>, ConfigError> {
    let url = match (env(ENV_DATABASE_URL), file.url) {
        (Some(value), _) => Some((value, Origin::Env(ENV_DATABASE_URL))),
        (None, Some(value)) => Some((value, Origin::File("database.url"))),
        (None, None) => None,
    };
    let schema = match (env(ENV_DATABASE_SCHEMA), file.schema) {
        (Some(value), _) => (value, Origin::Env(ENV_DATABASE_SCHEMA)),
        (None, Some(value)) => (value, Origin::File("database.schema")),
        (None, None) => (DEFAULT_SCHEMA.to_owned(), Origin::File("database.schema")),
    };
    check_schema(&schema.0).map_err(|error| schema.1.fail(error))?;
    let max_connections = match (env(ENV_DATABASE_MAX_CONNECTIONS), file.max_connections) {
        (Some(value), _) => {
            let origin = Origin::Env(ENV_DATABASE_MAX_CONNECTIONS);
            let count = value.trim().parse().map_err(|error| origin.fail(error))?;
            check_pool(count).map_err(|error| origin.fail(error))?;
            count
        }
        (None, Some(count)) => {
            check_pool(count)
                .map_err(|error| Origin::File("database.max_connections").fail(error))?;
            count
        }
        (None, None) => DEFAULT_MAX_CONNECTIONS,
    };
    let Some((url, origin)) = url else {
        return Ok(None);
    };
    let url = DatabaseUrl::parse(&url).map_err(|error| origin.fail(error))?;
    Ok(Some(DatabaseConfig {
        url,
        schema: schema.0,
        max_connections,
    }))
}

fn parse_listen(value: &str, origin: Origin<'_>) -> Result<SocketAddr, ConfigError> {
    value.trim().parse().map_err(|_| {
        origin.fail(format!(
            "\"{value}\" is not an address such as 0.0.0.0:8081"
        ))
    })
}

fn check_filter(value: String, origin: Origin<'_>) -> Result<String, ConfigError> {
    EnvFilter::try_new(&value).map_err(|error| origin.fail(error))?;
    Ok(value)
}

fn parse_duration(value: &str, origin: Origin<'_>) -> Result<Duration, ConfigError> {
    let text = value.trim().to_ascii_lowercase();
    let (number, unit) = match text.find(|c: char| !c.is_ascii_digit()) {
        Some(at) => text.split_at(at),
        None => (text.as_str(), "s"),
    };
    let factor = match unit.trim() {
        "ms" => {
            return number
                .parse::<u64>()
                .map(Duration::from_millis)
                .map_err(|_| {
                    origin.fail(format!("\"{value}\" is not a duration such as 5m or 300s"))
                });
        }
        "s" | "sec" | "seconds" => 1,
        "m" | "min" | "minutes" => 60,
        "h" | "hours" => 3600,
        "d" | "days" => 86_400,
        _ => 0,
    };
    match number.parse::<u64>() {
        Ok(count) if factor > 0 => Ok(Duration::from_secs(count.saturating_mul(factor))),
        _ => Err(origin.fail(format!("\"{value}\" is not a duration such as 5m or 300s"))),
    }
}

fn parse_format(value: &str, origin: Origin<'_>) -> Result<LogFormat, ConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "text" => Ok(LogFormat::Text),
        "json" => Ok(LogFormat::Json),
        _ => Err(origin.fail(format!("\"{value}\" is neither text nor json"))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        move |name| map.get(name).cloned()
    }

    fn data_dir_with(text: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("etc")).unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE), text).unwrap();
        dir
    }

    fn cli_in(dir: &Path) -> Cli {
        Cli {
            data_dir: Some(dir.to_owned()),
            ..Cli::default()
        }
    }

    #[test]
    fn the_ldap_cache_lifetime_reads_like_the_old_property() {
        let config = load(&Cli::default(), &env(&[])).unwrap();
        assert_eq!(config.ldap_cache_ttl, Duration::from_secs(300));
        let config = load(&Cli::default(), &env(&[(ENV_LDAP_CACHE_TTL, "1m")])).unwrap();
        assert_eq!(config.ldap_cache_ttl, Duration::from_secs(60));
        let dir = data_dir_with("[ldap]\ncache_ttl = \"90\"\n");
        let config = load(&cli_in(dir.path()), &env(&[])).unwrap();
        assert_eq!(config.ldap_cache_ttl, Duration::from_secs(90));
        let config = load(&Cli::default(), &env(&[(ENV_LDAP_CACHE_TTL, "0")])).unwrap();
        assert_eq!(config.ldap_cache_ttl, Duration::ZERO);
        assert!(load(&Cli::default(), &env(&[(ENV_LDAP_CACHE_TTL, "soon")])).is_err());
        assert!(load(&Cli::default(), &env(&[(ENV_LDAP_CACHE_TTL, "5x")])).is_err());
    }

    #[test]
    fn connector_certificates_come_in_pairs() {
        let config = load(&Cli::default(), &env(&[])).unwrap();
        assert_eq!(config.connectors, ConnectorConfig::default());
        let config = load(
            &Cli::default(),
            &env(&[
                (ENV_CONNECTOR_TLS_CERTIFICATE, "/tls/chain.pem"),
                (ENV_CONNECTOR_TLS_KEY, "/tls/key.pem"),
            ]),
        )
        .unwrap();
        assert_eq!(
            config.connectors.tls_certificate,
            Some(PathBuf::from("/tls/chain.pem"))
        );
        let dir = data_dir_with("[connectors]\ntls_certificate = \"/tls/chain.pem\"\n");
        let error = load(&cli_in(dir.path()), &env(&[]))
            .unwrap_err()
            .to_string();
        assert!(error.contains(ENV_CONNECTOR_TLS_KEY), "{error}");
    }

    #[test]
    fn defaults_need_nothing_but_a_database_to_serve() {
        let config = load(&Cli::default(), &env(&[])).unwrap();
        assert_eq!(config.listen, DEFAULT_LISTEN);
        assert_eq!(config.listen.to_string(), "0.0.0.0:8081");
        assert_eq!(config.data_dir, PathBuf::from("/astrum-data"));
        assert_eq!(config.file, PathBuf::from("/astrum-data/etc/astrum.toml"));
        assert!(!config.file_found);
        assert_eq!(config.log, LogConfig::default());
        assert_eq!(
            config.secrets_key,
            PathBuf::from("/astrum-data/etc/secrets.key")
        );
        assert_eq!(config.web_dir, PathBuf::from("/usr/share/astrum/web"));
        assert!(config.database.is_none());
        let error = config.database().unwrap_err().to_string();
        assert!(error.contains(ENV_DATABASE_URL), "{error}");
        assert!(error.contains("/astrum-data/etc/astrum.toml"), "{error}");
    }

    #[test]
    fn the_environment_configures_everything() {
        let config = load(
            &Cli::default(),
            &env(&[
                (ENV_DATA_DIR, "/srv/astrum"),
                (ENV_LISTEN, "127.0.0.1:9000"),
                (ENV_DATABASE_URL, "postgres://astrum:pw@db/astrum"),
                (ENV_DATABASE_SCHEMA, "repo"),
                (ENV_DATABASE_MAX_CONNECTIONS, "5"),
                (ENV_LOG, "astrum=debug,info"),
                (ENV_LOG_FORMAT, "JSON"),
                (ENV_SECRETS_KEY_FILE, "/run/secrets/astrum.key"),
                (ENV_WEB_DIR, "/srv/astrum-web"),
            ]),
        )
        .unwrap();
        assert_eq!(config.web_dir, PathBuf::from("/srv/astrum-web"));
        assert_eq!(config.secrets_key, PathBuf::from("/run/secrets/astrum.key"));
        assert_eq!(config.listen, "127.0.0.1:9000".parse().unwrap());
        assert_eq!(config.data_dir, PathBuf::from("/srv/astrum"));
        assert_eq!(config.file, PathBuf::from("/srv/astrum/etc/astrum.toml"));
        let database = config.database().unwrap();
        assert_eq!(database.url.expose(), "postgres://astrum:pw@db/astrum");
        assert_eq!(database.schema, "repo");
        assert_eq!(database.max_connections, 5);
        assert_eq!(config.log.filter, "astrum=debug,info");
        assert_eq!(config.log.format, LogFormat::Json);
    }

    #[test]
    fn the_file_in_the_data_directory_is_read() {
        let dir = data_dir_with(
            r#"
listen = "127.0.0.1:8082"

[database]
url = "postgres://astrum:pw@db/astrum"
max_connections = 7

[log]
filter = "warn"
format = "json"

[web]
dir = "/opt/astrum/web"
"#,
        );
        let config = load(&cli_in(dir.path()), &env(&[])).unwrap();
        assert_eq!(config.web_dir, PathBuf::from("/opt/astrum/web"));
        assert!(config.file_found);
        assert_eq!(config.file, dir.path().join(CONFIG_FILE));
        assert_eq!(config.listen, "127.0.0.1:8082".parse().unwrap());
        assert_eq!(config.data_dir, dir.path());
        let database = config.database().unwrap();
        assert_eq!(database.schema, "astrum");
        assert_eq!(database.max_connections, 7);
        assert_eq!(config.log.filter, "warn");
        assert_eq!(config.log.format, LogFormat::Json);
    }

    #[test]
    fn the_environment_wins_over_the_file_and_flags_win_over_both() {
        let dir = data_dir_with(
            r#"
listen = "127.0.0.1:8082"
[database]
url = "postgres://file@db/astrum"
schema = "from_file"
[web]
dir = "/from/file"
"#,
        );
        let variables = env(&[
            (ENV_LISTEN, "127.0.0.1:8083"),
            (ENV_DATABASE_URL, "postgres://env@db/astrum"),
            (ENV_WEB_DIR, "/from/env"),
        ]);
        let config = load(&cli_in(dir.path()), &variables).unwrap();
        assert_eq!(config.listen, "127.0.0.1:8083".parse().unwrap());
        assert_eq!(config.web_dir, PathBuf::from("/from/env"));
        let database = config.database().unwrap();
        assert_eq!(database.url.expose(), "postgres://env@db/astrum");
        assert_eq!(database.schema, "from_file");

        let cli = Cli {
            listen: Some("127.0.0.1:8084".parse().unwrap()),
            web_dir: Some(PathBuf::from("/from/flag")),
            ..cli_in(dir.path())
        };
        let config = load(&cli, &variables).unwrap();
        assert_eq!(config.listen, "127.0.0.1:8084".parse().unwrap());
        assert_eq!(config.web_dir, PathBuf::from("/from/flag"));
    }

    #[test]
    fn a_file_named_explicitly_must_exist() {
        let dir = tempfile::tempdir().unwrap();
        let cli = Cli {
            config: Some(dir.path().join("missing.toml")),
            ..Cli::default()
        };
        assert!(matches!(
            load(&cli, &env(&[])),
            Err(ConfigError::Read { .. })
        ));

        let named = env(&[(ENV_CONFIG, "/nonexistent/astrum.toml")]);
        assert!(matches!(
            load(&Cli::default(), &named),
            Err(ConfigError::Read { .. })
        ));
    }

    #[test]
    fn a_file_elsewhere_can_move_the_data_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("astrum.toml");
        std::fs::write(&path, "data_dir = \"/var/lib/astrum\"\n").unwrap();
        let cli = Cli {
            config: Some(path.clone()),
            ..Cli::default()
        };
        let config = load(&cli, &env(&[])).unwrap();
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/astrum"));
        assert_eq!(config.file, path);
    }

    #[test]
    fn unknown_keys_in_the_file_are_refused() {
        let dir = data_dir_with("[database]\npassword = \"x\"\n");
        let error = load(&cli_in(dir.path()), &env(&[])).unwrap_err();
        assert!(matches!(error, ConfigError::Parse { .. }), "{error}");
    }

    #[test]
    fn bad_values_name_where_they_came_from() {
        let cases: [(&[(&str, &str)], &str); 6] = [
            (&[(ENV_LISTEN, "8081")], ENV_LISTEN),
            (&[(ENV_DATABASE_URL, "mysql://db/astrum")], ENV_DATABASE_URL),
            (&[(ENV_DATABASE_SCHEMA, "Astrum")], ENV_DATABASE_SCHEMA),
            (
                &[(ENV_DATABASE_MAX_CONNECTIONS, "0")],
                ENV_DATABASE_MAX_CONNECTIONS,
            ),
            (&[(ENV_LOG, "astrum=loud")], ENV_LOG),
            (&[(ENV_LOG_FORMAT, "xml")], ENV_LOG_FORMAT),
        ];
        for (pairs, name) in cases {
            let error = load(&Cli::default(), &env(pairs)).unwrap_err().to_string();
            assert!(error.starts_with(name), "{error}");
        }

        let dir = data_dir_with("[database]\nurl = \"redis://cache\"\n");
        let error = load(&cli_in(dir.path()), &env(&[]))
            .unwrap_err()
            .to_string();
        assert!(
            error.starts_with("database.url in the configuration file"),
            "{error}"
        );
    }

    #[test]
    fn blank_variables_count_as_unset() {
        let config = load(
            &Cli::default(),
            &env(&[(ENV_LISTEN, " "), (ENV_DATABASE_URL, "")]),
        )
        .unwrap();
        assert_eq!(config.listen, DEFAULT_LISTEN);
        assert!(config.database.is_none());
    }

    #[test]
    fn errors_never_show_the_database_password() {
        let error = load(
            &Cli::default(),
            &env(&[
                (ENV_DATABASE_URL, "postgres://astrum:s3cret@db/astrum"),
                (ENV_DATABASE_SCHEMA, "Bad"),
            ]),
        )
        .unwrap_err();
        assert!(!error.to_string().contains("s3cret"));
        let config = load(
            &Cli::default(),
            &env(&[(ENV_DATABASE_URL, "postgres://astrum:s3cret@db/astrum")]),
        )
        .unwrap();
        assert!(!format!("{config:?}").contains("s3cret"));
    }
}
