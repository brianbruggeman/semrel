fn first_word_is_hex(s: &str) -> bool {
    let word = s.split_whitespace().next().unwrap_or("");
    !word.is_empty() && word.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_git_header(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    let trailers = ["co-authored-by:", "change-id:", "reviewed-by:"];
    if trailers.iter().any(|p| lower.starts_with(p)) {
        return true;
    }
    let headers = ["author ", "commit ", "committer ", "date ", "parent ", "tree "];
    headers.iter().any(|prefix| {
        lower.starts_with(prefix) && {
            let rest = &lower[prefix.len()..];
            first_word_is_hex(rest) || rest.contains('<')
        }
    })
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
    #[case::author("author John Doe <john@doe.com> 1612345678 +0000\n\nchore: test", "chore: test")]
    #[case::change_id("Change-Id: I1234567890\n\n\n\nchore: test the change-id", "chore: test the change-id")]
    #[case::commit("commit abc123def456\n\n\n\nchore: test the commit", "chore: test the commit")]
    #[case::committer("committer Jane <jane@doe.com> 1612345678 +0000\n\nchore: test", "chore: test")]
    #[case::date("date 1612345678 +0000\n\nchore: test the date", "chore: test the date")]
    #[case::parent("parent abc123def456\n\nchore: test the parent", "chore: test the parent")]
    #[case::reviewed_by("reviewed-by: John Doe\n\nchore: test the reviewed-by", "chore: test the reviewed-by")]
    #[case::tree("tree abc123def456\n\nchore: test the tree", "chore: test the tree")]
    #[case::spaced_header("           chore: test the spaced header", "chore: test the spaced header")]
    #[case::body_with_merge("chore: test\n\nMerge the two modules", "chore: test\n\nMerge the two modules")]
    #[case::body_with_date("feat: thing\n\nDate format was wrong", "feat: thing\n\nDate format was wrong")]
    #[case::body_with_commit("fix: bug\n\nCommit to quality", "fix: bug\n\nCommit to quality")]
    #[case::merge_commit_subject("Merge branch 'feature' into main", "Merge branch 'feature' into main")]
    #[case::merge_pr_subject("Merge pull request #123 from user/branch", "Merge pull request #123 from user/branch")]
    #[case::merge_commit_with_body("Merge branch 'feature'\n\n* feat: add feature\n* fix: fix bug", "Merge branch 'feature'\n\n* feat: add feature\n* fix: fix bug")]
    #[case::subject_starts_with_date("Date handling improvements", "Date handling improvements")]
    #[case::subject_starts_with_parent("Parent class refactored", "Parent class refactored")]
    #[case::subject_starts_with_commit("Commit to quality", "Commit to quality")]
    #[case::subject_starts_with_author("Author page redesign", "Author page redesign")]
    #[case::subject_starts_with_tree("Tree structure rewrite", "Tree structure rewrite")]
    fn test_prune_message(#[case] input: &str, #[case] expected: &str) {
        let result = prune_message(input);
        assert_eq!(result, expected);
    }
}
