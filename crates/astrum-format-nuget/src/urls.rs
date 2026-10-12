use astrum_content::Ctx;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Urls {
    base: String,
}

impl Urls {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into().trim_end_matches('/').to_owned(),
        }
    }

    pub fn of(ctx: &Ctx) -> Self {
        Self::new(ctx.repository_url())
    }

    pub fn root(&self) -> String {
        format!("{}/", self.base)
    }

    pub fn registration_base(&self) -> String {
        format!("{}/v3/registration/", self.base)
    }

    pub fn content_base(&self) -> String {
        format!("{}/v3/content/", self.base)
    }

    pub fn query(&self) -> String {
        format!("{}/v3/query", self.base)
    }

    pub fn autocomplete(&self) -> String {
        format!("{}/v3/autocomplete", self.base)
    }

    pub fn registration_index(&self, id: &str) -> String {
        format!(
            "{}{}/index.json",
            self.registration_base(),
            id.to_lowercase()
        )
    }

    pub fn registration_leaf(&self, id: &str, version: &str) -> String {
        format!(
            "{}{}/{}.json",
            self.registration_base(),
            id.to_lowercase(),
            version.to_lowercase()
        )
    }

    pub fn catalog_leaf(&self, id: &str, version: &str) -> String {
        format!(
            "{}/v3/catalog/{}/{}.json",
            self.base,
            id.to_lowercase(),
            version.to_lowercase()
        )
    }

    pub fn package_content(&self, id: &str, version: &str) -> String {
        let id = id.to_lowercase();
        let version = version.to_lowercase();
        format!("{}{id}/{version}/{id}.{version}.nupkg", self.content_base())
    }

    pub fn v2_download(&self, id: &str, version: &str) -> String {
        format!("{}/package/{id}/{version}", self.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_hang_off_the_repository() {
        let urls = Urls::new("http://h/repository/n/");
        assert_eq!(urls.root(), "http://h/repository/n/");
        assert_eq!(
            urls.registration_index("A.B"),
            "http://h/repository/n/v3/registration/a.b/index.json"
        );
        assert_eq!(
            urls.registration_leaf("A", "1.0.0-RC"),
            "http://h/repository/n/v3/registration/a/1.0.0-rc.json"
        );
        assert_eq!(
            urls.package_content("A", "1.0.0"),
            "http://h/repository/n/v3/content/a/1.0.0/a.1.0.0.nupkg"
        );
        assert_eq!(
            urls.catalog_leaf("A", "1.0.0"),
            "http://h/repository/n/v3/catalog/a/1.0.0.json"
        );
        assert_eq!(
            urls.v2_download("A", "1.0"),
            "http://h/repository/n/package/A/1.0"
        );
    }
}
