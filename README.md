# 🚀 Release Aggregator

A powerful Rust-based CLI tool that aggregates release information from multiple GitHub repositories into unified, beautifully formatted release notes. Perfect for organizations managing multiple related repositories that need consolidated release documentation.

## ✨ Features

- **Multi-Repository Aggregation**: Combine releases from multiple GitHub repos into a single document
- **Conventional Commit Support**: Automatically categorizes commits by type (feat, fix, docs, etc.)
- **Multiple Output Formats**: Generate Markdown, JSON, or HTML output
- **Customizable Templates**: Use Handlebars templates for personalized formatting
- **GitHub Actions Integration**: Automate release note generation in your CI/CD pipeline
- **Rich Release Information**: Include commit details, contributors, PR links, and issue references
- **Smart Release Detection**: Handle missing releases gracefully and provide comprehensive status checks

## 📋 Table of Contents

- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Commands](#-commands)
- [Configuration](#-configuration)
- [GitHub Actions Integration](#-github-actions-integration)
- [Output Examples](#-output-examples)
- [Contributing](#-contributing)

## 🛠 Installation

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **GitHub CLI** (optional): For easier authentication with `gh auth login`
- **GitHub Personal Access Token**: With `repo` scope permissions

### Build from Source

```bash
# Clone the repository
git clone https://github.com/gotoplanb/release-inator.git
cd release-inator

# Build the release binary
cargo build --release

# Binary will be available at ./target/release/release-aggregator
```

### GitHub Token Setup

```bash
# Option 1: Use GitHub CLI (recommended)
gh auth login

# Option 2: Set environment variable
export GITHUB_TOKEN=your_token_here

# Option 3: Pass token via CLI flag
./target/release/release-aggregator --token YOUR_TOKEN ...
```

## 🚀 Quick Start

```bash
# List recent releases across repositories
./target/release/release-aggregator --org "your-org" list --repos "repo1,repo2,repo3"

# Check if all repositories have a specific release
./target/release/release-aggregator --org "your-org" check --version "v1.0.0" --repos "repo1,repo2,repo3"

# Generate comprehensive release notes
./target/release/release-aggregator --org "your-org" generate \
  --version "v1.0.0" \
  --repos "repo1,repo2,repo3" \
  --output "releases/v1.0.0.md" \
  --categorize \
  --include-prs \
  --include-issues
```

## 📖 Commands

### `generate` - Create Release Notes

Generate aggregated release notes for a specific version across multiple repositories.

```bash
release-aggregator --org ORG generate [OPTIONS]
```

**Options:**
- `-v, --version <VERSION>` - Version/tag name to aggregate (required)
- `-r, --repos <REPOS>` - Comma-separated list of repository names (required)
- `-o, --output <PATH>` - Output file path (prints to stdout if not specified)
- `-f, --format <FORMAT>` - Output format: `markdown` (default), `json`, or `html`
- `--categorize` - Categorize commits by conventional commit types
- `--include-prs` - Include pull request links
- `--include-issues` - Include issue references

**Example:**
```bash
release-aggregator --org "acme-corp" generate \
  --version "v2.1.0" \
  --repos "frontend,backend,mobile-app" \
  --output "releases/v2.1.0.md" \
  --categorize \
  --include-prs
```

### `check` - Verify Release Presence

Check if all specified repositories have a particular release.

```bash
release-aggregator --org ORG check --version VERSION --repos REPOS
```

**Example:**
```bash
release-aggregator --org "acme-corp" check \
  --version "v2.1.0" \
  --repos "frontend,backend,mobile-app"
```

**Output:**
```
✓ frontend: Release v2.1.0 found
✓ backend: Release v2.1.0 found  
✗ mobile-app: Release v2.1.0 not found
```

### `list` - Show Recent Releases

List the most recent releases across repositories.

```bash
release-aggregator --org ORG list --repos REPOS [--limit N]
```

**Options:**
- `--limit <N>` - Number of releases to show per repository (default: 10)

**Example:**
```bash
release-aggregator --org "acme-corp" list \
  --repos "frontend,backend,mobile-app" \
  --limit 5
```

## ⚙️ Configuration

### Environment Variables

```bash
# GitHub authentication
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
export GITHUB_ORG=your-organization

# Logging level
export RUST_LOG=info
```

### Configuration File (Optional)

Create `release-aggregator.toml` in your project root:

```toml
[github]
org = "your-org"

[repos]
include = ["repo1", "repo2", "repo3"]
exclude = []

[output]
format = "markdown"
path = "releases"
template = "templates/custom.md.hbs"

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

### Custom Templates

Create custom Handlebars templates in the `templates/` directory:

```handlebars
# Release {{version}}

📅 **Date:** {{date}}

## Summary
- **Repositories Updated:** {{summary.updated_repos}}/{{summary.total_repos}}
- **Total Commits:** {{summary.total_commits}}

{{#each components}}
## {{repository}}
{{#if (eq status "Released")}}
**Changes since {{previous_version}}:**
{{#each commits}}
- {{message}} ([{{sha}}])
{{/each}}
{{/if}}
{{/each}}
```

## 🔄 GitHub Actions Integration

Add the provided workflow to `.github/workflows/aggregate-release.yml`:

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
        default: 'repo1,repo2,repo3'

jobs:
  aggregate:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        
      - name: Build Release Aggregator
        run: cargo build --release
        
      - name: Generate Release Notes
        run: |
          ./target/release/release-aggregator generate \
            --version "${{ inputs.version }}" \
            --repos "${{ inputs.repos }}" \
            --org "${{ github.repository_owner }}" \
            --output "releases/${{ inputs.version }}.md" \
            --categorize \
            --include-prs \
            --include-issues
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          
      - name: Commit Release Notes
        run: |
          git config --local user.email "github-actions[bot]@users.noreply.github.com"
          git config --local user.name "github-actions[bot]"
          git add releases/
          git commit -m "📝 Add release notes for ${{ inputs.version }}" || exit 0
          git push
```

**Usage:** Go to Actions tab → "Aggregate Release Notes" → "Run workflow"

## 📊 Output Examples

### Markdown Output

```markdown
# Release v2.1.0

📅 **Date:** 2024-01-15

## 📊 Summary

- **Total Repositories:** 3
- **Updated Repositories:** 3
- **Total Commits:** 15
- **Contributors:** 5

---

## frontend

**Version:** `v2.1.0`  
**Previous:** `v2.0.3`  
**Release Date:** 2024-01-15  
**Commits:** 8  

### ✨ Features
- Add dark mode support ([`abc123`]) (#45)
- Implement user preferences panel ([`def456`])

### 🐛 Bug Fixes  
- Fix responsive layout on mobile ([`ghi789`]) (#67)

### 👥 Contributors
- @alice
- @bob
- @charlie
```

### JSON Output

```json
{
  "version": "v2.1.0",
  "date": "2024-01-15T10:30:00Z",
  "components": [
    {
      "repository": "frontend",
      "status": {
        "Released": {
          "current_version": "v2.1.0",
          "previous_version": "v2.0.3",
          "commits": [
            {
              "sha": "abc123...",
              "message": "Add dark mode support",
              "author": "alice",
              "commit_type": "Feature"
            }
          ],
          "stats": {
            "commit_count": 8,
            "contributors": ["alice", "bob"]
          }
        }
      }
    }
  ]
}
```

## 🎯 Use Cases

- **Multi-Service Applications**: Aggregate releases from microservices
- **Library Ecosystems**: Combine releases from related packages  
- **Platform Releases**: Unify frontend, backend, and mobile app releases
- **Documentation**: Generate comprehensive changelogs for stakeholders
- **Release Planning**: Verify all components are ready for release

## 🔧 Development

### Running Tests

```bash
# Run unit tests
cargo test

# Run integration tests with real GitHub API
cargo test --test integration -- --ignored
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [octocrab](https://github.com/XAMPPRocky/octocrab)
- Templating powered by [Handlebars](https://handlebarsjs.com/)
- Inspired by conventional commit standards

---

**Made with ❤️ by the Release Aggregator team**