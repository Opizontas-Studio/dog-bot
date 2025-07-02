FROM --platform=linux/amd64 rust:latest

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfreetype6-dev \
    libfontconfig1-dev \
    mold \
    clang \
    libssl-dev \
    ca-certificates \
    lsb-release \
    && rm -rf /var/lib/apt/lists/*

# Default command
CMD ["/bin/bash"]