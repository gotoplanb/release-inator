# Detailed Implementation Plan: GitHub Multi-Repository Release Aggregator

## Project Overview

Create a Rust-based tool that aggregates release information from multiple GitHub repositories into unified release notes. The tool will be designed to run as a GitHub Action and store results in a dedicated repository.

## Project Structure

```
release-aggregator/
├── Cargo.toml
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── aggregate-release.yml    # Manual trigger workflow
│       └── auto-aggregate.yml       # Auto-trigger on releases
├── src/
│   ├── main.rs                      # CLI entry point
│   ├── lib.rs                       # Library exports
│   ├── github/
│   │   ├── mod.rs
│   │   ├── client.rs                # GitHub API client wrapper
│   │   └── types.rs                 # GitHub API types
│   ├── aggregator/
│   │   ├── mod.rs
│   │   ├── release_fetcher.rs       # Fetch release information
│   │   ├── commit_analyzer.rs       # Analyze commits between releases
│   │   └── changelog_generator.rs   # Generate output formats
│   └── config/
│       ├── mod.rs
│       └── types.rs                 # Configuration structures
├── templates/
│   └── release.md.hbs               # Handlebars template for markdown
└── examples/
    ├── config.toml                  # Example configuration file
    └── output/
        └── 1.0.0.md                 # Example output
```

## Core Dependencies

Add to `Cargo.toml`:

```toml
[package]
name = "release-aggregator"
version = "0.1.0"
edition = "2021"

[dependencies]
# GitHub API
octocrab = "0.32"

# CLI
clap = { version = "4.4", features = ["derive", "env"] }

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Templates
handlebars = "5.0"

# Markdown processing (optional, for enhanced output)
pulldown-cmark = "0.9"
pulldown-cmark-to-cmark = "11.0"

[dev-dependencies]
mockito = "1.2"
pretty_assertions = "1.4"
```

## Detailed Component Specifications

### 1. CLI Interface (`src/main.rs`)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "release-aggregator")]
#[command(about = "Aggregate release notes from multiple GitHub repositories")]
struct Cli {
    /// GitHub token (can also be set via GITHUB_TOKEN env var)
    #[arg(long, env = "GITHUB_TOKEN")]
    token: String,

    /// Organization or user name
    #[arg(short, long, env = "GITHUB_ORG")]
    org: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate release notes for a specific version
    Generate {
        /// Version/tag name to aggregate
        #[arg(short, long)]
        version: String,

        /// Comma-separated list of repository names
        #[arg(short, long, value_delimiter = ',')]
        repos: Vec<String>,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(short = 'f', long, default_value = "markdown")]
        format: OutputFormat,

        /// Include PR links
        #[arg(long)]
        include_prs: bool,

        /// Include issue links
        #[arg(long)]
        include_issues: bool,

        /// Categorize commits by type (feat, fix, etc.)
        #[arg(long)]
        categorize: bool,
    },

    /// Check if all repos have a specific release
    Check {
        #[arg(short, long)]
        version: String,
        
        #[arg(short, long, value_delimiter = ',')]
        repos: Vec<String>,
    },

    /// List recent releases across repositories
    List {
        #[arg(short, long, value_delimiter = ',')]
        repos: Vec<String>,
        
        #[arg(long, default_value = "10")]
        limit: usize,
    },
}
```

### 2. GitHub Client Wrapper (`src/github/client.rs`)

```rust
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

    pub async fn get_release(&self, repo: &str, tag: &str) -> Result<Option<Release>> {
        // Implementation: Try to fetch release by tag
        // Return None if not found instead of error
    }

    pub async fn get_latest_release(&self, repo: &str) -> Result<Option<Release>> {
        // Get the most recent release
    }

    pub async fn get_previous_release(&self, repo: &str, current_release: &Release) -> Result<Option<Release>> {
        // Get the release immediately before the current one by date
    }

    pub async fn get_commits_between(&self, repo: &str, from: &str, to: &str) -> Result<Vec<CommitInfo>> {
        // Use compare API to get commits between two tags/SHAs
    }

    pub async fn get_all_commits_until(&self, repo: &str, until: &str) -> Result<Vec<CommitInfo>> {
        // Get all commits up to a certain tag (for first releases)
    }

    pub async fn get_pull_requests_for_commits(&self, repo: &str, shas: Vec<String>) -> Result<Vec<PullRequest>> {
        // Optional: Get PR information for commits
    }
}
```

### 3. Release Aggregator (`src/aggregator/release_fetcher.rs`)

```rust
pub struct ReleaseAggregator {
    client: GitHubClient,
    config: AggregatorConfig,
}

