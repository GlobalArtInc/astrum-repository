use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlobId(Uuid);

impl BlobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn uuid(&self) -> Uuid {
        self.0
    }

    pub fn location(&self) -> String {
        let text = self.to_string();
        format!("{}/{}/{text}", &text[0..2], &text[2..4])
    }
}

impl Default for BlobId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for BlobId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_hyphenated().fmt(f)
    }
}

impl FromStr for BlobId {
    type Err = uuid::Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(text).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_spread_over_two_levels_of_directories() {
        let id: BlobId = "0a1b2c3d-0000-4000-8000-000000000000".parse().unwrap();
        assert_eq!(id.location(), "0a/1b/0a1b2c3d-0000-4000-8000-000000000000");
        assert_eq!(id.to_string().parse::<BlobId>().unwrap(), id);
        assert_ne!(BlobId::new(), BlobId::new());
        assert!("not-a-uuid".parse::<BlobId>().is_err());
    }
}
