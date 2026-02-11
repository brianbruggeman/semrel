use crate::{BumpRule, CommitType};

pub fn parse_rules(rules: &[impl AsRef<str>]) -> anyhow::Result<impl Iterator<Item = (CommitType, BumpRule)> + '_> {
    let iter = rules
        .iter()
        .flat_map(|rule| rule.as_ref().split(','))
        .map(|rule| rule.split('=').take(2))
        .flat_map(|mut rule| {
            let commit_type = match &rule.next() {
                Some(commit_type) => CommitType::from(*commit_type),
                None => anyhow::bail!("No rule found."),
            };
            let bump_rule = match &rule.next() {
                Some(bump_rule) => BumpRule::try_from(*bump_rule).map_err(|why| anyhow::anyhow!("invalid bump rule for {commit_type}: {why}"))?,
                None => anyhow::bail!("Invalid rule for: {commit_type}"),
            };
            Ok((commit_type, bump_rule))
        });
    Ok(iter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::build(vec!["build=major"], vec![(CommitType::Build, BumpRule::Major)])]
    #[case::build_fix(vec!["build=major,fix=minor"], vec![(CommitType::Build, BumpRule::Major), (CommitType::Fix, BumpRule::Minor)])]
    fn test_parse_rules(#[case] rules: Vec<&str>, #[case] expected: Vec<(CommitType, BumpRule)>) {
        let rules = parse_rules(rules.as_slice()).unwrap().collect::<Vec<_>>();
        assert_eq!(rules.len(), expected.len(), "Rule count mismatch: got {}, expected {}", rules.len(), expected.len());
        for ((actual_commit_type, actual_bump_rule), (expected_commit_type, expected_commit_rule)) in rules.iter().zip(expected.iter()) {
            assert_eq!(actual_commit_type, expected_commit_type);
            assert_eq!(actual_bump_rule, expected_commit_rule);
        }
    }
}
