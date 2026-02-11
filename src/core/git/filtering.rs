const GIT_HEADER_PREFIXES: &[&str] = &[
    "author",
    "co-authored-by",
    "change-id",
    "commit",
    "committer",
    "date",
    "merge",
    "parent",
    "reviewed-by",
    "tree",
];

fn is_git_header(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    GIT_HEADER_PREFIXES.iter().any(|&prefix| lower.starts_with(prefix))
}

pub fn prune_message(message: impl AsRef<str>) -> String {
    let mut past_preamble = false;
    message
        .as_ref()
        .lines()
        .filter(|line| {
            if past_preamble {
                return true;
            }
            if line.trim().is_empty() {
                return true;
            }
            if is_git_header(line) {
                tracing::debug!("Pruning: {line:?}");
                return false;
            }
            past_preamble = true;
            true
        })
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::empty("", "")]
    #[case::empty_header("\n\nchore: test the empty header", "chore: test the empty header")]
    #[case::empty_footer("chore: test the empty footer\n\n", "chore: test the empty footer")]
    #[case::author("Author: John Doe\n\n\n\nchore: test the author", "chore: test the author")]
    #[case::change_id("Change-Id: I1234567890\n\n\n\nchore: test the change-id", "chore: test the change-id")]
    #[case::commit("commit: 1234567890\n\n\n\nchore: test the commit", "chore: test the commit")]
    #[case::committer("committer: John Doe\n\n\n\nchore: test the committer", "chore: test the committer")]
    #[case::date("date: 2021-01-01\n\nchore: test the date", "chore: test the date")]
    #[case::parent("parent: 1234567890\n\nchore: test the parent", "chore: test the parent")]
    #[case::reviewed_by("reviewed-by: John Doe\n\nchore: test the reviewed-by", "chore: test the reviewed-by")]
    #[case::tree("tree: 1234567890\n\nchore: test the tree", "chore: test the tree")]
    #[case::spaced_header("           chore: test the spaced header", "chore: test the spaced header")]
    #[case::body_with_merge("chore: test\n\nMerge the two modules", "chore: test\n\nMerge the two modules")]
    #[case::body_with_date("feat: thing\n\nDate format was wrong", "feat: thing\n\nDate format was wrong")]
    #[case::body_with_commit("fix: bug\n\nCommit to quality", "fix: bug\n\nCommit to quality")]
    fn test_prune_message(#[case] input: &str, #[case] expected: &str) {
        let result = prune_message(input);
        assert_eq!(result, expected);
    }
}
