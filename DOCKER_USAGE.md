# Docker Development Environment Usage

## Overview

This Docker environment provides everything needed for OBS Optimizer development:
- Rust toolchain (cargo, rustc, rustfmt, clippy)
- Node.js 20 LTS + pnpm
- Tauri CLI
- Claude Code CLI

## Quick Start

### 1. Set API Key

Create a `.env` file or export the environment variable:

```bash
export ANTHROPIC_API_KEY=your_api_key_here
```

### 2. Build the Image

```bash
docker-compose build
```

### 3. Start Development Container

```bash
# Interactive shell
docker-compose run --rm obs-optimizer-dev

# Or keep running in background
docker-compose up -d obs-optimizer-dev
docker-compose exec obs-optimizer-dev bash
```

## Using Claude Code

### Interactive Mode

```bash
# Inside the container
claude

# Or from host
docker-compose exec obs-optimizer-dev claude
```

### Automated Mode (--dangerously-skip-permissions)

```bash
# Using entrypoint shortcut
docker-compose run --rm obs-optimizer-dev claude-code "Create the initial Tauri project structure"

# Or directly
docker-compose run --rm obs-optimizer-dev claude --dangerously-skip-permissions --print "Your prompt here"
```

### Using the Claude Agent Profile

```bash
# Start the dedicated Claude agent service
docker-compose --profile claude run --rm claude-agent
```

## Common Commands

### Initialize New Tauri Project

```bash
# Inside container
pnpm create tauri-app
# or
cargo tauri init
```

### Development Server

```bash
cargo tauri dev
```

### Build for Production

```bash
cargo tauri build
```

### Run Tests

```bash
cargo test
pnpm test
```

## Directory Structure

```
/workspace/          <- Mounted project directory
/usr/local/cargo/    <- Rust toolchain
/home/developer/     <- User home (caches)
```

## Troubleshooting

### GUI Issues

For Tauri GUI testing, you may need X11 forwarding:

```bash
# On Linux host
xhost +local:docker
docker-compose run --rm -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix obs-optimizer-dev
```

### Permission Issues

If you encounter permission issues with mounted files:

```bash
# Run as root temporarily
docker-compose run --rm -u root obs-optimizer-dev chown -R developer:developer /workspace
```

### Slow Builds

The docker-compose.yml includes volume mounts for Cargo cache. First build will be slow, subsequent builds should be faster.
