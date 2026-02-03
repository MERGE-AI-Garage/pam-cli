# PAM CLI

**Proactive Agentic Manager** - Command-line interface for PAM Chief of Staff.

Built in Rust following the Maestro CLI-first pattern: every capability testable from terminal.

## Installation

### Quick Install (Recommended)

```bash
# One-liner install (downloads pre-built binary or builds from source)
curl -fsSL https://raw.githubusercontent.com/MERGE-AI-Garage/pam-cli/main/install.sh | bash
```

### Using Cargo (Rust users)

```bash
# Install directly from GitHub
cargo install --git https://github.com/MERGE-AI-Garage/pam-cli.git

# Or clone and install locally
git clone https://github.com/MERGE-AI-Garage/pam-cli.git
cd pam-cli
cargo install --path .
```

### Download Pre-built Binary

Download the latest release for your platform from [GitHub Releases](https://github.com/MERGE-AI-Garage/pam-cli/releases):

| Platform | Architecture | Download |
|----------|--------------|----------|
| macOS | Apple Silicon (M1/M2/M3) | `pam-macos-aarch64` |
| macOS | Intel | `pam-macos-x86_64` |
| Linux | x86_64 | `pam-linux-x86_64` |
| Linux | ARM64 | `pam-linux-aarch64` |
| Windows | x86_64 | `pam-windows-x86_64.exe` |

Then make it executable and move to your PATH:

```bash
chmod +x pam-macos-aarch64
sudo mv pam-macos-aarch64 /usr/local/bin/pam
```

### Build from Source

```bash
# Requires Rust 1.70+
git clone https://github.com/MERGE-AI-Garage/pam-cli.git
cd pam-cli
cargo build --release
cp target/release/pam /usr/local/bin/
```

## Configuration

```bash
# Initialize config file
pam config init

# Set your user email
pam config set user_email sdulaney@mergeworld.com

# Show current config
pam config show
```

Or use environment variables:

```bash
export PAM_USER_EMAIL=sdulaney@mergeworld.com
export PAM_API_URL=https://pam-production-service-925072200586.us-central1.run.app
export PAM_DB_PASSWORD=your_password
```

## Commands

### Chat

```bash
# Single message
pam chat "What did the team accomplish yesterday?"

# Interactive mode
pam chat

# Continue previous session
pam chat --continue-session
```

### Skills

```bash
# List available skills
pam skills list

# Test a skill
pam skills test jira-query

# Invoke a skill with parameters
pam skills invoke github-commits --params '{"query": "Show commits from Sydney"}'

# View skill audit log
pam skills log --limit 10
```

### Memory

```bash
# Check memory status
pam memory status --deep

# Search memories
pam memory search "blockers" --limit 5

# List recent memories
pam memory list --user sdulaney@mergeworld.com
```

### Context

```bash
# Check context bundle status
pam context status --freshness

# Show specific context file
pam context show github

# Refresh context from GCS
pam context refresh

# View context statistics
pam context stats
```

### Reflection

```bash
# Generate reflection from today's sessions
pam reflect

# Reflect on specific session
pam reflect --session cos_20260129_143022_abc12345

# Export reflection to markdown
pam reflect --export
```

### Health Check

```bash
# Basic health check
pam health

# Deep health check (all services)
pam health --deep
```

## Interactive Chat Commands

When in interactive chat mode:

| Command | Description |
|---------|-------------|
| `quit`, `exit`, `q` | End the chat session |
| `clear` | Start a new session |
| `/reflect` | Generate reflection from current session |
| `/status` | Show current session info |
| `help` | Show help |

## Examples

### Morning Report

```bash
pam chat "Good morning PAM. Give me my morning report."
```

### Check Team Status

```bash
pam skills invoke daily-ambition --params '{"query": "What did the team commit to today?"}'
```

### Quick Context Check

```bash
pam context show jira
```

### Test All Skills

```bash
for skill in jira-query github-commits daily-ambition web-fetch pam-memory; do
  echo "Testing $skill..."
  pam skills test $skill
done
```

## Architecture

```
pam-cli/
├── src/
│   ├── main.rs          # CLI entry point (clap)
│   ├── config.rs        # Configuration management
│   ├── commands/
│   │   ├── memory.rs    # Memory subcommands
│   │   ├── skills.rs    # Skill management
│   │   ├── context.rs   # Context bundles
│   │   ├── reflect.rs   # Reflection loop
│   │   └── chat.rs      # Interactive chat
│   └── api/
│       └── client.rs    # HTTP client for PAM API
└── Cargo.toml
```

## Development

```bash
# Run in development
cargo run -- chat "Hello PAM"

# Run with verbose logging
cargo run -- -v skills list

# Build release
cargo build --release
```

## Related Projects

- [PAM Production Service](https://github.com/mergeworld/pam-meeting-agent) - Backend API
- [Lab Executive Command Center](https://github.com/mergeworld/lab-executive-command-center) - Web UI
- [Moltbot](https://docs.molt.bot) - Inspiration for CLI-first pattern

## License

MIT - AI Garage @ MERGE
