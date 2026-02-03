//! Memory management commands

use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::MemoryAction;
use crate::api;

pub async fn handle(action: MemoryAction, config: &Config, verbose: bool) -> Result<()> {
    match action {
        MemoryAction::Status { deep } => status(deep, config, verbose).await,
        MemoryAction::Search { query, limit, user } => search(&query, limit, user, config, verbose).await,
        MemoryAction::Index { content, file, tags } => index(content, file, tags, config, verbose).await,
        MemoryAction::List { limit, user } => list(limit, user, config, verbose).await,
        MemoryAction::Clear { user, force } => clear(&user, force, config, verbose).await,
    }
}

async fn status(deep: bool, config: &Config, verbose: bool) -> Result<()> {
    println!("{}", "PAM Memory Status".bold());
    println!("{}", "─".repeat(40));

    // Get memory stats from API
    match api::client::get_memory_status(&config.api_url).await {
        Ok(stats) => {
            println!("{} Memory system: {}", "•".green(), "Online".green());
            println!("  Total memories:    {}", stats.total_memories);
            println!("  Total sessions:    {}", stats.total_sessions);
            println!("  Total reflections: {}", stats.total_reflections);

            if deep {
                println!("\n{}", "Database Tables".bold());
                for table in &stats.tables {
                    println!("  {} {}: {} rows", "•".cyan(), table.name, table.row_count);
                }
            }
        }
        Err(e) => {
            println!("{} Memory system: {} - {}", "•".red(), "Error".red(), e);
        }
    }

    Ok(())
}

async fn search(query: &str, limit: usize, user: Option<String>, config: &Config, verbose: bool) -> Result<()> {
    if verbose {
        println!("Searching memories for: \"{}\"", query);
    }

    println!("{}", format!("Memory Search: \"{}\"", query).bold());
    println!("{}", "─".repeat(40));

    match api::client::search_memories(&config.api_url, query, limit, user.as_deref()).await {
        Ok(results) => {
            if results.is_empty() {
                println!("{}", "No memories found.".yellow());
            } else {
                for (i, result) in results.iter().enumerate() {
                    println!("\n{} {}", format!("[{}]", i + 1).cyan(), result.title.bold());
                    println!("    Session: {}", result.session_id);
                    println!("    Date:    {}", result.created_at);
                    println!("    Score:   {:.2}", result.relevance_score);
                    if verbose {
                        println!("    Preview: {}", &result.content[..result.content.len().min(200)]);
                    }
                }
                println!("\n{} {} memories found", "✓".green(), results.len());
            }
        }
        Err(e) => {
            println!("{} Search failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn index(content: Option<String>, file: Option<String>, tags: Vec<String>, config: &Config, verbose: bool) -> Result<()> {
    let text = match (content, file) {
        (Some(c), _) => c,
        (None, Some(f)) => std::fs::read_to_string(&f)?,
        (None, None) => {
            // Read from stdin
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    if verbose {
        println!("Indexing {} characters with tags: {:?}", text.len(), tags);
    }

    println!("Indexing content...");

    match api::client::index_memory(&config.api_url, &text, &tags).await {
        Ok(id) => {
            println!("{} Memory indexed with ID: {}", "✓".green(), id);
        }
        Err(e) => {
            println!("{} Indexing failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn list(limit: usize, user: Option<String>, config: &Config, verbose: bool) -> Result<()> {
    println!("{}", "Recent Memories".bold());
    println!("{}", "─".repeat(40));

    match api::client::list_memories(&config.api_url, limit, user.as_deref()).await {
        Ok(memories) => {
            if memories.is_empty() {
                println!("{}", "No memories found.".yellow());
            } else {
                for memory in &memories {
                    let age = chrono::Utc::now().signed_duration_since(memory.created_at);
                    let age_str = if age.num_hours() < 1 {
                        format!("{}m ago", age.num_minutes())
                    } else if age.num_days() < 1 {
                        format!("{}h ago", age.num_hours())
                    } else {
                        format!("{}d ago", age.num_days())
                    };

                    println!("{} {} ({})", "•".cyan(), memory.session_id, age_str.dimmed());
                    if verbose {
                        println!("    {}", &memory.preview);
                    }
                }
            }
        }
        Err(e) => {
            println!("{} Failed to list memories: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn clear(user: &str, force: bool, config: &Config, _verbose: bool) -> Result<()> {
    if !force {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Clear all memories for {}? This cannot be undone.", user))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    println!("Clearing memories for {}...", user);

    match api::client::clear_memories(&config.api_url, user).await {
        Ok(count) => {
            println!("{} Cleared {} memories", "✓".green(), count);
        }
        Err(e) => {
            println!("{} Failed to clear memories: {}", "✗".red(), e);
        }
    }

    Ok(())
}
