//! Jira ticket management commands

use anyhow::Result;
use colored::Colorize;
use std::process::Command;

use crate::config::Config;
use crate::JiraAction;

pub async fn handle(action: JiraAction, _config: &Config, verbose: bool) -> Result<()> {
    match action {
        JiraAction::Create { summary, description, ticket_type, priority, assignee } => {
            create(&summary, description, ticket_type, priority, assignee, verbose).await
        }
        JiraAction::List { project, status, assignee, limit } => {
            list(project, status, assignee, limit, verbose).await
        }
        JiraAction::Projects => {
            projects(verbose).await
        }
    }
}

async fn create(
    summary: &str,
    description: Option<String>,
    ticket_type: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    verbose: bool,
) -> Result<()> {
    println!("{}", "Creating Jira Ticket".bold());
    println!("{}", "─".repeat(40));
    println!("Summary: {}", summary.cyan());

    if let Some(ref desc) = description {
        println!("Description: {}", desc.dimmed());
    }

    // Build command to call Python script
    let script_path = std::env::var("PAM_MEETING_AGENT_PATH")
        .unwrap_or_else(|_| "/Users/sdulaney/Documents/pam-meeting-agent".to_string());

    let script = format!("{}/create_jira_ticket.py", script_path);

    let mut cmd = Command::new("python3");
    cmd.arg(&script)
        .arg("-s").arg(summary);

    if let Some(ref desc) = description {
        cmd.arg("-d").arg(desc);
    }

    if let Some(ref t) = ticket_type {
        cmd.arg("-t").arg(t);
    }

    if let Some(ref p) = priority {
        cmd.arg("-p").arg(p);
    }

    if let Some(ref a) = assignee {
        cmd.arg("-a").arg(a);
    }

    if verbose {
        println!("\nRunning: python3 {} -s \"{}\"", script, summary);
    }

    println!();

    let output = cmd.output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse the output to extract ticket key and URL
        for line in stdout.lines() {
            if line.contains("Created:") {
                println!("{} {}", "✓".green(), line);
            } else if line.contains("URL:") {
                println!("  {}", line.cyan());
            } else if !line.starts_with("Creating") && !line.starts_with("  Summary")
                && !line.starts_with("  Type") && !line.is_empty() {
                println!("{}", line);
            }
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{} Failed to create ticket", "✗".red());
        if !stderr.is_empty() {
            println!("{}", stderr);
        }
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
    }

    Ok(())
}

async fn list(
    project: Option<String>,
    status: Option<String>,
    assignee: Option<String>,
    limit: usize,
    verbose: bool,
) -> Result<()> {
    println!("{}", "Jira Tickets".bold());
    println!("{}", "─".repeat(40));

    let proj = project.as_deref().unwrap_or("AP");

    if verbose {
        println!("Project: {}", proj);
        if let Some(ref s) = status {
            println!("Status filter: {}", s);
        }
        if let Some(ref a) = assignee {
            println!("Assignee filter: {}", a);
        }
        println!("Limit: {}", limit);
        println!();
    }

    // Build JQL query
    let mut jql_parts = vec![format!("project = {}", proj)];

    if let Some(ref s) = status {
        jql_parts.push(format!("status = \"{}\"", s));
    } else {
        jql_parts.push("status != Done".to_string());
    }

    if let Some(ref a) = assignee {
        jql_parts.push(format!("assignee = \"{}\"", a));
    }

    let jql = jql_parts.join(" AND ");

    // Call Python to query Jira
    let script_path = std::env::var("PAM_MEETING_AGENT_PATH")
        .unwrap_or_else(|_| "/Users/sdulaney/Documents/pam-meeting-agent".to_string());

    let python_code = format!(r#"
import sys
sys.path.insert(0, '{}')
from src.test_jira_integration import get_jira_issues
import os

# Load env
env_path = '{}'
if os.path.exists(env_path + '/.env'):
    with open(env_path + '/.env') as f:
        for line in f:
            if '=' in line and not line.startswith('#'):
                key, value = line.strip().split('=', 1)
                os.environ[key] = value

result = get_jira_issues(
    '{}',  # JQL becomes the first param - we'll use project directly
    os.getenv('JIRA_DOMAIN', 'mergeworld.atlassian.net'),
    os.getenv('JIRA_EMAIL'),
    os.getenv('JIRA_API_TOKEN')
)

if result['success']:
    issues = result['issues'][:{}]
    for issue in issues:
        print(f"{{issue['key']}}: {{issue['summary']}}")
        print(f"  Status: {{issue['status']}} | Priority: {{issue['priority']}}")
else:
    print(f"Error: {{result.get('error', 'Unknown error')}}")
"#, script_path, script_path, proj, limit);

    // Actually, let's use a simpler approach - just call a dedicated list script
    // For now, show a helpful message
    println!("{}", format!("Querying {} project...", proj).dimmed());
    println!();

    // Use the test_jira_integration.py directly with subprocess
    let output = Command::new("python3")
        .arg("-c")
        .arg(&python_code)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            println!("{}", "No tickets found matching criteria.".yellow());
        } else {
            for line in stdout.lines() {
                if line.starts_with("  ") {
                    println!("{}", line.dimmed());
                } else if line.starts_with("Error:") {
                    println!("{} {}", "✗".red(), line);
                } else {
                    println!("{} {}", "•".green(), line);
                }
            }
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{} Failed to list tickets: {}", "✗".red(), stderr);
    }

    Ok(())
}

async fn projects(verbose: bool) -> Result<()> {
    println!("{}", "Jira Projects".bold());
    println!("{}", "─".repeat(40));

    // Hardcoded for now - these are the known projects
    let projects = vec![
        ("AP", "PAM - Proactive Agentic Manager"),
        ("AIG", "AI Garage"),
        ("SK", "Sage Knowledge Base"),
    ];

    for (key, name) in &projects {
        println!("{} {} - {}", "•".green(), key.bold(), name);
    }

    if verbose {
        println!();
        println!("{}", "Use 'pam jira list -p <PROJECT>' to see tickets".dimmed());
    }

    Ok(())
}
