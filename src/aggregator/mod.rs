pub mod release_fetcher;
pub mod commit_analyzer;
pub mod changelog_generator;

pub use release_fetcher::{ReleaseAggregator, AggregatorConfig, AggregatedRelease};
pub use commit_analyzer::CommitType;