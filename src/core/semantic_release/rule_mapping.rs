use crate::{BumpRule, CommitType};

pub fn parse_rules(rules: &[impl AsRef<str>]) -> anyhow::Result<impl Iterator<Item = (CommitType, BumpRule)> + '_> {
    let parsed = rules
        .iter()
        .flat_map(|rule| rule.as_ref().split(','))
        .map(|rule| {
            let mut parts = rule.split('=').take(2);
            let commit_type = match parts.next() {
                Some(ct) => CommitType::from(ct),
                None => anyhow::bail!("No rule found."),
            };
            let bump_rule = match parts.next() {
                Some(br) => BumpRule::try_from(br).map_err(|why| anyhow::anyhow!("invalid bump rule for {commit_type}: {why}"))?,
                None => anyhow::bail!("Invalid rule for: {commit_type}"),
            };
            Ok((commit_type, bump_rule))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(parsed.into_iter())
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

    #[rstest]
    #[case::invalid_bump(vec!["build=invalid"])]
    #[case::missing_bump(vec!["build"])]
    #[case::valid_then_invalid(vec!["build=major,fix=invalid"])]
    fn test_parse_rules_errors(#[case] rules: Vec<&str>) {
        let result = parse_rules(rules.as_slice());
        assert!(result.is_err(), "Expected error for rules: {rules:?}");
    }
}
