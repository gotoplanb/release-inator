use anyhow::Result;
use octocrab::Octocrab;
use octocrab::models;
use super::types::{CommitInfo, CommitAuthor, PullRequest};

pub struct GitHubClient {
    client: Octocrab,
    org: String,
}

impl GitHubClient {
    pub async fn new(token: String, org: String) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()?;
        Ok(Self { client, org })
    }

    pub async fn get_release(&self, repo: &str, tag: &str) -> Result<Option<models::repos::Release>> {
        let result = self.client
            .repos(&self.org, repo)
            .releases()
            .get_by_tag(tag)
            .await;

        match result {
            Ok(release) => Ok(Some(release)),
            Err(octocrab::Error::GitHub { source, .. }) if source.message.contains("Not Found") => {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_latest_release(&self, repo: &str) -> Result<Option<models::repos::Release>> {
        let result = self.client
            .repos(&self.org, repo)
            .releases()
            .get_latest()
            .await;

        match result {
            Ok(release) => Ok(Some(release)),
            Err(octocrab::Error::GitHub { source, .. }) if source.message.contains("Not Found") => {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn list_releases(&self, repo: &str, limit: usize) -> Result<Vec<models::repos::Release>> {
        let releases = self.client
            .repos(&self.org, repo)
            .releases()
            .list()
            .per_page(limit as u8)
            .send()
            .await?;

        Ok(releases.items)
    }

    pub async fn get_previous_release(&self, repo: &str, current_release: &models::repos::Release) -> Result<Option<models::repos::Release>> {
        let releases = self.client
            .repos(&self.org, repo)
            .releases()
            .list()
            .per_page(100)
            .send()
            .await?;

        let current_date = current_release.created_at;
        
        // Find the release immediately before the current one by date
        let mut previous: Option<models::repos::Release> = None;
        for release in releases.items {
            if release.created_at < current_date {
                if previous.is_none() || release.created_at > previous.as_ref().unwrap().created_at {
                    previous = Some(release);
                }
            }
        }

        Ok(previous)
    }

    pub async fn get_commits_between(&self, repo: &str, from: &str, to: &str) -> Result<Vec<CommitInfo>> {
        // Get all commits for the 'to' ref
        let to_commits = self.client
            .repos(&self.org, repo)
            .list_commits()
            .sha(to)
            .per_page(100)
            .send()
            .await?;

        // Get all commits for the 'from' ref
        let from_commits = self.client
            .repos(&self.org, repo)
            .list_commits()
            .sha(from)
            .per_page(100)
            .send()
            .await?;

        // Create a set of SHAs from the 'from' commits
        let from_shas: std::collections::HashSet<String> = from_commits.items
            .iter()
            .map(|c| c.sha.clone())
            .collect();

        // Filter to get commits that are in 'to' but not in 'from'
        let commits = to_commits.items
            .into_iter()
            .filter(|c| !from_shas.contains(&c.sha))
            .map(|commit| {
                let commit_data = commit.commit;
                CommitInfo {
                    sha: commit.sha.clone(),
                    message: commit_data.message.clone(),
                    author: CommitAuthor {
                        name: commit.author.as_ref().map(|a| a.login.clone()).unwrap_or_else(|| "Unknown".to_string()),
                        email: "".to_string(), // Email not directly available from API
                        username: commit.author.as_ref().map(|a| a.login.clone()),
                    },
                    date: commit_data.author.as_ref().and_then(|a| a.date).unwrap_or_else(|| chrono::Utc::now()),
                }
            })
            .collect();

        Ok(commits)
    }

    pub async fn get_all_commits_until(&self, repo: &str, until: &str) -> Result<Vec<CommitInfo>> {
        // Get commits from the beginning up to the specified tag
        let commits_page = self.client
            .repos(&self.org, repo)
            .list_commits()
            .sha(until)
            .per_page(100)
            .send()
            .await?;

        let commits = commits_page.items
            .into_iter()
            .map(|commit| {
                let commit_data = commit.commit;
                CommitInfo {
                    sha: commit.sha.clone(),
                    message: commit_data.message.clone(),
                    author: CommitAuthor {
                        name: commit.author.as_ref().map(|a| a.login.clone()).unwrap_or_else(|| "Unknown".to_string()),
                        email: "".to_string(), // Email not directly available from API
                        username: commit.author.as_ref().map(|a| a.login.clone()),
                    },
                    date: commit_data.author.as_ref().and_then(|a| a.date).unwrap_or_else(|| chrono::Utc::now()),
                }
            })
            .collect();

        Ok(commits)
    }

    pub async fn get_pull_requests_for_commits(&self, repo: &str, shas: Vec<String>) -> Result<Vec<PullRequest>> {
        // This is a simplified implementation
        // In practice, you might need to search for PRs that contain these commits
        let mut prs = Vec::new();
        
        for sha in shas {
            // Try to find PRs associated with this commit
            let pr_search = self.client
                .search()
                .issues_and_pull_requests(&format!("repo:{}/{} sha:{}", self.org, repo, &sha[..7]))
                .send()
                .await;

            if let Ok(results) = pr_search {
                for item in results {
                    // Fetch full PR details
                    if let Ok(pr) = self.client
                        .pulls(&self.org, repo)
                        .get(item.number)
                        .await
                    {
                        prs.push(PullRequest {
                            number: pr.number,
                            title: pr.title.unwrap_or_default(),
                            body: pr.body,
                            merged_at: pr.merged_at,
                            merge_commit_sha: pr.merge_commit_sha,
                        });
                    }
                }
            }
        }

        Ok(prs)
    }
}