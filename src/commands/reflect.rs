//! Reflection loop commands - generate insights from conversations

use anyhow::Result;
use colored::Colorize;
use chrono::Utc;

use crate::config::Config;
use crate::api;

pub async fn handle(
    session: Option<String>,
    export: bool,
    user: Option<String>,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    let user_email = user.or(config.user_email.clone()).unwrap_or_else(|| {
        println!("{} No user email specified. Use --user or set PAM_USER_EMAIL", "⚠".yellow());
        "unknown@mergeworld.com".to_string()
    });

    println!("{}", "PAM Reflection Loop".bold());
    println!("{}", "─".repeat(40));
    println!("User: {}", user_email.cyan());

    if let Some(ref sid) = session {
        println!("Session: {}", sid);
    } else {
        println!("Scope: Today's sessions");
    }

    println!("\n{}", "Analyzing conversations...".dimmed());

    // Get sessions to reflect on
    let sessions = if let Some(sid) = session {
        vec![sid]
    } else {
        // Get today's sessions
        match api::client::get_today_sessions(&config.api_url, &user_email).await {
            Ok(s) => s,
            Err(e) => {
                println!("{} Failed to get sessions: {}", "✗".red(), e);
                return Ok(());
            }
        }
    };

    if sessions.is_empty() {
        println!("{}", "No sessions found to reflect on.".yellow());
        return Ok(());
    }

    if verbose {
        println!("Found {} sessions to analyze", sessions.len());
    }

    // Generate reflection
    println!("\n{}", "Generating reflection...".dimmed());

    match api::client::generate_reflection(&config.api_url, &user_email, &sessions).await {
        Ok(reflection) => {
            println!("{} Reflection generated", "✓".green());

            println!("\n{}", "═".repeat(50).cyan());
            println!("{}", "REFLECTION SUMMARY".bold().cyan());
            println!("{}", "═".repeat(50).cyan());

            println!("\n{}", "What Worked:".green().bold());
            for item in &reflection.what_worked {
                println!("  {} {}", "✓".green(), item);
            }

            println!("\n{}", "What Could Be Improved:".yellow().bold());
            for item in &reflection.what_failed {
                println!("  {} {}", "•".yellow(), item);
            }

            println!("\n{}", "Key Learnings:".cyan().bold());
            for learning in &reflection.learnings {
                println!("  {} {}", "💡".to_string(), learning);
            }

            if !reflection.action_items.is_empty() {
                println!("\n{}", "Action Items:".magenta().bold());
                for (i, item) in reflection.action_items.iter().enumerate() {
                    println!("  {}. {}", i + 1, item);
                }
            }

            println!("\n{}", "═".repeat(50).cyan());

            // Export if requested
            if export {
                let filename = format!(
                    "reflection_{}.md",
                    Utc::now().format("%Y%m%d_%H%M%S")
                );
                export_reflection(&filename, &reflection)?;
                println!("\n{} Exported to: {}", "✓".green(), filename);
            }

            // Save to database
            if verbose {
                println!("\nSaving reflection to database...");
            }

            match api::client::save_reflection(&config.api_url, &user_email, &reflection).await {
                Ok(id) => {
                    println!("{} Reflection saved (ID: {})", "✓".green(), id);
                }
                Err(e) => {
                    println!("{} Failed to save reflection: {}", "⚠".yellow(), e);
                }
            }
        }
        Err(e) => {
            println!("{} Reflection generation failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

fn export_reflection(filename: &str, reflection: &api::client::Reflection) -> Result<()> {
    let mut content = String::new();

    content.push_str(&format!("# PAM Reflection\n"));
    content.push_str(&format!("*Generated: {}*\n\n", Utc::now().format("%Y-%m-%d %H:%M UTC")));

    content.push_str("## What Worked\n");
    for item in &reflection.what_worked {
        content.push_str(&format!("- {}\n", item));
    }

    content.push_str("\n## What Could Be Improved\n");
    for item in &reflection.what_failed {
        content.push_str(&format!("- {}\n", item));
    }

    content.push_str("\n## Key Learnings\n");
    for learning in &reflection.learnings {
        content.push_str(&format!("- {}\n", learning));
    }

    if !reflection.action_items.is_empty() {
        content.push_str("\n## Action Items\n");
        for (i, item) in reflection.action_items.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", i + 1, item));
        }
    }

    std::fs::write(filename, content)?;
    Ok(())
}
