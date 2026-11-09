#!/usr/bin/env sh
# Automated uninstaller for TIDE (Terminal Integrated Development Environment) on macOS and Linux.
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/kurarensu16/TIDE/main/scripts/uninstall.sh | sh

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

echo ""
echo "${CYAN}=================================================${NC}"
echo "   TIDE - Uninstaller                            "
echo "${CYAN}=================================================${NC}"
echo ""

INSTALL_DIR="${TIDE_INSTALL_DIR:-$HOME/.local/bin}"
BIN_PATH="$INSTALL_DIR/tide"
CONFIG_DIR="$HOME/.config/tide"

# 1. Remove binary
if [ -f "$BIN_PATH" ]; then
    echo "${YELLOW}==> Removing binary at $BIN_PATH...${NC}"
    rm -f "$BIN_PATH"
    echo "${GREEN}==> Removed $BIN_PATH.${NC}"
else
    echo "${GRAY}==> No binary found at $BIN_PATH.${NC}"
fi

# Also check /usr/local/bin/tide if installed with sudo
if [ -f "/usr/local/bin/tide" ]; then
    echo "${YELLOW}==> Found /usr/local/bin/tide. Removing...${NC}"
    rm -f "/usr/local/bin/tide" 2>/dev/null || sudo rm -f "/usr/local/bin/tide" || true
fi

# 2. Remove configuration
if [ -d "$CONFIG_DIR" ]; then
    echo "${YELLOW}==> Removing global configuration at $CONFIG_DIR...${NC}"
    rm -rf "$CONFIG_DIR"
    echo "${GREEN}==> Removed configuration directory.${NC}"
fi

echo ""
echo "${GREEN}  SUCCESS: TIDE has been uninstalled.${NC}"
echo ""

