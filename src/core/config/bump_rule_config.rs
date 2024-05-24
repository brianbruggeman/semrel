
use crate::{BumpRule, CommitType};


#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct BumpRuleConfig {
    rules: Vec<(CommitType, BumpRule)>,
}

impl BumpRuleConfig {
    pub fn new(rules: Vec<(CommitType, BumpRule)>) -> Self {
        Self { rules }
    }

    pub fn add(&mut self, commit_type: CommitType, bump_rule: BumpRule) {
        self.rules.push((commit_type, bump_rule));
    }

    pub fn remove(&mut self, commit_type: CommitType, bump_rule: BumpRule) {
        self.rules.retain(|(ct, br)| !(ct == &commit_type && br == &bump_rule));
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (CommitType, BumpRule)> {
        self.rules.iter()
    }
}

impl IntoIterator for BumpRuleConfig {
    type Item = (CommitType, BumpRule);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter()
    }
}

impl<'a> IntoIterator for &'a BumpRuleConfig {
    type Item = &'a (CommitType, BumpRule);
    type IntoIter = std::iter::Map<std::slice::Iter<'a, (CommitType, BumpRule)>, fn(&'a (CommitType, BumpRule)) -> &'a (CommitType, BumpRule)>;

    fn into_iter(self) -> Self::IntoIter {
        fn deref<'a>(item: &'a (CommitType, BumpRule)) -> &'a (CommitType, BumpRule) {
            item
        }
        self.rules.iter().map(deref)
    }
}