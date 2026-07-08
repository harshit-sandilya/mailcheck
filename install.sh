#!/bin/sh
set -e

REPO="harshit-sandilya/mailcheck"

# Detect OS + arch
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)  OS="linux" ;;
  darwin) OS="macos" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64)  ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

# OS-specific install dir
# macOS: ~/.local/bin — user-owned, no Gatekeeper quarantine issues
# Linux: /usr/local/bin — standard system bin
if [ "$OS" = "macos" ]; then
  DEFAULT_BIN_DIR="$HOME/.local/bin"
else
  DEFAULT_BIN_DIR="/usr/local/bin"
fi
BIN_DIR="${MAILCHECK_BIN_DIR:-$DEFAULT_BIN_DIR}"

# Get latest release tag
VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' | cut -d'"' -f4)

ARTIFACT="mailcheck-${OS}-${ARCH}"
URL="https://github.com/$REPO/releases/download/$VERSION/$ARTIFACT"

echo "Installing mailcheck $VERSION ($OS/$ARCH)..."
mkdir -p "$BIN_DIR"
curl -fsSL "$URL" -o /tmp/mailcheck
chmod +x /tmp/mailcheck

if [ -w "$BIN_DIR" ]; then
  mv /tmp/mailcheck "$BIN_DIR/mailcheck"
else
  sudo mv /tmp/mailcheck "$BIN_DIR/mailcheck"
fi

echo ""
echo "mailcheck $VERSION installed to $BIN_DIR/mailcheck"
echo ""

# Check if BIN_DIR already on PATH
case ":$PATH:" in
  *":$BIN_DIR:"*)
    echo "mailcheck is ready. Run: mailcheck --help"
    ;;
  *)
    echo "$BIN_DIR is not in your PATH."
    echo ""
    echo "Run this command to add it:"
    echo ""
    echo "  For zsh:  echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
    echo "  For bash: echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
    echo ""
    echo "Or for this session only:"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
    ;;
esac
