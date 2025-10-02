#!/bin/bash
set -euo pipefail

# Universal installer for failover
# Automatically detects platform and downloads the correct binary

REPO="kagehq/failover"
LATEST_URL="https://github.com/$REPO/releases/latest/download"

# Detect platform
detect_platform() {
    local os arch
    
    # Detect OS
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          echo "❌ Unsupported OS: $(uname -s)"; exit 1 ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        armv7l)         arch="armv7" ;;
        *)              echo "❌ Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac
    
    echo "${os}-${arch}"
}

# Download and install
install_failover() {
    local platform="$1"
    local filename="failover"
    
    # Windows uses .exe extension
    if [[ "$platform" == windows-* ]]; then
        filename="failover.exe"
    fi
    
    echo "🔍 Detected platform: $platform"
    echo "📦 Downloading failover-$platform..."
    
    # Download the binary
    if ! curl -L "$LATEST_URL/failover-$platform" -o "$filename"; then
        echo "❌ Failed to download failover-$platform"
        echo "💡 Available platforms:"
        echo "   - linux-x86_64, linux-aarch64"
        echo "   - macos-x86_64, macos-aarch64" 
        echo "   - windows-x86_64"
        exit 1
    fi
    
    # Make executable (Unix only)
    if [[ "$platform" != windows-* ]]; then
        chmod +x "$filename"
    fi
    
    echo "✅ Installed failover-$platform as $filename"
    echo ""
    echo "🚀 Usage:"
    if [[ "$platform" == windows-* ]]; then
        echo "   $filename --primary=https://myapp.com --backup=https://myapp-backup.s3.amazonaws.com"
    else
        echo "   ./$filename --primary=https://myapp.com --backup=https://myapp-backup.s3.amazonaws.com"
    fi
}

# Main
main() {
    echo "🎯 failover installer"
    echo "   Repository: https://github.com/$REPO"
    echo ""
    
    local platform
    platform=$(detect_platform)
    
    install_failover "$platform"
}

# Run if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
