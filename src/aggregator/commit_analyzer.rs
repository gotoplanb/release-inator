use crate::github::types::CommitInfo;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommitType {
    Feature,
    Fix,
    Documentation,
    Performance,
    Refactor,
    Test,
    Build,
    CI,
    Chore,
    Style,
    Other,
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitType::Feature => write!(f, "✨ Features"),
            CommitType::Fix => write!(f, "🐛 Bug Fixes"),
            CommitType::Documentation => write!(f, "📚 Documentation"),
            CommitType::Performance => write!(f, "⚡ Performance"),
            CommitType::Refactor => write!(f, "♻️ Refactoring"),
            CommitType::Test => write!(f, "✅ Tests"),
            CommitType::Build => write!(f, "📦 Build System"),
            CommitType::CI => write!(f, "👷 CI/CD"),
            CommitType::Chore => write!(f, "🔧 Chores"),
            CommitType::Style => write!(f, "💄 Style"),
            CommitType::Other => write!(f, "📝 Other Changes"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub commit_type: Option<CommitType>,
    pub breaking: bool,
    pub pr_number: Option<u64>,
    pub issues: Vec<u64>,
}

pub struct CommitAnalyzer;

impl CommitAnalyzer {
    pub fn analyze_commits(commits: Vec<CommitInfo>) -> Vec<EnrichedCommit> {
        commits
            .into_iter()
            .map(|commit| Self::analyze_single_commit(commit))
            .collect()
    }

    fn analyze_single_commit(commit: CommitInfo) -> EnrichedCommit {
        let (commit_type, breaking) = Self::parse_commit_message(&commit.message);
        let issues = Self::extract_issues(&commit.message);
        let pr_number = Self::extract_pr_number(&commit.message);

        EnrichedCommit {
            sha: commit.sha.clone(),
            message: Self::clean_message(&commit.message),
            author: commit.author.username.unwrap_or(commit.author.name),
            date: commit.date,
            commit_type,
            breaking,
            pr_number,
            issues,
        }
    }

    fn parse_commit_message(message: &str) -> (Option<CommitType>, bool) {
        let lower = message.to_lowercase();
        let first_line = lower.lines().next().unwrap_or("");
        
        let breaking = first_line.contains("breaking") || 
                       first_line.contains("!:") ||
                       message.contains("BREAKING CHANGE");

        let commit_type = if first_line.starts_with("feat") || first_line.starts_with("feature") {
            Some(CommitType::Feature)
        } else if first_line.starts_with("fix") || first_line.starts_with("bugfix") {
            Some(CommitType::Fix)
        } else if first_line.starts_with("docs") || first_line.starts_with("documentation") {
            Some(CommitType::Documentation)
        } else if first_line.starts_with("perf") || first_line.starts_with("performance") {
            Some(CommitType::Performance)
        } else if first_line.starts_with("refactor") {
            Some(CommitType::Refactor)
        } else if first_line.starts_with("test") || first_line.starts_with("tests") {
            Some(CommitType::Test)
        } else if first_line.starts_with("build") {
            Some(CommitType::Build)
        } else if first_line.starts_with("ci") || first_line.starts_with("cd") {
            Some(CommitType::CI)
        } else if first_line.starts_with("chore") {
            Some(CommitType::Chore)
        } else if first_line.starts_with("style") {
            Some(CommitType::Style)
        } else {
            None
        };

        (commit_type, breaking)
    }

    fn clean_message(message: &str) -> String {
        let first_line = message.lines().next().unwrap_or("");
        
        // Remove conventional commit prefixes
        let cleaned = first_line
            .trim_start_matches(|c: char| !c.is_alphabetic())
            .trim_start_matches("feat")
            .trim_start_matches("feature")
            .trim_start_matches("fix")
            .trim_start_matches("bugfix")
            .trim_start_matches("docs")
            .trim_start_matches("documentation")
            .trim_start_matches("perf")
            .trim_start_matches("performance")
            .trim_start_matches("refactor")
            .trim_start_matches("test")
            .trim_start_matches("tests")
            .trim_start_matches("build")
            .trim_start_matches("ci")
            .trim_start_matches("cd")
            .trim_start_matches("chore")
            .trim_start_matches("style")
            .trim_start_matches(":")
            .trim_start_matches("(")
            .trim_start_matches(")")
            .trim();

        // Capitalize first letter
        let mut chars = cleaned.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    fn extract_issues(message: &str) -> Vec<u64> {
        let mut issues = Vec::new();
        
        // Look for patterns like #123, fixes #456, closes #789
        let re = regex::Regex::new(r"(?:(?:fix|fixes|fixed|close|closes|closed|resolve|resolves|resolved)\s+)?#(\d+)").unwrap();
        
        for cap in re.captures_iter(message) {
            if let Some(issue_str) = cap.get(1) {
                if let Ok(issue_num) = issue_str.as_str().parse::<u64>() {
                    issues.push(issue_num);
                }
            }
        }
        
        issues.sort_unstable();
        issues.dedup();
        issues
    }

    fn extract_pr_number(message: &str) -> Option<u64> {
        // Look for patterns like (#123) at the end of commit messages
        let re = regex::Regex::new(r"\(#(\d+)\)").unwrap();
        
        if let Some(cap) = re.captures(message) {
            if let Some(pr_str) = cap.get(1) {
                return pr_str.as_str().parse::<u64>().ok();
            }
        }
        
        None
    }

    pub fn group_commits_by_type(commits: &[EnrichedCommit]) -> std::collections::HashMap<CommitType, Vec<&EnrichedCommit>> {
        let mut grouped = std::collections::HashMap::new();
        
        for commit in commits {
            if let Some(ref commit_type) = commit.commit_type {
                grouped.entry(commit_type.clone())
                    .or_insert_with(Vec::new)
                    .push(commit);
            }
        }
        
        grouped
    }
}