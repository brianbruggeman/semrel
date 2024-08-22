use crate::{BumpRule, BumpRuleConfig, CommitType};

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]

pub struct SemRelConfig {
    semrel: SemRel,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct SemRel {
    rules: BumpRuleConfig,
}

impl SemRel {
    pub fn has_rules(&self) -> bool {
        !self.rules.is_empty()
    }

    pub fn extend_rules(&mut self, rules: &[(CommitType, BumpRule)]) {
        self.rules.extend(rules);
    }
}

impl SemRelConfig {
    pub fn has_rules(&self) -> bool {
        self.semrel.has_rules()
    }

    pub fn rules(&self) -> impl IntoIterator<Item = (CommitType, BumpRule)> {
        self.semrel.rules.clone().into_iter()
    }

    pub fn extend_rules(&mut self, rules: &[(CommitType, BumpRule)]) {
        self.semrel.extend_rules(rules);
    }
}
