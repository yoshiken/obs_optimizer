#!/bin/bash
set -e

# OBS Optimizer Development Environment Entrypoint

echo "========================================"
echo "  OBS Optimizer Dev Environment"
echo "  Tauri + Rust + React + Claude Code"
echo "========================================"

# Display environment info
echo ""
echo "[Environment Info]"
echo "  Rust: $(rustc --version)"
echo "  Cargo: $(cargo --version)"
echo "  Node.js: $(node --version)"
echo "  pnpm: $(pnpm --version)"
echo "  Tauri CLI: $(cargo tauri --version 2>/dev/null || echo 'installed')"
echo ""

# Support both ANTHROPIC_API_KEY and CLAUDE_CODE_API_KEY
if [ -z "$ANTHROPIC_API_KEY" ] && [ -n "$CLAUDE_CODE_API_KEY" ]; then
    export ANTHROPIC_API_KEY="$CLAUDE_CODE_API_KEY"
fi

# Check for ANTHROPIC_API_KEY
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "[WARNING] ANTHROPIC_API_KEY is not set."
    echo "  Claude Code requires this environment variable."
    echo "  Set it with: docker run -e ANTHROPIC_API_KEY=your_key ..."
    echo ""
else
    echo "[OK] API Key configured"
fi

# If running Claude Code in automated mode
if [ "$1" = "claude-code" ]; then
    shift
    echo "[Starting Claude Code with --dangerously-skip-permissions]"
    exec claude --dangerously-skip-permissions "$@"
fi

# If running Claude Code interactively
if [ "$1" = "claude" ]; then
    shift
    echo "[Starting Claude Code]"
    exec claude "$@"
fi

# Default: execute the provided command
exec "$@"
