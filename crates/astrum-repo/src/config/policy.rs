use serde::{Deserialize, Serialize};

use crate::config::problems::Problems;
use crate::config::{Check, Context};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub name: Option<String>,
}

impl Check for Policy {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        problems.require_text("policyName", &self.name);
    }
}
