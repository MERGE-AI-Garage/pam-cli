//! PAM CLI - Proactive Agentic Manager
//!
//! Command-line interface for PAM Chief of Staff.
//! Follows Maestro's CLI-first pattern: every capability testable from terminal.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
mod api;
mod config;

use commands::{memory, skills, context, reflect, chat};

/// PAM - Proactive Agentic Manager CLI
///
/// Chief of Staff for the AI Garage team at MERGE.
#[derive(Parser)]
#[command(name = "pam")]
#[command(author = "AI Garage <ai-garage@mergeworld.com>")]
#[command(version = "0.1.0")]
#[command(about = "PAM Chief of Staff CLI - Your AI-powered PM assistant", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true, env = "PAM_CONFIG")]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Memory management - search, index, and manage PAM's memory
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },

    /// Skills - list, test, and invoke PAM skills
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
    },

    /// Context - manage context bundles from GCS
    Context {
        #[command(subcommand)]
        action: ContextAction,
    },

    /// Reflect - generate insights from conversations
    Reflect {
        /// Session ID to reflect on (default: today's sessions)
        #[arg(short, long)]
        session: Option<String>,

        /// Export reflections to markdown file
        #[arg(short, long)]
        export: bool,

        /// User email to reflect for
        #[arg(short, long, env = "PAM_USER_EMAIL")]
        user: Option<String>,
    },

    /// Chat - interactive conversation with PAM
    Chat {
        /// The message to send (or omit for interactive mode)
        message: Option<String>,

        /// User email for context
        #[arg(short, long, env = "PAM_USER_EMAIL")]
        user: Option<String>,

        /// Continue previous session
        #[arg(short, long)]
        continue_session: bool,
    },

    /// Health - check PAM system health
    Health {
        /// Deep health check (probes all services)
        #[arg(short, long)]
        deep: bool,
    },

    /// Config - manage PAM CLI configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum MemoryAction {
    /// Show memory system status
    Status {
        /// Deep status check (probes vector + embedding availability)
        #[arg(short, long)]
        deep: bool,
    },

    /// Search memories semantically
    Search {
        /// The search query
        query: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// User email to search for
        #[arg(short, long)]
        user: Option<String>,
    },

    /// Index content into memory
    Index {
        /// Content to index (or - for stdin)
        content: Option<String>,

        /// File to index
        #[arg(short, long)]
        file: Option<String>,

        /// Tags for the memory
        #[arg(short, long)]
        tags: Vec<String>,
    },

    /// List recent memories
    List {
        /// Number of memories to list
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter by user
        #[arg(short, long)]
        user: Option<String>,
    },

    /// Clear memories (with confirmation)
    Clear {
        /// User email to clear (required)
        #[arg(short, long)]
        user: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum SkillsAction {
    /// List available skills
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Test a specific skill
    Test {
        /// Skill key to test (e.g., jira-query, github-commits)
        skill: String,

        /// Test parameters as JSON
        #[arg(short, long)]
        params: Option<String>,
    },

    /// Invoke a skill
    Invoke {
        /// Skill key to invoke
        skill: String,

        /// Parameters as JSON
        #[arg(short, long)]
        params: String,

        /// User email for audit
        #[arg(short, long, env = "PAM_USER_EMAIL")]
        user: Option<String>,
    },

    /// Show skill audit log
    Log {
        /// Skill key to filter by
        #[arg(short, long)]
        skill: Option<String>,

        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum ContextAction {
    /// Show context bundle status
    Status {
        /// Check freshness of all bundles
        #[arg(short, long)]
        freshness: bool,
    },

    /// Refresh context from GCS
    Refresh {
        /// Force refresh even if fresh
        #[arg(short, long)]
        force: bool,
    },

    /// Show specific context file
    Show {
        /// Context file name (e.g., github, jira, daily-ambition)
        name: String,

        /// Show raw content (no formatting)
        #[arg(short, long)]
        raw: bool,
    },

    /// List all context files
    List,

    /// Show context bundle statistics
    Stats,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Initialize configuration
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },

    /// Show configuration file path
    Path,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration
    let config = config::Config::load(cli.config.as_deref())?;

    // Print banner in verbose mode
    if cli.verbose {
        print_banner();
    }

    // Route to appropriate command handler
    match cli.command {
        Commands::Memory { action } => memory::handle(action, &config, cli.verbose).await,
        Commands::Skills { action } => skills::handle(action, &config, cli.verbose).await,
        Commands::Context { action } => context::handle(action, &config, cli.verbose).await,
        Commands::Reflect { session, export, user } => {
            reflect::handle(session, export, user, &config, cli.verbose).await
        }
        Commands::Chat { message, user, continue_session } => {
            chat::handle(message, user, continue_session, &config, cli.verbose).await
        }
        Commands::Health { deep } => health_check(deep, &config).await,
        Commands::Config { action } => handle_config(action, &config),
    }
}

fn print_banner() {
    println!("{}", "╔════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║  PAM - Proactive Agentic Manager                           ║".bright_cyan());
    println!("{}", "║  Chief of Staff CLI for AI Garage @ MERGE                  ║".bright_cyan());
    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();
}

async fn health_check(deep: bool, config: &config::Config) -> Result<()> {
    println!("{}", "PAM Health Check".bold());
    println!("{}", "─".repeat(40));

    // Basic health
    println!("{} API Endpoint: {}", "•".green(), config.api_url);

    if deep {
        println!("\n{}", "Deep Health Check".bold());

        // Check API
        print!("  Checking API... ");
        match api::client::health_check(&config.api_url).await {
            Ok(status) => println!("{} {}", "✓".green(), status),
            Err(e) => println!("{} {}", "✗".red(), e),
        }

        // Check Database
        print!("  Checking Database... ");
        match api::client::check_database(config).await {
            Ok(_) => println!("{}", "✓ Connected".green()),
            Err(e) => println!("{} {}", "✗".red(), e),
        }

        // Check GCS
        print!("  Checking GCS Context... ");
        match api::client::check_gcs(config).await {
            Ok(count) => println!("{} {} files available", "✓".green(), count),
            Err(e) => println!("{} {}", "✗".red(), e),
        }
    }

    Ok(())
}

fn handle_config(action: ConfigAction, config: &config::Config) -> Result<()> {
    match action {
        ConfigAction::Show => {
            println!("{}", "PAM Configuration".bold());
            println!("{}", "─".repeat(40));
            println!("API URL:     {}", config.api_url);
            println!("GCS Bucket:  {}", config.gcs_bucket);
            println!("User Email:  {}", config.user_email.as_deref().unwrap_or("(not set)"));
            println!("DB Host:     {}", config.db_host);
            Ok(())
        }
        ConfigAction::Set { key, value } => {
            println!("Setting {} = {}", key.bold(), value);
            config::Config::set_value(&key, &value)?;
            println!("{} Configuration updated", "✓".green());
            Ok(())
        }
        ConfigAction::Init { force } => {
            config::Config::init(force)?;
            println!("{} Configuration initialized", "✓".green());
            Ok(())
        }
        ConfigAction::Path => {
            println!("{}", config::Config::config_path()?.display());
            Ok(())
        }
    }
}
