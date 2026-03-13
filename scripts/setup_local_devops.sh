#!/bin/bash
# MEMORY_P Local DevOps Setup: Install act + Run GitHub Actions Locally
# Enables local workflow testing before GitHub push
# Installs act (GitHub Actions runner) and validates Docker

set -e

echo "🚀 Setting up local DevOps environment for MEMORY_P..."

# Optional Docker check (skip if not available)
if command -v docker &> /dev/null; then
    if docker info > /dev/null 2>&1; then
        echo "✅ Docker is running"
        DOCKER_AVAILABLE=true
    else
        echo "⚠️  Docker detected but not running - will use containerless mode"
        DOCKER_AVAILABLE=false
    fi
else
    echo "⚠️  Docker not found - will use containerless mode"
    DOCKER_AVAILABLE=false
fi

# Install act if not present
if ! command -v act &> /dev/null; then
    echo "📦 Installing act (GitHub Actions runner)..."
    # Download and install act
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        # Windows - use PowerShell to download
        echo "Downloading act for Windows..."
        powershell -Command "Invoke-WebRequest -Uri 'https://github.com/nektos/act/releases/latest/download/act_Windows_x86_64.zip' -OutFile 'act.zip'"
        powershell -Command "Expand-Archive -Path 'act.zip' -DestinationPath '.' -Force"
        # Move to a directory in PATH
        if [[ -d "/usr/local/bin" ]]; then
            mv act.exe /usr/local/bin/act.exe
        elif [[ -d "/c/Windows/System32" ]]; then
            mv act.exe /c/Windows/System32/act.exe
        else
            mv act.exe ./act.exe
            export PATH="$PATH:$(pwd)"
        fi
        # Move to a directory in PATH
        if [[ -d "/usr/local/bin" ]]; then
            mv act.exe /usr/local/bin/act.exe
        elif [[ -d "/c/Windows/System32" ]]; then
            mv act.exe /c/Windows/System32/act.exe
        else
            mv act.exe ./act.exe
            export PATH="$PATH:$(pwd)"
        fi
        # Move to a directory in PATH
        if [[ -d "/usr/local/bin" ]]; then
            mv act.exe /usr/local/bin/act.exe
        elif [[ -d "/c/Windows/System32" ]]; then
            mv act.exe /c/Windows/System32/act.exe
        else
            mv act.exe ./act.exe
            export PATH="$PATH:$(pwd)"
        fi
        # Move to a directory in PATH
        if [[ -d "/usr/local/bin" ]]; then
            mv act.exe /usr/local/bin/act.exe
        elif [[ -d "/c/Windows/System32" ]]; then
            mv act.exe /c/Windows/System32/act.exe
        else
            mv act.exe ./act.exe
            export PATH="$PATH:$(pwd)"
        fi
        # Move to a directory in PATH
        if [[ -d "/usr/local/bin" ]]; then
            mv act.exe /usr/local/bin/act.exe
        elif [[ -d "/c/Windows/System32" ]]; then
            mv act.exe /c/Windows/System32/act.exe
        else
            mv act.exe ./act.exe
            export PATH="$PATH:$(pwd)"
        fi
        # Move to a directory in PATH
        if [[ -d "/usr/local/bin" ]]; then
            mv act.exe /usr/local/bin/act.exe
        elif [[ -d "/c/Windows/System32" ]]; then
            mv act.exe /c/Windows/System32/act.exe
        else
            mv act.exe ./act.exe
            export PATH="$PATH:$(pwd)"
        fi
        # Move to a directory in PATH
        if [[ -d "/usr/local/bin" ]]; then
            mv act.exe /usr/local/bin/act.exe
        elif [[ -d "/c/Windows/System32" ]]; then
            mv act.exe /c/Windows/System32/act.exe
        else
            mv act.exe ./act.exe
            export PATH="$PATH:$(pwd)"
        fi
        # Move to a directory in PATH
        if [[ -d "/usr/local/bin" ]]; then
            mv act.exe /usr/local/bin/act.exe
        elif [[ -d "/c/Windows/System32" ]]; then
            mv act.exe /c/Windows/System32/act.exe
        else
            mv act.exe ./act.exe
            export PATH="$PATH:$(pwd)"
        fi
        rm act.zip
    else
        # Linux/Mac
        curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | bash
    fi
    echo "✅ act installed"
else
    echo "✅ act already installed"
fi

# Validate act version
act --version

# Check if we're in the correct directory
if [[ ! -f ".github/workflows/multi-lang-ci.yml" ]]; then
    echo "❌ Not in MEMORY_P repository root. Please cd to the repository."
    exit 1
fi

echo "✅ Local DevOps environment ready!"
echo ""
echo "Usage:"
if [[ "$DOCKER_AVAILABLE" == "true" ]]; then
    echo "  act                    # Run all workflows (with containers)"
    echo "  act -j rust-core       # Run specific job"
    echo "  act --list             # List available jobs"
    echo "  act --dry-run          # Dry run without execution"
else
    echo "  act --container=false  # Run workflows without containers"
    echo "  act -j rust-core --container=false  # Run specific job without containers"
    echo "  act --list             # List available jobs"
    echo "  act --dry-run          # Dry run without execution"
    echo ""
    echo "⚠️  Note: Some jobs require Docker containers and will be skipped"
fi