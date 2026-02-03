#!/bin/bash
# PAM CLI Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/MERGE-AI-Garage/pam-cli/main/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  PAM CLI Installer                                         ║"
echo "║  Proactive Agentic Manager - Chief of Staff CLI            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        PLATFORM="linux"
        ;;
    Darwin*)
        PLATFORM="macos"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="windows"
        ;;
    *)
        echo -e "${RED}Error: Unsupported operating system: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    arm64|aarch64)
        ARCH="aarch64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}Detected: $PLATFORM-$ARCH${NC}"

# Get latest release
REPO="MERGE-AI-Garage/pam-cli"
RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"

echo -e "${YELLOW}Fetching latest release...${NC}"

# Check if GitHub CLI is available for authentication (for private repos)
if command -v gh &> /dev/null && gh auth status &> /dev/null; then
    LATEST_VERSION=$(gh api repos/$REPO/releases/latest --jq '.tag_name' 2>/dev/null || echo "")
else
    LATEST_VERSION=$(curl -s "$RELEASE_URL" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
fi

if [ -z "$LATEST_VERSION" ]; then
    echo -e "${YELLOW}No release found. Building from source...${NC}"

    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        echo -e "${YELLOW}Rust not found. Installing via rustup...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Clone and build
    TEMP_DIR=$(mktemp -d)
    echo -e "${YELLOW}Cloning repository...${NC}"
    git clone "https://github.com/$REPO.git" "$TEMP_DIR/pam-cli"
    cd "$TEMP_DIR/pam-cli"

    echo -e "${YELLOW}Building PAM CLI (this may take a few minutes)...${NC}"
    cargo build --release

    BINARY_PATH="$TEMP_DIR/pam-cli/target/release/pam"
else
    echo -e "${GREEN}Latest version: $LATEST_VERSION${NC}"

    # Construct download URL
    if [ "$PLATFORM" = "windows" ]; then
        BINARY_NAME="pam-$PLATFORM-$ARCH.exe"
    else
        BINARY_NAME="pam-$PLATFORM-$ARCH"
    fi

    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_VERSION/$BINARY_NAME"

    echo -e "${YELLOW}Downloading $BINARY_NAME...${NC}"

    TEMP_DIR=$(mktemp -d)
    BINARY_PATH="$TEMP_DIR/pam"

    if command -v wget &> /dev/null; then
        wget -q "$DOWNLOAD_URL" -O "$BINARY_PATH" || {
            echo -e "${RED}Failed to download binary. Falling back to source build...${NC}"
            BINARY_PATH=""
        }
    else
        curl -fsSL "$DOWNLOAD_URL" -o "$BINARY_PATH" || {
            echo -e "${RED}Failed to download binary. Falling back to source build...${NC}"
            BINARY_PATH=""
        }
    fi
fi

# Determine install location
INSTALL_DIR="$HOME/.local/bin"
if [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
fi

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Install binary
if [ -n "$BINARY_PATH" ] && [ -f "$BINARY_PATH" ]; then
    chmod +x "$BINARY_PATH"

    echo -e "${YELLOW}Installing to $INSTALL_DIR/pam...${NC}"

    if [ -w "$INSTALL_DIR" ]; then
        mv "$BINARY_PATH" "$INSTALL_DIR/pam"
    else
        sudo mv "$BINARY_PATH" "$INSTALL_DIR/pam"
    fi

    echo -e "${GREEN}✓ PAM CLI installed successfully!${NC}"
else
    echo -e "${RED}Error: Installation failed${NC}"
    exit 1
fi

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo -e "${YELLOW}Note: Add $INSTALL_DIR to your PATH:${NC}"
    echo ""
    echo "  # Add to ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
fi

# Verify installation
echo ""
if command -v pam &> /dev/null; then
    echo -e "${GREEN}PAM CLI is ready! Try:${NC}"
    echo ""
    echo "  pam --help              # Show all commands"
    echo "  pam config init         # Initialize configuration"
    echo "  pam health              # Check system health"
    echo "  pam chat \"Hello PAM\"    # Start chatting"
    echo ""
else
    echo -e "${YELLOW}Restart your terminal or run: source ~/.bashrc${NC}"
    echo -e "${YELLOW}Then try: pam --help${NC}"
fi

# Cleanup
rm -rf "$TEMP_DIR" 2>/dev/null || true

echo -e "${BLUE}Thank you for installing PAM CLI!${NC}"
