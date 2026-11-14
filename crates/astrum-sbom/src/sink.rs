use crate::text::Text;
use crate::{MAX_PACKAGES, Package, ecosystem};

pub(crate) const LIMIT: &str = "the package limit is exceeded";
pub(crate) const DUPLICATE: &str = "a field is repeated";

#[derive(Default)]
pub(crate) struct Sink {
    pub(crate) packages: Vec<Package>,
    pub(crate) overflow: bool,
}

impl Sink {
    pub(crate) fn add(
        &mut self,
        name: Text,
        version: Text,
        purl: Option<String>,
        licenses: Vec<String>,
    ) -> Result<(), ()> {
        let Some(name) = name.0.filter(|name| !name.trim().is_empty()) else {
            return Ok(());
        };
        if self.packages.len() >= MAX_PACKAGES {
            self.overflow = true;
            return Err(());
        }
        let purl = purl.filter(|purl| !purl.is_empty());
        let ecosystem = purl.as_deref().and_then(ecosystem::of);
        self.packages.push(Package {
            name,
            version: version.0.unwrap_or_default(),
            purl,
            licenses,
            ecosystem,
        });
        Ok(())
    }
}
