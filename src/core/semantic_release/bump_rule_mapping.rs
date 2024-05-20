use super::BumpRule;
use crate::{CommitType, SimpleVersion};

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

pub fn get_rule(mut rules: impl Iterator<Item = (CommitType, BumpRule)>, commit_type: impl Into<CommitType>) -> BumpRule {
    let commit_type = commit_type.into();
    tracing::debug!("Finding rule for {commit_type:?}");
    rules
        .find(|(t, _)| *t == commit_type)
        .map(|(_, r)| r)
        .unwrap_or(BumpRule::Notset)
}

pub fn bump_version(mut rules: impl Iterator<Item = (CommitType, BumpRule)>, commit_type: impl Into<CommitType>, version: impl Into<SimpleVersion>) -> SimpleVersion {
    let commit_type = commit_type.into();
    let version = version.into();
    let rule = get_rule(rules.by_ref(), commit_type);
    tracing::debug!("Bumping version with rule: {rule:?}");
    version.bump(rule)
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
    #[case::empty(build_default_rules(), "", "1.0.0", "1.0.0")]
    #[case::build(build_default_rules(), "build", "1.0.0", "1.0.0")]
    #[case::chore(build_default_rules(), "chore", "1.0.0", "1.0.1")]
    #[case::ci(build_default_rules(), "ci", "1.0.0", "1.0.0")]
    #[case::cd(build_default_rules(), "cd", "1.0.0", "1.0.0")]
    #[case::docs(build_default_rules(), "docs", "1.0.0", "1.0.0")]
    #[case::feat(build_default_rules(), "feat", "1.0.0", "1.1.0")]
    #[case::fix(build_default_rules(), "fix", "1.0.0", "1.0.1")]
    #[case::perf(build_default_rules(), "perf", "1.0.0", "1.0.1")]
    #[case::refactor(build_default_rules(), "refactor", "1.0.0", "1.0.1")]
    #[case::revert(build_default_rules(), "revert", "1.0.0", "1.0.1")]
    #[case::style(build_default_rules(), "style", "1.0.0", "1.0.1")]
    #[case::test(build_default_rules(), "test", "1.0.0", "1.0.0")]
    #[case::custom(custom_rules(), "ENG-2345", "1.0.0", "2.0.0")]
    fn test_bump_version(
        #[case] rules: impl Iterator<Item = (CommitType, BumpRule)>,
        #[case] commit_type: impl Into<CommitType>,
        #[case] version: impl Into<SimpleVersion>,
        #[case] expected: impl Into<SimpleVersion>,
    ) {
        let commit_type = commit_type.into();
        let version = version.into();
        let expected = expected.into();
        assert_eq!(bump_version(rules, commit_type, version), expected);
    }
}
