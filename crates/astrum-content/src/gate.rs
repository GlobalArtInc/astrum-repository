use astrum_policy::exception::Exception;
use astrum_policy::glob;
use async_trait::async_trait;
use chrono::{DateTime, TimeDelta, Utc};

use crate::store::Asset;
use crate::{ContentError, Ctx};

#[derive(Clone, Debug)]
pub struct Cooldown {
    pub cutoff: DateTime<Utc>,
    pub min_age: TimeDelta,
    pub block: bool,
    pub exclude: Vec<String>,
    pub excepted: Vec<Exception>,
}

impl Cooldown {
    pub fn hides(
        &self,
        display_name: &str,
        namespace: &str,
        name: &str,
        version: &str,
        published: Option<DateTime<Utc>>,
    ) -> bool {
        let Some(published) = published else {
            return false;
        };
        self.block
            && published > self.cutoff
            && !self
                .exclude
                .iter()
                .any(|pattern| glob::matches(pattern, display_name))
            && !self
                .excepted
                .iter()
                .any(|exception| exception.covers_component(namespace, name, version))
    }

    pub fn available_at(&self, published: DateTime<Utc>) -> DateTime<Utc> {
        published + self.min_age
    }
}

#[async_trait]
pub trait Gate: Send + Sync + 'static {
    async fn check(&self, ctx: &Ctx, asset: &Asset) -> Result<(), ContentError>;

    async fn check_digest(
        &self,
        ctx: &Ctx,
        repository: &str,
        digest: &str,
    ) -> Result<(), ContentError>;

    async fn cooldown(&self, _ctx: &Ctx) -> Result<Option<Cooldown>, ContentError> {
        Ok(None)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Open;

#[async_trait]
impl Gate for Open {
    async fn check(&self, _ctx: &Ctx, _asset: &Asset) -> Result<(), ContentError> {
        Ok(())
    }

    async fn check_digest(
        &self,
        _ctx: &Ctx,
        _repository: &str,
        _digest: &str,
    ) -> Result<(), ContentError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use astrum_policy::exception::{Exception, Rule};
    use chrono::{TimeDelta, Utc};
    use uuid::Uuid;

    use super::Cooldown;

    fn excepted(name: &str, version: Option<&str>) -> Exception {
        Exception {
            id: Uuid::new_v4(),
            policy: None,
            rule: Rule::Cooldown,
            advisory: None,
            license: None,
            format: None,
            namespace: Some("@globalart".to_owned()),
            name: Some(name.to_owned()),
            version: version.map(str::to_owned),
            digest: None,
            reason: "audited".to_owned(),
            expires_at: None,
            created_by: "tester".to_owned(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn the_window_hides_blocked_components_that_are_not_excluded_or_excepted() {
        let now = Utc::now();
        let cooldown = Cooldown {
            cutoff: now - TimeDelta::days(7),
            min_age: TimeDelta::days(7),
            block: true,
            exclude: vec!["@internal/*".to_owned()],
            excepted: vec![excepted("ui", Some("1.2.0"))],
        };
        let recent = Some(now - TimeDelta::days(1));
        let old = Some(now - TimeDelta::days(30));
        assert!(cooldown.hides("@globalart/left", "@globalart", "left", "1.0.0", recent));
        assert!(!cooldown.hides("@globalart/left", "@globalart", "left", "1.0.0", old));
        assert!(!cooldown.hides("@globalart/left", "@globalart", "left", "1.0.0", None));
        assert!(!cooldown.hides("@internal/left", "@internal", "left", "1.0.0", recent));
        assert!(!cooldown.hides("@globalart/ui", "@globalart", "ui", "1.2.0", recent));
        assert!(cooldown.hides("@globalart/ui", "@globalart", "ui", "1.3.0", recent));
        let report = Cooldown {
            block: false,
            ..cooldown.clone()
        };
        assert!(!report.hides("@globalart/left", "@globalart", "left", "1.0.0", recent));
        assert_eq!(
            cooldown.available_at(now - TimeDelta::days(1)),
            now + TimeDelta::days(6)
        );
    }
}
