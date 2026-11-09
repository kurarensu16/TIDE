#!/usr/bin/env sh
# Automated installer for TIDE (Terminal Integrated Development Environment) on macOS and Linux.
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/kurarensu16/TIDE/main/scripts/install.sh | sh

set -e

REPO="kurarensu16/TIDE"
INSTALL_DIR="${TIDE_INSTALL_DIR:-$HOME/.local/bin}"

# ANSI Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

echo ""
echo "${CYAN}=================================================${NC}"
echo "   TIDE - Terminal Integrated Development Env    "
echo "   Parallel Coding-Agent Orchestration Platform  "
echo "${CYAN}=================================================${NC}"
echo ""

# 1. Detect OS
OS_TYPE="$(uname -s)"
case "$OS_TYPE" in
    Linux*)     OS="linux" ;;
    Darwin*)    OS="macos" ;;
    *)
        echo "${RED}Unsupported Operating System: $OS_TYPE${NC}"
        echo "TIDE currently supports Linux, macOS, and Windows."
        exit 1
        ;;
esac

# 2. Detect Architecture
ARCH_TYPE="$(uname -m)"
case "$ARCH_TYPE" in
    x86_64|amd64)   ARCH="x86_64" ;;
    arm64|aarch64)  ARCH="arm64" ;;
    *)
        echo "${RED}Unsupported Architecture: $ARCH_TYPE${NC}"
        echo "TIDE provides release binaries for x86_64 and arm64."
        exit 1
        ;;
esac

# Linux currently provides x86_64 release builds
if [ "$OS" = "linux" ] && [ "$ARCH" != "x86_64" ]; then
    echo "${YELLOW}Notice: Pre-built Linux binaries are currently optimized for x86_64.${NC}"
    echo "If on ARM Linux, you can compile from source using: cargo install --git https://github.com/$REPO.git"
    exit 1
fi

# 3. Determine Latest Version
echo "${YELLOW}==> Fetching latest release info from GitHub...${NC}"
TAG="${TIDE_VERSION:-}"
if [ -z "$TAG" ]; then
    if command -v curl >/dev/null 2>&1; then
        LATEST_JSON="$(curl -sSL "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null || true)"
    elif command -v wget >/dev/null 2>&1; then
        LATEST_JSON="$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null || true)"
    fi

    TAG="$(echo "$LATEST_JSON" | grep '"tag_name":' | head -n1 | sed -E 's/.*"tag_name":[[:space:]]*"([^"]+)".*/\1/')"
    if [ -z "$TAG" ]; then
        TAG="v0.1.0"
        echo "${YELLOW}==> Could not detect release tag automatically; defaulting to $TAG${NC}"
    fi
fi

echo "${GREEN}==> Installing TIDE $TAG ($OS-$ARCH)...${NC}"

# 4. Form Archive Name and Download URL
ARCHIVE_NAME="tide-${TAG}-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/${TAG}/${ARCHIVE_NAME}"

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'tide-install')"
ARCHIVE_PATH="$TMP_DIR/$ARCHIVE_NAME"

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

echo "${YELLOW}==> Downloading $ARCHIVE_NAME...${NC}"
if command -v curl >/dev/null 2>&1; then
    curl -fSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$ARCHIVE_PATH" "$DOWNLOAD_URL"
else
    echo "${RED}Error: curl or wget is required to download TIDE.${NC}"
    exit 1
fi

# 5. Extract and Install
echo "${YELLOW}==> Extracting binary...${NC}"
tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"

mkdir -p "$INSTALL_DIR"
if [ -f "$TMP_DIR/tide" ]; then
    mv "$TMP_DIR/tide" "$INSTALL_DIR/tide"
else
    # Find binary if inside a subfolder
    FOUND_BIN="$(find "$TMP_DIR" -type f -name tide | head -n1)"
    if [ -n "$FOUND_BIN" ]; then
        mv "$FOUND_BIN" "$INSTALL_DIR/tide"
    else
        echo "${RED}Error: Could not locate 'tide' executable in release archive.${NC}"
        exit 1
    fi
fi

chmod +x "$INSTALL_DIR/tide"
echo "${GREEN}==> Installed binary to: $INSTALL_DIR/tide${NC}"

# 6. Verify PATH
echo ""
echo "${GREEN}  SUCCESS! TIDE $TAG is ready to use.${NC}"
echo ""

case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        echo "  Launch your workspace right now:"
        echo "    ${CYAN}tide .${NC}"
        ;;
    *)
        echo "${YELLOW}  Note: $INSTALL_DIR is not currently in your \$PATH.${NC}"
        echo "  Add it to your profile (~/.bashrc or ~/.zshrc):"
        echo "    ${CYAN}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
        echo ""
        echo "  To launch immediately using full path:"
        echo "    ${CYAN}$INSTALL_DIR/tide .${NC}"
        ;;
esac

echo ""
echo "${GRAY}  Documentation & Guides: https://kurarensu16.github.io/TIDE/${NC}"
echo ""

