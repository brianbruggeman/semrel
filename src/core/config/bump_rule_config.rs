use std::collections::HashMap;

use crate::{BumpRule, CommitType};


#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BumpRuleConfig {
    #[serde(flatten)]
    rules: HashMap<CommitType, BumpRule>,
}

impl BumpRuleConfig {
    pub fn new(rules: &[(CommitType, BumpRule)]) -> Self {
        let rules = rules.iter().cloned().collect::<HashMap<_, _>>();
        Self { rules }
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn add(&mut self, commit_type: CommitType, bump_rule: BumpRule) {
        self.rules.insert(commit_type, bump_rule);
    }

    pub fn remove(&mut self, commit_type: CommitType) {
        self.rules.remove(&commit_type);
    }

    pub fn extend(&mut self, rules: &[(CommitType, BumpRule)])  {
        self.rules.extend(rules.into_iter().cloned());
    }

    pub fn iter<'a>(&'a self) -> impl IntoIterator<Item = (&'a CommitType, &'a BumpRule)> {
        self.rules.iter().map(|(c, b)| (c, b))
    }
}

impl IntoIterator for BumpRuleConfig {
    type Item = (CommitType, BumpRule);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter().map(|(a, b)| (a, b)).collect::<Vec<_>>().into_iter()
    }
}

impl<'a> IntoIterator for &'a BumpRuleConfig {
    type Item = (&'a CommitType, &'a BumpRule);
    type IntoIter = std::collections::hash_map::Iter<'a, CommitType, BumpRule>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.iter()
    }
}