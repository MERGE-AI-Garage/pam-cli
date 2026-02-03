//! Skills management commands

use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::SkillsAction;
use crate::api;

pub async fn handle(action: SkillsAction, config: &Config, verbose: bool) -> Result<()> {
    match action {
        SkillsAction::List { detailed } => list(detailed, config, verbose).await,
        SkillsAction::Test { skill, params } => test(&skill, params, config, verbose).await,
        SkillsAction::Invoke { skill, params, user } => invoke(&skill, &params, user, config, verbose).await,
        SkillsAction::Log { skill, limit } => log(skill, limit, config, verbose).await,
    }
}

async fn list(detailed: bool, config: &Config, verbose: bool) -> Result<()> {
    println!("{}", "PAM Skills".bold());
    println!("{}", "─".repeat(40));

    match api::client::list_skills(&config.api_url).await {
        Ok(skills) => {
            for skill in &skills {
                let status_icon = if skill.enabled { "✓".green() } else { "○".dimmed() };
                let risk_badge = match skill.risk_level.as_str() {
                    "safe" => "safe".green(),
                    "moderate" => "moderate".yellow(),
                    _ => skill.risk_level.normal(),
                };

                println!("\n{} {} [{}]", status_icon, skill.skill_key.bold(), risk_badge);

                if detailed || verbose {
                    println!("    {}", skill.description.dimmed());
                    println!("    Usage: {} invocations", skill.usage_count);
                }
            }
            println!("\n{} {} skills available", "✓".green(), skills.len());
        }
        Err(e) => {
            println!("{} Failed to list skills: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn test(skill: &str, params: Option<String>, config: &Config, verbose: bool) -> Result<()> {
    println!("{}", format!("Testing Skill: {}", skill).bold());
    println!("{}", "─".repeat(40));

    let test_params = params.unwrap_or_else(|| get_default_test_params(skill));

    if verbose {
        println!("Test params: {}", test_params);
    }

    println!("Running test...\n");

    let start = std::time::Instant::now();

    match api::client::invoke_skill(&config.api_url, skill, &test_params, Some("test@mergeworld.com")).await {
        Ok(result) => {
            let duration = start.elapsed();

            println!("{} Skill executed successfully", "✓".green());
            println!("Duration: {}ms", duration.as_millis());

            if let Some(content) = result.get("content").and_then(|v| v.as_str()) {
                println!("\n{}", "Output:".bold());
                // Show first 500 chars
                let preview = if content.len() > 500 {
                    format!("{}...", &content[..500])
                } else {
                    content.to_string()
                };
                println!("{}", preview);
            } else {
                println!("\n{}", "Result:".bold());
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        Err(e) => {
            println!("{} Skill test failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn invoke(skill: &str, params: &str, user: Option<String>, config: &Config, verbose: bool) -> Result<()> {
    let user_email = user.or(config.user_email.clone()).unwrap_or_else(|| "unknown@mergeworld.com".to_string());

    if verbose {
        println!("Invoking {} as {}", skill, user_email);
        println!("Params: {}", params);
    }

    println!("Invoking {}...", skill.bold());

    match api::client::invoke_skill(&config.api_url, skill, params, Some(&user_email)).await {
        Ok(result) => {
            println!("{} Skill completed", "✓".green());

            if let Some(content) = result.get("content").and_then(|v| v.as_str()) {
                println!("\n{}", content);
            } else {
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        Err(e) => {
            println!("{} Skill failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn log(skill: Option<String>, limit: usize, config: &Config, _verbose: bool) -> Result<()> {
    println!("{}", "Skill Audit Log".bold());
    println!("{}", "─".repeat(40));

    match api::client::get_skill_log(&config.api_url, skill.as_deref(), limit).await {
        Ok(entries) => {
            if entries.is_empty() {
                println!("{}", "No log entries found.".yellow());
            } else {
                for entry in &entries {
                    let status_icon = if entry.success { "✓".green() } else { "✗".red() };
                    println!(
                        "{} {} {} ({}ms) - {}",
                        status_icon,
                        entry.skill_key.bold(),
                        entry.user_email.dimmed(),
                        entry.duration_ms,
                        entry.created_at
                    );
                }
            }
        }
        Err(e) => {
            println!("{} Failed to get skill log: {}", "✗".red(), e);
        }
    }

    Ok(())
}

/// Get default test parameters for each skill
fn get_default_test_params(skill: &str) -> String {
    match skill {
        "jira-query" => r#"{"query": "What Jira projects exist?"}"#.to_string(),
        "github-commits" => r#"{"query": "Show recent commits"}"#.to_string(),
        "daily-ambition" => r#"{"query": "What did the team accomplish?"}"#.to_string(),
        "web-fetch" => r#"{"url": "https://www.mergeworld.com/about"}"#.to_string(),
        "pam-memory" => r#"{"query_type": "team_member", "search_term": "Stephen"}"#.to_string(),
        "freebusy" => r#"{"emails": ["mwood@mergeworld.com"], "date": "2026-01-30"}"#.to_string(),
        "jira-create" => r#"{"project_key": "AIGAR", "summary": "Test", "description": "Test issue"}"#.to_string(),
        _ => r#"{}"#.to_string(),
    }
}
