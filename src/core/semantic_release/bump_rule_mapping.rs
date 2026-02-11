use super::BumpRule;
use crate::CommitType;

pub fn build_default_rules() -> impl Iterator<Item = (CommitType, BumpRule)> {
    let mapping = vec![
        (CommitType::Build, BumpRule::NoBump),
        (CommitType::Chore, BumpRule::Patch),
        (CommitType::Ci, BumpRule::NoBump),
        (CommitType::Cd, BumpRule::NoBump),
        (CommitType::Docs, BumpRule::NoBump),
        (CommitType::Feat, BumpRule::Minor),
        (CommitType::Fix, BumpRule::Patch),
        (CommitType::Perf, BumpRule::Patch),
        (CommitType::Refactor, BumpRule::Patch),
        (CommitType::Revert, BumpRule::Patch),
        (CommitType::Style, BumpRule::Patch),
        (CommitType::Test, BumpRule::NoBump),
    ];
    mapping.into_iter()
}

pub fn match_rule(rules: impl IntoIterator<Item = (CommitType, BumpRule)>, commit_type: impl Into<CommitType>) -> BumpRule {
    let commit_type = commit_type.into();
    tracing::trace!("Searching for bump rule for: {commit_type:?}");
    let rule = rules.into_iter().find(|(t, _)| *t == commit_type).map(|(_, r)| r);
    match rule {
        Some(rule) => {
            tracing::trace!("Found rule: {rule:?}");
            rule
        }
        None => {
            tracing::trace!("No rule found for `{commit_type:?}`, using default rule: {:?}", BumpRule::default());
            BumpRule::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn custom_rules() -> impl Iterator<Item = (CommitType, BumpRule)> {
        let mapping = vec![(CommitType::Custom("ENG-2345".to_string()), BumpRule::Major)];
        mapping.into_iter()
    }

    #[rstest]
    #[case::empty(build_default_rules(), "", BumpRule::Notset)]
    #[case::build(build_default_rules(), "build", BumpRule::NoBump)]
    #[case::chore(build_default_rules(), "chore", BumpRule::Patch)]
    #[case::ci(build_default_rules(), "ci", BumpRule::NoBump)]
    #[case::cd(build_default_rules(), "cd", BumpRule::NoBump)]
    #[case::docs(build_default_rules(), "docs", BumpRule::NoBump)]
    #[case::feat(build_default_rules(), "feat", BumpRule::Minor)]
    #[case::fix(build_default_rules(), "fix", BumpRule::Patch)]
    #[case::perf(build_default_rules(), "perf", BumpRule::Patch)]
    #[case::refactor(build_default_rules(), "refactor", BumpRule::Patch)]
    #[case::revert(build_default_rules(), "revert", BumpRule::Patch)]
    #[case::style(build_default_rules(), "style", BumpRule::Patch)]
    #[case::test(build_default_rules(), "test", BumpRule::NoBump)]
    #[case::custom(custom_rules(), "ENG-2345", BumpRule::Major)]
    fn test_match_rule(
        #[case] rules: impl Iterator<Item = (CommitType, BumpRule)>,
        #[case] commit_type: impl Into<CommitType>,
        #[case] expected: BumpRule,
    ) {
        assert_eq!(match_rule(rules, commit_type), expected);
    }
}
