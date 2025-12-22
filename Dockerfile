# OBS Optimizer Development Environment
# Tauri (Rust + React) + Claude Code CLI

FROM ubuntu:22.04

# Prevent interactive prompts during installation
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Asia/Tokyo

# System dependencies for Tauri development
RUN apt-get update && apt-get install -y \
    # Basic build tools
    build-essential \
    curl \
    wget \
    git \
    pkg-config \
    # Tauri 2.x dependencies (WebKit/GTK 4.1)
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    # SSL and crypto
    libssl-dev \
    # Additional Tauri 2.x dependencies
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    # For GPU monitoring (NVML)
    libnvidia-ml-dev || true \
    # Useful tools
    ca-certificates \
    gnupg \
    file \
    xdg-utils \
    # For headless testing
    xvfb \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && rustup default stable \
    && rustup component add rustfmt clippy

# Install Node.js (v20 LTS)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install pnpm (faster package manager)
RUN npm install -g pnpm

# Install Tauri CLI
RUN cargo install tauri-cli

# Install Claude Code CLI
RUN npm install -g @anthropic-ai/claude-code

# Create workspace directory
WORKDIR /workspace

# Create non-root user for development
RUN useradd -m -s /bin/bash developer \
    && chown -R developer:developer /workspace \
    && chown -R developer:developer /usr/local/cargo \
    && chown -R developer:developer /usr/local/rustup

# Switch to developer user
USER developer

# Set up shell environment
RUN echo 'export PATH="/usr/local/cargo/bin:$PATH"' >> ~/.bashrc

# Expose Tauri dev server port
EXPOSE 1420

# Entry point script
COPY --chown=developer:developer entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["bash"]
