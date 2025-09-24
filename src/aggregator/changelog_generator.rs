use anyhow::Result;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::collections::HashMap;
use super::release_fetcher::{AggregatedRelease, ComponentStatus};
use super::commit_analyzer::{CommitType, EnrichedCommit};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Markdown,
    Json,
    Html,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "json" => Ok(OutputFormat::Json),
            "html" => Ok(OutputFormat::Html),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

pub struct ChangelogGenerator {
    template_engine: Handlebars<'static>,
    format: OutputFormat,
}

impl ChangelogGenerator {
    pub fn new(format: OutputFormat, template_path: Option<PathBuf>) -> Result<Self> {
        let mut template_engine = Handlebars::new();
        
        // Register helper to check if status is released
        template_engine.register_helper(
            "eq",
            Box::new(|h: &handlebars::Helper,
                     _: &Handlebars,
                     _: &handlebars::Context,
                     _: &mut handlebars::RenderContext,
                     out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
                let param1 = h.param(0).and_then(|v| v.value().as_str());
                let param2 = h.param(1).and_then(|v| v.value().as_str());
                
                if param1 == param2 {
                    out.write("true")?;
                }
                Ok(())
            }),
        );

        // Register default template if no custom one provided
        if template_path.is_none() {
            let default_template = include_str!("../../templates/default.md.hbs");
            template_engine.register_template_string("default", default_template)?;
        } else {
            let template_content = std::fs::read_to_string(template_path.as_ref().unwrap())?;
            template_engine.register_template_string("custom", &template_content)?;
        }

