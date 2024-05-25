pub fn prune_message(message: impl AsRef<str>) -> String {
    message
        .as_ref()
        .lines()
        .filter(|line| {
            let please_ignore = ["author", "co-authored-by", "change-id", "commit", "committer", "date", "merge", "parent", "reviewed-by", "tree"]
                .iter()
                .any(|&prefix| line.trim().to_ascii_lowercase().starts_with(prefix));
            // There could be a bunch of preamble here, but we're only interested in the conventional commit lines
            if please_ignore {
                tracing::debug!("Pruning: {line:?}");
            }
            !please_ignore
        })
        .map(|line| line.trim()) // make sure we can effectively trim empty lines around conventional commit lines
        .collect::<Vec<_>>()
        .join("\n")
        .trim() // sometimes there are newlines before the first conventional commit line that are empty
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
    fn test_prune_message(#[case] input: &str, #[case] expected: &str) {
        let result = prune_message(input);
        assert_eq!(result, expected);
    }
}
