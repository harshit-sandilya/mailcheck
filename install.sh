#!/bin/sh
set -e

REPO="harshit-sandilya/mailcheck"
TMP_BIN=$(mktemp "${TMPDIR:-/tmp}/mailcheck.XXXXXX")
TMP_CHECKSUMS=$(mktemp "${TMPDIR:-/tmp}/mailcheck-checksums.XXXXXX")
trap 'rm -f "$TMP_BIN" "$TMP_CHECKSUMS"' EXIT HUP INT TERM

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
if [ -z "$VERSION" ]; then
  echo "Unable to determine the latest mailcheck release." >&2
  exit 1
fi

ARTIFACT="mailcheck-${OS}-${ARCH}"
URL="https://github.com/$REPO/releases/download/$VERSION/$ARTIFACT"
CHECKSUMS_URL="https://github.com/$REPO/releases/download/$VERSION/checksums.txt"

echo "Installing mailcheck $VERSION ($OS/$ARCH)..."
mkdir -p "$BIN_DIR"
curl -fsSL "$URL" -o "$TMP_BIN"
curl -fsSL "$CHECKSUMS_URL" -o "$TMP_CHECKSUMS"

EXPECTED=$(awk -v artifact="$ARTIFACT" '$2 == artifact { print $1 }' "$TMP_CHECKSUMS")
if [ -z "$EXPECTED" ]; then
  echo "Release checksum missing for $ARTIFACT." >&2
  exit 1
fi

if command -v shasum >/dev/null 2>&1; then
  ACTUAL=$(shasum -a 256 "$TMP_BIN" | awk '{ print $1 }')
elif command -v sha256sum >/dev/null 2>&1; then
  ACTUAL=$(sha256sum "$TMP_BIN" | awk '{ print $1 }')
else
  echo "A SHA-256 utility (shasum or sha256sum) is required." >&2
  exit 1
fi

if [ "$ACTUAL" != "$EXPECTED" ]; then
  echo "Checksum verification failed for $ARTIFACT." >&2
  exit 1
fi

chmod +x "$TMP_BIN"

if [ -w "$BIN_DIR" ]; then
  mv "$TMP_BIN" "$BIN_DIR/mailcheck"
else
  sudo mv "$TMP_BIN" "$BIN_DIR/mailcheck"
fi

"$BIN_DIR/mailcheck" --version

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