        Ok(Self {
            template_engine,
            format,
        })
    }

    pub fn generate(&self, release: &AggregatedRelease) -> Result<String> {
        match self.format {
            OutputFormat::Markdown => self.generate_markdown(release),
            OutputFormat::Json => self.generate_json(release),
            OutputFormat::Html => self.generate_html(release),
        }
    }

    fn generate_markdown(&self, release: &AggregatedRelease) -> Result<String> {
        // Convert to JSON for template rendering
        let mut data = json!({
            "version": release.version,
            "date": release.date.format("%Y-%m-%d").to_string(),
            "summary": {
                "total_repos": release.summary.total_repos,
                "updated_repos": release.summary.updated_repos,
                "total_commits": release.summary.total_commits,
                "contributors": release.summary.contributors.len(),
            },
            "components": Vec::<serde_json::Value>::new(),
        });

        // Process components
        if let Some(components) = data.get_mut("components") {
            if let Some(components_array) = components.as_array_mut() {
                for component in &release.components {
                    let comp_data = match &component.status {
                        ComponentStatus::Released {
                            current_version,
                            previous_version,
                            release_date,
                            commits,
                            release_notes,
                            stats,
                        } => {
                            let grouped_commits = self.group_commits_by_type(commits);
                            json!({
                                "repository": component.repository,
                                "status": "Released",
                                "current_version": current_version,
                                "previous_version": previous_version,
                                "release_date": release_date.format("%Y-%m-%d").to_string(),
                                "commits": commits.iter().map(|c| json!({
                                    "sha": &c.sha[..7],
                                    "message": c.message,
                                    "author": c.author,
                                    "pr_number": c.pr_number,
                                    "issues": c.issues,
                                })).collect::<Vec<_>>(),
                                "grouped_commits": grouped_commits,
                                "release_notes": release_notes,
                                "stats": {
                                    "commit_count": stats.commit_count,
                                    "contributors": stats.contributors,
                                    "breaking_changes": stats.breaking_changes,
                                    "features": stats.features,
                                    "fixes": stats.fixes,
                                }
                            })
                        }
                        ComponentStatus::NoRelease {
                            latest_version,
                            latest_date,
                        } => {
                            json!({
                                "repository": component.repository,
                                "status": "NoRelease",
                                "latest_version": latest_version,
                                "latest_date": latest_date.map(|d| d.format("%Y-%m-%d").to_string()),
                            })
                        }
                    };
                    components_array.push(comp_data);
                }
            }
        }

        // Use template or fallback to simple format
        if self.template_engine.has_template("custom") {
            Ok(self.template_engine.render("custom", &data)?)
        } else if self.template_engine.has_template("default") {
            Ok(self.template_engine.render("default", &data)?)
        } else {
            // Fallback to simple markdown
            Ok(self.generate_simple_markdown(release))
        }
    }

    fn generate_simple_markdown(&self, release: &AggregatedRelease) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# Release {}\n\n", release.version));
        output.push_str(&format!("📅 **Date:** {}\n\n", release.date.format("%Y-%m-%d")));
        
        output.push_str("## 📊 Summary\n\n");
        output.push_str(&format!("- **Total Repositories:** {}\n", release.summary.total_repos));
        output.push_str(&format!("- **Updated Repositories:** {}\n", release.summary.updated_repos));
        output.push_str(&format!("- **Total Commits:** {}\n", release.summary.total_commits));
        output.push_str(&format!("- **Contributors:** {}\n\n", release.summary.contributors.len()));
        
        output.push_str("---\n\n");
        
        for component in &release.components {
            output.push_str(&format!("## {}\n\n", component.repository));
            
            match &component.status {
                ComponentStatus::Released {
                    current_version,
                    previous_version,
                    release_date,
                    commits,
                    release_notes,
                    stats,
                } => {
                    output.push_str(&format!("**Version:** `{}`  \n", current_version));
                    if let Some(prev) = previous_version {
                        output.push_str(&format!("**Previous:** `{}`  \n", prev));
                    } else {
                        output.push_str("**Previous:** *Initial Release*  \n");
                    }
                    output.push_str(&format!("**Release Date:** {}  \n", release_date.format("%Y-%m-%d")));
                    output.push_str(&format!("**Commits:** {}  \n\n", stats.commit_count));
                    
                    if !commits.is_empty() {
                        output.push_str("### 🎯 Changes\n\n");
                        
                        let grouped = self.group_commits_by_type(commits);
                        if !grouped.is_empty() {
                            for (commit_type, type_commits) in grouped {
                                output.push_str(&format!("#### {}\n", commit_type));
                                for commit in type_commits {
                                    output.push_str(&format!("- {} ([`{}`])\n", 
                                        commit.message, 
                                        &commit.sha[..7]
                                    ));
                                }
                                output.push_str("\n");
                            }
                        } else {
                            for commit in commits {
                                output.push_str(&format!("- {} ([`{}`])\n", 
                                    commit.message, 
                                    &commit.sha[..7]
                                ));
                            }
                            output.push_str("\n");
                        }
                    }
                    
                    if let Some(notes) = release_notes {
                        output.push_str("### 📝 Release Notes\n\n");
                        output.push_str(notes);
                        output.push_str("\n\n");
                    }
                    
                    if !stats.contributors.is_empty() {
                        output.push_str("### 👥 Contributors\n");
                        for contributor in &stats.contributors {
                            output.push_str(&format!("- @{}\n", contributor));
                        }
                        output.push_str("\n");
                    }
                }
                ComponentStatus::NoRelease {
                    latest_version,
                    latest_date,
                } => {
                    output.push_str("*No changes in this release*\n\n");
                    if let Some(latest) = latest_version {
                        output.push_str(&format!("Latest version: `{}`", latest));
                        if let Some(date) = latest_date {
                            output.push_str(&format!(" ({})", date.format("%Y-%m-%d")));
                        }
                        output.push_str("\n\n");
                    }
                }
            }
            
            output.push_str("---\n\n");
        }
        
        output
    }

    fn generate_json(&self, release: &AggregatedRelease) -> Result<String> {
        Ok(serde_json::to_string_pretty(release)?)
    }

    fn generate_html(&self, release: &AggregatedRelease) -> Result<String> {
        // Convert markdown to HTML
        let markdown = self.generate_markdown(release)?;
        let parser = pulldown_cmark::Parser::new(&markdown);
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);
        
        // Wrap in basic HTML structure
        Ok(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Release {}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; max-width: 900px; margin: 0 auto; padding: 20px; }}
        h1, h2, h3 {{ border-bottom: 1px solid #e1e4e8; padding-bottom: 0.3em; }}
        code {{ background: #f6f8fa; padding: 2px 4px; border-radius: 3px; }}
    </style>
</head>
<body>
    {}
</body>
</html>"#,
            release.version,
            html
        ))
    }

    fn group_commits_by_type<'a>(&self, commits: &'a [EnrichedCommit]) -> HashMap<CommitType, Vec<&'a EnrichedCommit>> {
        let mut grouped: HashMap<CommitType, Vec<&'a EnrichedCommit>> = HashMap::new();
        
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