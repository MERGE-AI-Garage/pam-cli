//! HTTP API client for PAM services

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::Config;

lazy_static::lazy_static! {
    static ref HTTP_CLIENT: Client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client");
}

// =============================================================================
// DATA STRUCTURES
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct MemoryStatus {
    pub total_memories: i64,
    pub total_sessions: i64,
    pub total_reflections: i64,
    pub tables: Vec<TableInfo>,
}

#[derive(Debug, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub row_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct MemorySearchResult {
    pub title: String,
    pub session_id: String,
    pub content: String,
    pub created_at: String,
    pub relevance_score: f64,
}

#[derive(Debug, Deserialize)]
pub struct MemoryEntry {
    pub session_id: String,
    pub preview: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Skill {
    pub skill_key: String,
    pub description: String,
    pub risk_level: String,
    pub enabled: bool,
    pub usage_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct SkillLogEntry {
    pub skill_key: String,
    pub user_email: String,
    pub success: bool,
    pub duration_ms: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ContextStatus {
    pub file_count: i32,
    pub total_size_kb: f64,
    pub estimated_tokens: i64,
    pub files: Vec<ContextFile>,
}

#[derive(Debug, Deserialize)]
pub struct ContextFile {
    pub name: String,
    pub size_kb: f64,
    pub age_minutes: f64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshResult {
    pub files_loaded: i32,
    pub total_size_kb: f64,
}

#[derive(Debug, Deserialize)]
pub struct ContextStats {
    pub total_size_kb: f64,
    pub estimated_tokens: i64,
    pub realtime_kb: f64,
    pub realtime_pct: f64,
    pub projects_kb: f64,
    pub projects_pct: f64,
    pub team_kb: f64,
    pub team_pct: f64,
    pub activity_kb: f64,
    pub activity_pct: f64,
    pub team_members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reflection {
    pub what_worked: Vec<String>,
    pub what_failed: Vec<String>,
    pub learnings: Vec<String>,
    pub action_items: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    message: String,
    user: String,
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    response: String,
    session_id: String,
}

// =============================================================================
// HEALTH CHECKS
// =============================================================================

pub async fn health_check(api_url: &str) -> Result<String> {
    let url = format!("{}/api/health", api_url);
    let resp = HTTP_CLIENT.get(&url).send().await?;

    if resp.status().is_success() {
        Ok("Healthy".to_string())
    } else {
        Ok(format!("Unhealthy ({})", resp.status()))
    }
}

pub async fn check_database(config: &Config) -> Result<()> {
    // This would connect to the database directly
    // For now, we'll use the API health endpoint
    let url = format!("{}/api/health/detailed", config.api_url);
    let resp = HTTP_CLIENT.get(&url).send().await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        anyhow::bail!("Database health check failed")
    }
}

pub async fn check_gcs(config: &Config) -> Result<i32> {
    let url = format!("{}/api/chief-of-staff/context-debug", config.api_url);
    let resp = HTTP_CLIENT.get(&url).send().await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        let count = data["file_count"].as_i64().unwrap_or(0) as i32;
        Ok(count)
    } else {
        anyhow::bail!("GCS health check failed")
    }
}

// =============================================================================
// MEMORY OPERATIONS
// =============================================================================

pub async fn get_memory_status(api_url: &str) -> Result<MemoryStatus> {
    let url = format!("{}/api/chief-of-staff/memory/status", api_url);
    let resp = HTTP_CLIENT.get(&url).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        anyhow::bail!("Failed to get memory status: {}", resp.status())
    }
}

pub async fn search_memories(
    api_url: &str,
    query: &str,
    limit: usize,
    user: Option<&str>,
) -> Result<Vec<MemorySearchResult>> {
    let url = format!("{}/api/chief-of-staff/memory/search", api_url);

    let mut params = vec![
        ("query", query.to_string()),
        ("limit", limit.to_string()),
    ];
    if let Some(u) = user {
        params.push(("user", u.to_string()));
    }

    let resp = HTTP_CLIENT.get(&url).query(&params).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        anyhow::bail!("Memory search failed: {}", resp.status())
    }
}

pub async fn index_memory(api_url: &str, content: &str, tags: &[String]) -> Result<String> {
    let url = format!("{}/api/chief-of-staff/memory/index", api_url);

    let body = serde_json::json!({
        "content": content,
        "tags": tags,
    });

    let resp = HTTP_CLIENT.post(&url).json(&body).send().await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        Ok(data["id"].as_str().unwrap_or("unknown").to_string())
    } else {
        anyhow::bail!("Memory indexing failed: {}", resp.status())
    }
}

