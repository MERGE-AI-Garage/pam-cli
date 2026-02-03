//! Context bundle management commands

use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::ContextAction;
use crate::api;

pub async fn handle(action: ContextAction, config: &Config, verbose: bool) -> Result<()> {
    match action {
        ContextAction::Status { freshness } => status(freshness, config, verbose).await,
        ContextAction::Refresh { force } => refresh(force, config, verbose).await,
        ContextAction::Show { name, raw } => show(&name, raw, config, verbose).await,
        ContextAction::List => list(config, verbose).await,
        ContextAction::Stats => stats(config, verbose).await,
    }
}

async fn status(freshness: bool, config: &Config, verbose: bool) -> Result<()> {
    println!("{}", "Context Bundle Status".bold());
    println!("{}", "─".repeat(40));

    match api::client::get_context_status(&config.api_url).await {
        Ok(status) => {
            println!("{} Context bundle: {}", "•".green(), "Available".green());
            println!("  Files:  {}", status.file_count);
            println!("  Size:   {:.2} KB", status.total_size_kb);
            println!("  Tokens: ~{}", status.estimated_tokens);

            if freshness || verbose {
                println!("\n{}", "File Freshness:".bold());
                for file in &status.files {
                    let freshness_icon = if file.age_minutes < 30.0 {
                        "🟢".to_string()
                    } else if file.age_minutes < 60.0 {
                        "🟡".to_string()
                    } else {
                        "🔴".to_string()
                    };

                    println!(
                        "  {} {} ({:.0}m old, {:.1} KB)",
                        freshness_icon,
                        file.name,
                        file.age_minutes,
                        file.size_kb
                    );
                }
            }
        }
        Err(e) => {
            println!("{} Context status failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn refresh(force: bool, config: &Config, verbose: bool) -> Result<()> {
    if verbose {
        println!("Refreshing context bundle (force={})", force);
    }

    println!("Refreshing context from GCS...");

    match api::client::refresh_context(&config.api_url, force).await {
        Ok(result) => {
            println!("{} Context refreshed", "✓".green());
            println!("  Files loaded: {}", result.files_loaded);
            println!("  Total size:   {:.2} KB", result.total_size_kb);
        }
        Err(e) => {
            println!("{} Refresh failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn show(name: &str, raw: bool, config: &Config, _verbose: bool) -> Result<()> {
    // Map friendly names to actual file names
    let filename = match name.to_lowercase().as_str() {
        "github" | "git" => "github_ai_garage.md",
        "jira" => "jira_summary.md",
        "daily" | "ambition" | "daily-ambition" => "daily_ambitions_summary.md",
        "strategic" => "strategic_context_30min.md",
        "tactical" => "tactical_context_10min.md",
        "operational" => "operational_context_5min.md",
        "database" | "db" => "database_summary.md",
        _ => name,
    };

    match api::client::get_context_file(&config.api_url, filename).await {
        Ok(content) => {
            if raw {
                println!("{}", content);
            } else {
                println!("{}", format!("Context: {}", filename).bold());
                println!("{}", "─".repeat(40));
                println!("{}", content);
            }
        }
        Err(e) => {
            println!("{} Failed to load context file: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn list(config: &Config, _verbose: bool) -> Result<()> {
    println!("{}", "Context Files".bold());
    println!("{}", "─".repeat(40));

    match api::client::list_context_files(&config.api_url).await {
        Ok(files) => {
            println!("\n{}", "Real-Time Layers:".cyan());
            for f in files.iter().filter(|f| f.name.contains("context_")) {
                println!("  • {} ({:.1} KB)", f.name, f.size_kb);
            }

            println!("\n{}", "Project Data:".cyan());
            for f in files.iter().filter(|f| f.name.contains("summary") || f.name.contains("activity")) {
                println!("  • {} ({:.1} KB)", f.name, f.size_kb);
            }

            println!("\n{}", "Team Profiles:".cyan());
            for f in files.iter().filter(|f| f.name.contains("person") || f.name.contains("people/")) {
                println!("  • {} ({:.1} KB)", f.name, f.size_kb);
            }

            println!("\n{} {} files total", "✓".green(), files.len());
        }
        Err(e) => {
            println!("{} Failed to list context files: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn stats(config: &Config, _verbose: bool) -> Result<()> {
    println!("{}", "Context Bundle Statistics".bold());
    println!("{}", "─".repeat(40));

    match api::client::get_context_stats(&config.api_url).await {
        Ok(stats) => {
            println!("\n{}", "Size Breakdown:".cyan());
            println!("  Total Size:      {:.2} KB", stats.total_size_kb);
            println!("  Estimated Tokens: ~{}", stats.estimated_tokens);

            println!("\n{}", "By Category:".cyan());
            println!("  Real-Time:   {:.1} KB ({:.0}%)", stats.realtime_kb, stats.realtime_pct);
            println!("  Projects:    {:.1} KB ({:.0}%)", stats.projects_kb, stats.projects_pct);
            println!("  Team:        {:.1} KB ({:.0}%)", stats.team_kb, stats.team_pct);
            println!("  Activity:    {:.1} KB ({:.0}%)", stats.activity_kb, stats.activity_pct);

            println!("\n{}", "Team Members:".cyan());
            for member in &stats.team_members {
                println!("  • {}", member);
            }
        }
        Err(e) => {
            println!("{} Failed to get context stats: {}", "✗".red(), e);
        }
    }

    Ok(())
}
