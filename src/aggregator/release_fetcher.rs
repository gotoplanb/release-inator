use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::github::client::GitHubClient;
use super::commit_analyzer::{CommitAnalyzer, EnrichedCommit};

#[derive(Debug)]
pub struct AggregatorConfig {
    pub include_prs: bool,
    pub include_issues: bool,
    pub categorize_commits: bool,
    pub template_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregatedRelease {
    pub version: String,
    pub date: DateTime<Utc>,
    pub components: Vec<ComponentRelease>,
    pub summary: ReleaseSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentRelease {
    pub repository: String,
    pub status: ComponentStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentStatus {
    Released {
        current_version: String,
        previous_version: Option<String>,
        release_date: DateTime<Utc>,
        commits: Vec<EnrichedCommit>,
        release_notes: Option<String>,
        stats: ReleaseStats,
    },
    NoRelease {
        latest_version: Option<String>,
        latest_date: Option<DateTime<Utc>>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseStats {
    pub commit_count: usize,
    pub contributors: Vec<String>,
    pub breaking_changes: usize,
    pub features: usize,
    pub fixes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseSummary {
    pub total_repos: usize,
    pub updated_repos: usize,
    pub total_commits: usize,
    pub contributors: Vec<String>,
}

pub struct ReleaseAggregator {
    client: GitHubClient,
    config: AggregatorConfig,
}

impl ReleaseAggregator {
    pub fn new(client: GitHubClient, config: AggregatorConfig) -> Self {
        Self { client, config }
    }

    pub async fn aggregate(&self, version: &str, repos: Vec<String>) -> Result<AggregatedRelease> {
        let mut components = Vec::new();
        let mut all_contributors = Vec::new();
        let mut total_commits = 0;
        let mut updated_repos = 0;

        // Process each repository
        for repo in &repos {
            let component = self.process_repository(repo, version).await?;
            
            // Collect stats
            match &component.status {
                ComponentStatus::Released { commits, stats, .. } => {
                    total_commits += commits.len();
                    all_contributors.extend(stats.contributors.clone());
                    updated_repos += 1;
                }
                _ => {}
            }
            
            components.push(component);
        }

        // Deduplicate contributors
        all_contributors.sort();
        all_contributors.dedup();

        let summary = ReleaseSummary {
            total_repos: repos.len(),
            updated_repos,
            total_commits,
            contributors: all_contributors,
        };

        Ok(AggregatedRelease {
            version: version.to_string(),
            date: Utc::now(),
            components,
            summary,
        })
    }

    async fn process_repository(&self, repo: &str, version: &str) -> Result<ComponentRelease> {
        // Try to get the release for this version
        let release = self.client.get_release(repo, version).await?;

        if let Some(release) = release {
            // Get the previous release to compare
            let previous_release = self.client.get_previous_release(repo, &release).await?;
            
            let commits = if let Some(prev) = &previous_release {
                // Get commits between releases
                self.client.get_commits_between(repo, &prev.tag_name, &release.tag_name).await?
            } else {
                // First release - get all commits up to this point
                self.client.get_all_commits_until(repo, &release.tag_name).await?
            };

            // Analyze commits
            let enriched_commits = if self.config.categorize_commits {
                CommitAnalyzer::analyze_commits(commits)
            } else {
                commits.into_iter().map(|c| EnrichedCommit {
                    sha: c.sha.clone(),
                    message: c.message.clone(),
                    author: c.author.username.unwrap_or(c.author.name),
                    date: c.date,
                    commit_type: None,
                    breaking: false,
                    pr_number: None,
                    issues: vec![],
                }).collect()
            };

            // Get PR information if requested
            let enriched_commits = if self.config.include_prs {
                let shas = enriched_commits.iter().map(|c| c.sha.clone()).collect();
                let prs = self.client.get_pull_requests_for_commits(repo, shas).await?;
                
                // Merge PR information into commits
                enriched_commits.into_iter().map(|mut commit| {
                    for pr in &prs {
                        if let Some(ref merge_sha) = pr.merge_commit_sha {
                            if merge_sha == &commit.sha {
                                commit.pr_number = Some(pr.number);
                            }
                        }
                    }
                    commit
                }).collect()
            } else {
                enriched_commits
            };

            // Calculate statistics
            let mut contributors: Vec<String> = enriched_commits.iter()
                .map(|c| c.author.clone())
                .collect();
            contributors.sort();
            contributors.dedup();

            let stats = ReleaseStats {
                commit_count: enriched_commits.len(),
                contributors: contributors.clone(),
                breaking_changes: enriched_commits.iter().filter(|c| c.breaking).count(),
                features: enriched_commits.iter()
                    .filter(|c| matches!(c.commit_type, Some(super::commit_analyzer::CommitType::Feature)))
                    .count(),
                fixes: enriched_commits.iter()
                    .filter(|c| matches!(c.commit_type, Some(super::commit_analyzer::CommitType::Fix)))
                    .count(),
            };

            Ok(ComponentRelease {
                repository: repo.to_string(),
                status: ComponentStatus::Released {
                    current_version: release.tag_name.clone(),
                    previous_version: previous_release.map(|r| r.tag_name),
                    release_date: release.created_at.unwrap_or_else(|| Utc::now()),
                    commits: enriched_commits,
                    release_notes: release.body.clone(),
                    stats,
                },
            })
        } else {
            // No release for this version - get the latest release info
            let latest = self.client.get_latest_release(repo).await?;

            Ok(ComponentRelease {
                repository: repo.to_string(),
                status: ComponentStatus::NoRelease {
                    latest_version: latest.as_ref().map(|r| r.tag_name.clone()),
                    latest_date: latest.and_then(|r| r.created_at),
                },
            })
        }
    }
}