pub async fn list_memories(
    api_url: &str,
    limit: usize,
    user: Option<&str>,
) -> Result<Vec<MemoryEntry>> {
    let url = format!("{}/api/chief-of-staff/memory/list", api_url);

    let mut params = vec![("limit", limit.to_string())];
    if let Some(u) = user {
        params.push(("user", u.to_string()));
    }

    let resp = HTTP_CLIENT.get(&url).query(&params).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        anyhow::bail!("Failed to list memories: {}", resp.status())
    }
}

pub async fn clear_memories(api_url: &str, user: &str) -> Result<i64> {
    let url = format!("{}/api/chief-of-staff/memory/clear", api_url);

    let body = serde_json::json!({ "user": user });
    let resp = HTTP_CLIENT.post(&url).json(&body).send().await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        Ok(data["deleted_count"].as_i64().unwrap_or(0))
    } else {
        anyhow::bail!("Failed to clear memories: {}", resp.status())
    }
}

// =============================================================================
// SKILLS OPERATIONS
// =============================================================================

pub async fn list_skills(api_url: &str) -> Result<Vec<Skill>> {
    let url = format!("{}/api/chief-of-staff/skills", api_url);
    let resp = HTTP_CLIENT.get(&url).send().await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(data["skills"].clone())?)
    } else {
        anyhow::bail!("Failed to list skills: {}", resp.status())
    }
}

pub async fn invoke_skill(
    api_url: &str,
    skill: &str,
    params: &str,
    user: Option<&str>,
) -> Result<serde_json::Value> {
    let url = format!("{}/api/chief-of-staff/skill", api_url);

    let params_json: serde_json::Value = serde_json::from_str(params)
        .context("Invalid JSON params")?;

    let body = serde_json::json!({
        "skill_key": skill,
        "params": params_json,
        "user_email": user.unwrap_or("cli@mergeworld.com"),
        "session_id": format!("cli_{}", chrono::Utc::now().timestamp()),
    });

    let resp = HTTP_CLIENT.post(&url).json(&body).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        let error_text = resp.text().await?;
        anyhow::bail!("Skill invocation failed: {}", error_text)
    }
}

pub async fn get_skill_log(
    api_url: &str,
    skill: Option<&str>,
    limit: usize,
) -> Result<Vec<SkillLogEntry>> {
    let url = format!("{}/api/chief-of-staff/skill-log", api_url);

    let mut params = vec![("limit", limit.to_string())];
    if let Some(s) = skill {
        params.push(("skill", s.to_string()));
    }

    let resp = HTTP_CLIENT.get(&url).query(&params).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        anyhow::bail!("Failed to get skill log: {}", resp.status())
    }
}

// =============================================================================
// CONTEXT OPERATIONS
// =============================================================================

pub async fn get_context_status(api_url: &str) -> Result<ContextStatus> {
    let url = format!("{}/api/chief-of-staff/context-debug", api_url);
    let resp = HTTP_CLIENT.get(&url).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        anyhow::bail!("Failed to get context status: {}", resp.status())
    }
}

pub async fn refresh_context(api_url: &str, _force: bool) -> Result<RefreshResult> {
    let url = format!("{}/api/chief-of-staff/context-refresh", api_url);
    let resp = HTTP_CLIENT.post(&url).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        anyhow::bail!("Failed to refresh context: {}", resp.status())
    }
}

