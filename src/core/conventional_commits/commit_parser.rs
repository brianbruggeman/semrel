use pest_derive::Parser as DeriveParser;

#[derive(DeriveParser)]
#[grammar = "src/core/conventional_commits/commit_message.pest"]
pub struct CommitMessageParser;
