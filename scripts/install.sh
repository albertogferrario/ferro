#!/bin/sh
# Ferro Framework Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/albertogferrario/ferro/main/scripts/install.sh | sh
# Or with project creation: curl -fsSL ... | sh -s -- my-app

set -e

# Configuration
REPO="albertogferrario/ferro"
BINARY_NAME="ferro"
INSTALL_DIR="${FERRO_INSTALL_DIR:-$HOME/.ferro/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info() {
    printf "${CYAN}info${NC}: %s\n" "$1"
}

success() {
    printf "${GREEN}success${NC}: %s\n" "$1"
}

warn() {
    printf "${YELLOW}warn${NC}: %s\n" "$1"
}

error() {
    printf "${RED}error${NC}: %s\n" "$1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)
            OS="linux"
            ;;
        Darwin)
            OS="darwin"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            OS="windows"
            ;;
        *)
            error "Unsupported operating system: $OS"
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
            error "Unsupported architecture: $ARCH"
            ;;
    esac

    PLATFORM="${OS}-${ARCH}"
}

# Get latest release version from GitHub
get_latest_version() {
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        error "Failed to get latest version"
    fi
}

# Download and install the binary
install_ferro() {
    info "Detected platform: $PLATFORM"
    info "Latest version: $VERSION"

    # Construct download URL
    if [ "$OS" = "windows" ]; then
        ARCHIVE_NAME="ferro-${VERSION}-${PLATFORM}.zip"
    else
        ARCHIVE_NAME="ferro-${VERSION}-${PLATFORM}.tar.gz"
    fi
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"

    info "Downloading from: $DOWNLOAD_URL"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download to temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    cd "$TMP_DIR"

    # Download archive
    if command -v curl > /dev/null; then
        curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_NAME"
    elif command -v wget > /dev/null; then
        wget -q "$DOWNLOAD_URL" -O "$ARCHIVE_NAME"
    else
        error "curl or wget is required"
    fi

    # Extract archive
    if [ "$OS" = "windows" ]; then
        unzip -q "$ARCHIVE_NAME"
    else
        tar -xzf "$ARCHIVE_NAME"
    fi

    # Install binary
    if [ "$OS" = "windows" ]; then
        mv "${BINARY_NAME}.exe" "$INSTALL_DIR/"
    else
        mv "$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi

    success "Ferro installed to $INSTALL_DIR/$BINARY_NAME"
}

# Add to PATH instructions
setup_path() {
    SHELL_NAME=$(basename "$SHELL")

    case "$SHELL_NAME" in
        bash)
            PROFILE="$HOME/.bashrc"
            ;;
        zsh)
            PROFILE="$HOME/.zshrc"
            ;;
        fish)
            PROFILE="$HOME/.config/fish/config.fish"
            ;;
        *)
            PROFILE="$HOME/.profile"
            ;;
    esac

    # Check if already in PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        return
    fi

    echo ""
    warn "Add Ferro to your PATH by adding this to $PROFILE:"
    echo ""
    if [ "$SHELL_NAME" = "fish" ]; then
        printf "  ${CYAN}set -gx PATH \$PATH %s${NC}\n" "$INSTALL_DIR"
    else
        printf "  ${CYAN}export PATH=\"\$PATH:%s\"${NC}\n" "$INSTALL_DIR"
    fi
    echo ""
    info "Then restart your shell or run: source $PROFILE"
}

# Create a new project if name provided
create_project() {
    PROJECT_NAME="$1"

    if [ -n "$PROJECT_NAME" ]; then
        echo ""
        info "Creating new project: $PROJECT_NAME"

        # Use the installed binary
        "$INSTALL_DIR/$BINARY_NAME" new "$PROJECT_NAME" --no-interaction

        echo ""
        success "Project created successfully!"
        echo ""
        echo "Next steps:"
        printf "  ${CYAN}cd %s${NC}\n" "$PROJECT_NAME"
        printf "  ${CYAN}cd frontend && npm install && cd ..${NC}\n"
        printf "  ${CYAN}ferro db:migrate${NC}\n"
        printf "  ${CYAN}ferro serve${NC}\n"
    fi
}

# Main
main() {
    echo ""
    printf "${CYAN}Ferro Framework Installer${NC}\n"
    echo ""

    detect_platform
    get_latest_version
    install_ferro
    setup_path

    # If a project name was passed as argument, create the project
    create_project "$1"

    echo ""
    success "Installation complete!"
}

main "$@"