pub async fn get_context_file(api_url: &str, filename: &str) -> Result<String> {
    let url = format!("{}/api/chief-of-staff/context/{}", api_url, filename);
    let resp = HTTP_CLIENT.get(&url).send().await?;

    if resp.status().is_success() {
        Ok(resp.text().await?)
    } else {
        anyhow::bail!("Failed to get context file: {}", resp.status())
    }
}

pub async fn list_context_files(api_url: &str) -> Result<Vec<ContextFile>> {
    let status = get_context_status(api_url).await?;
    Ok(status.files)
}

pub async fn get_context_stats(api_url: &str) -> Result<ContextStats> {
    let url = format!("{}/api/chief-of-staff/context-stats", api_url);
    let resp = HTTP_CLIENT.get(&url).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        anyhow::bail!("Failed to get context stats: {}", resp.status())
    }
}

// =============================================================================
// CHAT OPERATIONS
// =============================================================================

pub async fn chat(
    api_url: &str,
    user_email: &str,
    session_id: &str,
    message: &str,
) -> Result<String> {
    let url = format!("{}/api/chief-of-staff/chat", api_url);

    let body = ChatRequest {
        message: message.to_string(),
        user: user_email.to_string(),
        session_id: session_id.to_string(),
    };

    // Get CLI API key from environment
    let cli_api_key = std::env::var("PAM_CLI_API_KEY").unwrap_or_default();

    let resp = HTTP_CLIENT.post(&url)
        .header("X-User-Email", user_email)
        .header("X-PAM-CLI-Key", &cli_api_key)
        .json(&body)
        .send()
        .await?;

    if resp.status().is_success() {
        let data: ChatResponse = resp.json().await?;
        Ok(data.response)
    } else {
        let error = resp.text().await?;
        anyhow::bail!("Chat failed: {}", error)
    }
}

pub async fn get_latest_session(api_url: &str, user_email: &str) -> Result<Option<String>> {
    let url = format!("{}/api/chief-of-staff/sessions/latest", api_url);

    let resp = HTTP_CLIENT.get(&url)
        .query(&[("user", user_email)])
        .send()
        .await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        Ok(data["session_id"].as_str().map(|s| s.to_string()))
    } else {
        Ok(None)
    }
}

// =============================================================================
// REFLECTION OPERATIONS
// =============================================================================

pub async fn get_today_sessions(api_url: &str, user_email: &str) -> Result<Vec<String>> {
    let url = format!("{}/api/chief-of-staff/sessions/today", api_url);

    let resp = HTTP_CLIENT.get(&url)
        .query(&[("user", user_email)])
        .send()
        .await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        let sessions: Vec<String> = data["sessions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(sessions)
    } else {
        anyhow::bail!("Failed to get today's sessions: {}", resp.status())
    }
}

pub async fn generate_reflection(
    api_url: &str,
    user_email: &str,
    sessions: &[String],
) -> Result<Reflection> {
    let url = format!("{}/api/chief-of-staff/reflect", api_url);

    let body = serde_json::json!({
        "user_email": user_email,
        "sessions": sessions,
    });

    let resp = HTTP_CLIENT.post(&url).json(&body).send().await?;

    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        anyhow::bail!("Failed to generate reflection: {}", resp.status())
    }
}

pub async fn save_reflection(
    api_url: &str,
    user_email: &str,
    reflection: &Reflection,
) -> Result<String> {
    let url = format!("{}/api/chief-of-staff/reflection/save", api_url);

    let body = serde_json::json!({
        "user_email": user_email,
        "reflection": reflection,
    });

    let resp = HTTP_CLIENT.post(&url).json(&body).send().await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        Ok(data["id"].as_str().unwrap_or("unknown").to_string())
    } else {
        anyhow::bail!("Failed to save reflection: {}", resp.status())
    }
}