#[derive(Debug)]
pub struct AggregatorConfig {
    pub include_prs: bool,
    pub include_issues: bool,
    pub categorize_commits: bool,
    pub template_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct AggregatedRelease {
    pub version: String,
    pub date: DateTime<Utc>,
    pub components: Vec<ComponentRelease>,
    pub summary: ReleaseSummary,
}

#[derive(Debug)]
pub struct ComponentRelease {
    pub repository: String,
    pub status: ComponentStatus,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct EnrichedCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub commit_type: Option<CommitType>,  // feat, fix, docs, etc.
    pub breaking: bool,
    pub pr_number: Option<u64>,
    pub issues: Vec<u64>,
}

impl ReleaseAggregator {
    pub async fn aggregate(&self, version: &str, repos: Vec<String>) -> Result<AggregatedRelease> {
        // Main aggregation logic
    }

    async fn process_repository(&self, repo: &str, version: &str) -> Result<ComponentRelease> {
        // Process a single repository
    }

    fn categorize_commits(&self, commits: Vec<CommitInfo>) -> Vec<EnrichedCommit> {
        // Parse commit messages and categorize them
        // Support conventional commits format
    }
}
```

### 4. Changelog Generator (`src/aggregator/changelog_generator.rs`)

```rust
pub struct ChangelogGenerator {
    template_engine: Handlebars,
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Markdown,
    Json,
    Html,
}

impl ChangelogGenerator {
    pub fn new(format: OutputFormat, template_path: Option<PathBuf>) -> Result<Self> {
        // Initialize with default or custom template
    }

    pub fn generate(&self, release: &AggregatedRelease) -> Result<String> {
        match self.format {
            OutputFormat::Markdown => self.generate_markdown(release),
            OutputFormat::Json => self.generate_json(release),
            OutputFormat::Html => self.generate_html(release),
        }
    }

    fn generate_markdown(&self, release: &AggregatedRelease) -> Result<String> {
        // Generate formatted markdown
        // Use template engine for customization
    }

