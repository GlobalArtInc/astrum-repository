pub const LOCAL: &str = "AstrumAuthenticatingRealm";
pub const AUTHORIZING: &str = "AstrumAuthorizingRealm";
pub const ACCESS_TOKEN: &str = "AccessTokenRealm";
pub const LDAP: &str = "LdapRealm";
pub const OIDC: &str = "OAuth2Realm";
pub const DEFAULT_ROLE: &str = "DefaultRole";

pub const DEFAULT_ACTIVE: [&str; 2] = [LOCAL, ACCESS_TOKEN];

pub struct Realm {
    pub id: &'static str,
    pub name: &'static str,
}

pub const AVAILABLE: [Realm; 12] = [
    Realm {
        id: ACCESS_TOKEN,
        name: "Access Token Realm",
    },
    Realm {
        id: "CargoToken",
        name: "Cargo Bearer Token Realm",
    },
    Realm {
        id: "ConanToken",
        name: "Conan Bearer Token Realm",
    },
    Realm {
        id: DEFAULT_ROLE,
        name: "Default Role Realm",
    },
    Realm {
        id: "DockerToken",
        name: "Docker Bearer Token Realm",
    },
    Realm {
        id: LDAP,
        name: "LDAP Realm",
    },
    Realm {
        id: LOCAL,
        name: "Local Authenticating Realm",
    },
    Realm {
        id: "NpmToken",
        name: "npm Bearer Token Realm",
    },
    Realm {
        id: "NuGetApiKey",
        name: "NuGet API-Key Realm",
    },
    Realm {
        id: OIDC,
        name: "OpenID Connect Realm",
    },
    Realm {
        id: "RubyGemsApiKey",
        name: "RubyGems API Key Realm",
    },
    Realm {
        id: "TerraformToken",
        name: "Terraform Bearer Token Realm",
    },
];

pub struct UserSource {
    pub id: &'static str,
    pub realm: &'static str,
}

pub const USER_SOURCES: [UserSource; 3] = [
    UserSource {
        id: "default",
        realm: LOCAL,
    },
    UserSource {
        id: "LDAP",
        realm: LDAP,
    },
    UserSource {
        id: "OAuth2",
        realm: OIDC,
    },
];

pub fn is_known(id: &str) -> bool {
    AVAILABLE.iter().any(|realm| realm.id == id)
}
