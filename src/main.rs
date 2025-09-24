use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber;

mod aggregator;
mod config;
mod github;

use aggregator::changelog_generator::OutputFormat;

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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // Create GitHub client
    let github_client = github::client::GitHubClient::new(cli.token.clone(), cli.org.clone()).await?;

    match cli.command {
        Commands::Generate {
            version,
            repos,
            output,
            format,
            include_prs,
            include_issues,
            categorize,
        } => {
            let config = aggregator::AggregatorConfig {
                include_prs,
                include_issues,
                categorize_commits: categorize,
                template_path: None,
            };

            let aggregator = aggregator::ReleaseAggregator::new(github_client, config);
            let release = aggregator.aggregate(&version, repos).await?;

            let generator = aggregator::changelog_generator::ChangelogGenerator::new(format, None)?;
            let content = generator.generate(&release)?;

            if let Some(output_path) = output {
                std::fs::write(output_path, content)?;
                println!("Release notes written successfully!");
            } else {
                println!("{}", content);
            }
        }
        Commands::Check { version, repos } => {
            println!("Checking release {} for repositories: {:?}", version, repos);
            
            let mut all_present = true;
            for repo in repos {
                let release = github_client.get_release(&repo, &version).await?;
                if release.is_some() {
                    println!("✓ {}: Release {} found", repo, version);
                } else {
                    println!("✗ {}: Release {} not found", repo, version);
                    all_present = false;
                }
            }
            
            if !all_present {
                std::process::exit(1);
            }
        }
        Commands::List { repos, limit } => {
            println!("Recent releases (limit: {}):", limit);
            println!();
            
            for repo in repos {
                println!("Repository: {}", repo);
                let releases = github_client.list_releases(&repo, limit).await?;
                
                if releases.is_empty() {
                    println!("  No releases found");
                } else {
                    for release in releases {
                        println!("  - {}: {}", release.tag_name, release.published_at.unwrap_or_default());
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}