    fn group_commits_by_type(&self, commits: &[EnrichedCommit]) -> HashMap<CommitType, Vec<&EnrichedCommit>> {
        // Group commits for better organization
    }
}
```

### 5. Template (`templates/release.md.hbs`)

```handlebars
# Release {{version}}

📅 **Date:** {{date}}

## 📊 Summary

- **Total Repositories:** {{summary.total_repos}}
- **Updated Repositories:** {{summary.updated_repos}}
- **Total Commits:** {{summary.total_commits}}
- **Contributors:** {{summary.contributors}}

---

{{#each components}}
## {{repository}}

{{#if (is_released status)}}
**Version:** `{{status.current_version}}`  
**Previous:** {{#if status.previous_version}}`{{status.previous_version}}`{{else}}*Initial Release*{{/if}}  
**Release Date:** {{status.release_date}}  
**Commits:** {{status.stats.commit_count}}  

{{#if status.commits}}
### 🎯 Changes

{{#if ../categorize}}
{{#each (group_by_type status.commits)}}
#### {{@key}}
{{#each this}}
- {{message}} ([`{{short_sha}}`](https://github.com/{{../../../org}}/{{../../repository}}/commit/{{sha}})){{#if pr_number}} (#{{pr_number}}){{/if}}
{{/each}}

{{/each}}
{{else}}
{{#each status.commits}}
- {{message}} ([`{{short_sha}}`](https://github.com/{{../../org}}/{{../repository}}/commit/{{sha}}))
{{/each}}
{{/if}}
{{/if}}

{{#if status.release_notes}}
### 📝 Release Notes

{{status.release_notes}}
{{/if}}

{{#if status.stats.contributors}}
### 👥 Contributors
{{#each status.stats.contributors}}
- @{{this}}
{{/each}}
{{/if}}

{{else}}
*No changes in this release*

{{#if status.latest_version}}
Latest version: `{{status.latest_version}}` ({{status.latest_date}})
{{/if}}
{{/if}}

---
{{/each}}

## 🔗 Links

{{#each components}}
{{#if (is_released status)}}
- [{{repository}} v{{status.current_version}}](https://github.com/{{../org}}/{{repository}}/releases/tag/{{status.current_version}})
{{/if}}
{{/each}}
```

### 6. GitHub Actions Workflows

**`.github/workflows/aggregate-release.yml`:**

```yaml
name: Aggregate Release Notes

on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Release version to aggregate'
        required: true
        type: string
      repos:
        description: 'Comma-separated list of repositories'
        required: true
        default: 'foo,bar,baz'
      output_path:
        description: 'Output path for release notes'
        required: false
        default: 'releases'

jobs:
  aggregate:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
      
      - name: Cache Cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Build Release Aggregator
        run: cargo build --release
      
      - name: Generate Release Notes
        run: |
          ./target/release/release-aggregator generate \
            --version "${{ inputs.version }}" \
            --repos "${{ inputs.repos }}" \
            --org "${{ github.repository_owner }}" \
            --output "${{ inputs.output_path }}/${{ inputs.version }}.md" \
            --categorize \
            --include-prs \
            --include-issues
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          RUST_LOG: info
      
      - name: Commit Release Notes
        run: |
          git config --local user.email "github-actions[bot]@users.noreply.github.com"
          git config --local user.name "github-actions[bot]"
          git add "${{ inputs.output_path }}/"
          git commit -m "📝 Add release notes for v${{ inputs.version }}" || echo "No changes to commit"
          git push
```

## Testing Strategy

### Unit Tests
- Mock GitHub API responses using `mockito`
- Test commit categorization logic
- Test markdown generation
- Test template rendering

### Integration Tests
- Test against real GitHub API with test repositories
- Test rate limiting handling
- Test error scenarios (missing releases, API errors)

### Example Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Mock};

    #[tokio::test]
    async fn test_get_release_when_exists() {
        let _m = mock("GET", "/repos/test-org/test-repo/releases/tags/v1.0.0")
            .with_status(200)
            .with_body(r#"{"tag_name": "v1.0.0", "name": "Version 1.0.0"}"#)
            .create();

        // Test implementation
    }

    #[tokio::test]
    async fn test_aggregate_with_missing_releases() {
        // Test that missing releases are handled gracefully
    }
}
```

## Configuration Options

Create a `release-aggregator.toml` for defaults:

```toml
[github]
org = "my-org"

[repos]
include = ["foo", "bar", "baz"]
exclude = []

[output]
format = "markdown"
path = "releases"
template = "templates/release.md.hbs"

[features]
categorize_commits = true
include_prs = true
include_issues = true
include_stats = true

[commit_types]
feat = "✨ Features"
fix = "🐛 Bug Fixes"
docs = "📚 Documentation"
perf = "⚡ Performance"
refactor = "♻️ Refactoring"
test = "✅ Tests"
build = "📦 Build System"
ci = "👷 CI/CD"
```

## Error Handling

- Gracefully handle missing releases (don't fail, note as "No changes")
- Implement retry logic for GitHub API rate limits
- Provide clear error messages for configuration issues
- Log all API calls for debugging
- Handle network timeouts and transient failures

## Performance Considerations

- Implement parallel fetching for multiple repositories
- Cache GitHub API responses where appropriate
- Use GitHub's GraphQL API for bulk operations if needed
- Implement pagination for large commit ranges
- Consider using conditional requests (ETags) for caching

## Future Enhancements to Consider

1. **Webhook Support**: Auto-trigger on release events
2. **Diff Links**: Generate comparison URLs between versions
3. **Statistics Dashboard**: Generate charts/graphs of release activity
4. **Notification System**: Send release notes to Slack/Discord/email
5. **Version Compatibility Matrix**: Track which versions work together
6. **Change Risk Assessment**: Analyze commits for potential breaking changes
7. **Multi-format Export**: Support additional formats (RSS, CHANGELOG.md)
8. **Incremental Updates**: Only regenerate changed components

## Getting Started Commands for Claude Code

```bash
# Initialize the project
cargo init release-aggregator
cd release-aggregator

# Add dependencies to Cargo.toml (see above)

# Create the directory structure
mkdir -p src/{github,aggregator,config} templates examples .github/workflows

# Start implementing core components in order:
# 1. CLI structure and argument parsing
# 2. GitHub client wrapper
# 3. Basic release fetching
# 4. Commit analysis
# 5. Markdown generation
# 6. GitHub Actions workflow

# Test with a simple case first
cargo run -- generate --version "1.0.0" --repos "test-repo" --org "test-org"
```

This plan provides a comprehensive foundation for implementing the release aggregator. The modular structure allows you to build and test components incrementally, starting with basic functionality and adding features as needed.