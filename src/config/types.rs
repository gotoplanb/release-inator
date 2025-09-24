use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub github: GithubConfig,
    pub repos: ReposConfig,
    pub output: OutputConfig,
    pub features: FeaturesConfig,
    pub commit_types: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubConfig {
    pub org: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReposConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: String,
    pub path: String,
    pub template: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub categorize_commits: bool,
    pub include_prs: bool,
    pub include_issues: bool,
    pub include_stats: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut commit_types = HashMap::new();
        commit_types.insert("feat".to_string(), "✨ Features".to_string());
        commit_types.insert("fix".to_string(), "🐛 Bug Fixes".to_string());
        commit_types.insert("docs".to_string(), "📚 Documentation".to_string());
        commit_types.insert("perf".to_string(), "⚡ Performance".to_string());
        commit_types.insert("refactor".to_string(), "♻️ Refactoring".to_string());
        commit_types.insert("test".to_string(), "✅ Tests".to_string());
        commit_types.insert("build".to_string(), "📦 Build System".to_string());
        commit_types.insert("ci".to_string(), "👷 CI/CD".to_string());
        
        Config {
            github: GithubConfig {
                org: String::new(),
            },
            repos: ReposConfig {
                include: vec![],
                exclude: vec![],
            },
            output: OutputConfig {
                format: "markdown".to_string(),
                path: "releases".to_string(),
                template: None,
            },
            features: FeaturesConfig {
                categorize_commits: true,
                include_prs: true,
                include_issues: true,
                include_stats: true,
            },
            commit_types,
        }
    }
}