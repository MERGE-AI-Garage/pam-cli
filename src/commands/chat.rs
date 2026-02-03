//! Interactive chat with PAM

use anyhow::Result;
use colored::Colorize;
use dialoguer::Input;

use crate::config::Config;
use crate::api;

pub async fn handle(
    message: Option<String>,
    user: Option<String>,
    continue_session: bool,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    let user_email = user.or(config.user_email.clone()).unwrap_or_else(|| {
        println!("{} No user email specified. Use --user or set PAM_USER_EMAIL", "⚠".yellow());
        "unknown@mergeworld.com".to_string()
    });

    // Get or create session ID
    let session_id = if continue_session {
        // Try to get most recent session
        match api::client::get_latest_session(&config.api_url, &user_email).await {
            Ok(Some(sid)) => {
                println!("{} Continuing session: {}", "•".cyan(), sid);
                sid
            }
            _ => {
                println!("{} No previous session found, starting new one", "•".cyan());
                generate_session_id()
            }
        }
    } else {
        generate_session_id()
    };

    if let Some(msg) = message {
        // Single message mode
        send_message(&config.api_url, &user_email, &session_id, &msg, verbose).await
    } else {
        // Interactive mode
        interactive_chat(&config.api_url, &user_email, &session_id, verbose).await
    }
}

async fn send_message(
    api_url: &str,
    user_email: &str,
    session_id: &str,
    message: &str,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Session: {}", session_id);
        println!("User: {}", user_email);
        println!("Message: {}", message);
    }

    println!("{} {}", "You:".bold(), message);
    println!();

    // Show thinking indicator
    print!("{}", "PAM is thinking...".dimmed());
    std::io::Write::flush(&mut std::io::stdout())?;

    match api::client::chat(&api_url, user_email, session_id, message).await {
        Ok(response) => {
            // Clear thinking indicator
            print!("\r{}", " ".repeat(20));
            print!("\r");

            println!("{}", "PAM:".bold().cyan());
            println!("{}", response);
        }
        Err(e) => {
            print!("\r");
            println!("{} Chat failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn interactive_chat(
    api_url: &str,
    user_email: &str,
    session_id: &str,
    verbose: bool,
) -> Result<()> {
    println!("{}", "╔════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║  PAM Chief of Staff - Interactive Chat                     ║".cyan());
    println!("{}", "║  Type 'quit' or 'exit' to end, 'clear' to reset session    ║".cyan());
    println!("{}", "╚════════════════════════════════════════════════════════════╝".cyan());
    println!();
    println!("Session: {}", session_id.dimmed());
    println!("User: {}", user_email.dimmed());
    println!();

    let mut current_session = session_id.to_string();

    loop {
        let input: String = Input::new()
            .with_prompt("You")
            .interact_text()?;

        let trimmed = input.trim();

        // Handle special commands
        match trimmed.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                println!("\n{} Goodbye!", "👋".to_string());
                break;
            }
            "clear" => {
                current_session = generate_session_id();
                println!("{} Started new session: {}", "✓".green(), current_session);
                continue;
            }
            "help" => {
                print_help();
                continue;
            }
            "/reflect" => {
                println!("{}", "Generating reflection...".dimmed());
                // Trigger reflection
                match api::client::generate_reflection(api_url, user_email, &[current_session.clone()]).await {
                    Ok(reflection) => {
                        println!("\n{}", "Reflection:".bold().cyan());
                        for learning in &reflection.learnings {
                            println!("  💡 {}", learning);
                        }
                    }
                    Err(e) => println!("{} Reflection failed: {}", "✗".red(), e),
                }
                continue;
            }
            "/status" => {
                println!("Session: {}", current_session);
                println!("User: {}", user_email);
                continue;
            }
            "" => continue,
            _ => {}
        }

        // Send message to PAM
        println!();
        print!("{}", "PAM is thinking...".dimmed());
        std::io::Write::flush(&mut std::io::stdout())?;

        match api::client::chat(api_url, user_email, &current_session, trimmed).await {
            Ok(response) => {
                // Clear thinking indicator
                print!("\r{}", " ".repeat(20));
                print!("\r");

                println!("{}", "PAM:".bold().cyan());
                println!("{}", response);
                println!();
            }
            Err(e) => {
                print!("\r");
                println!("{} Error: {}", "✗".red(), e);
                println!();
            }
        }
    }

    Ok(())
}

fn generate_session_id() -> String {
    format!(
        "cli_{}_{:08x}",
        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
        rand::random::<u32>()
    )
}

fn print_help() {
    println!("\n{}", "Commands:".bold());
    println!("  {}      - End the chat session", "quit, exit, q".cyan());
    println!("  {}          - Start a new session", "clear".cyan());
    println!("  {}       - Generate reflection from this session", "/reflect".cyan());
    println!("  {}        - Show current session info", "/status".cyan());
    println!("  {}           - Show this help", "help".cyan());
    println!();
}
