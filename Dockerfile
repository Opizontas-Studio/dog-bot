FROM --platform=linux/amd64 ubuntu:latest

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libfreetype6-dev \
    libfontconfig1-dev \
    rustup \
    curl \
    wget \
    git \
    libssl-dev \
    ca-certificates \
    gnupg \
    lsb-release \
    sudo \
    vim \
    nano \
    htop \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for development
RUN useradd -m -s /bin/bash rustdev && \
    echo 'rustdev ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

# Switch to the development user
USER rustdev
WORKDIR /home/rustdev

# Install Rust via rustup
RUN rustup default stable && \
    rustup update

# Set up workspace directory
WORKDIR /workspace

# Default command
CMD ["/bin/bash"]