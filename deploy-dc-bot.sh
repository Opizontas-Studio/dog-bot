#!/bin/bash

# Deploy script for dc-bot
# Builds locally and deploys to Oracle server

set -e  # Exit on any error

# Configuration
ORACLE_HOST="Oracle"
ORACLE_USER=""  # Add your username if needed (e.g., "user@Oracle")
REMOTE_PATH="bot/"
BINARY_NAME="dc-bot"
SCREEN_SESSION_NAME="dc-bot"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root directory."
    exit 1
fi

# Step 1: Build the binary locally
print_step "Building binary locally for x86_64-unknown-linux-gnu..."
if CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc cargo build --release --target x86_64-unknown-linux-gnu; then
    print_success "Binary built successfully"
else
    print_error "Failed to build binary"
    exit 1
fi

# Check if binary exists
BINARY_PATH="target/x86_64-unknown-linux-gnu/release/${BINARY_NAME}"
if [ ! -f "$BINARY_PATH" ]; then
    print_error "Binary not found at $BINARY_PATH"
    exit 1
fi

print_success "Binary found at $BINARY_PATH"

# Step 2: Kill existing screen session on Oracle
print_step "Stopping existing screen session '$SCREEN_SESSION_NAME' on Oracle..."
if ssh $ORACLE_USER$ORACLE_HOST "screen -S $SCREEN_SESSION_NAME -X quit" 2>/dev/null; then
    print_success "Existing screen session terminated"
else
    print_warning "No existing screen session found (or failed to terminate)"
fi

# Step 3: Upload binary to Oracle
print_step "Uploading binary to Oracle:$REMOTE_PATH..."
if scp "$BINARY_PATH" $ORACLE_USER$ORACLE_HOST:$REMOTE_PATH; then
    print_success "Binary uploaded successfully"
else
    print_error "Failed to upload binary"
    exit 1
fi

# Step 4: Make binary executable on Oracle
print_step "Making binary executable on Oracle..."
if ssh $ORACLE_USER$ORACLE_HOST "chmod +x ${REMOTE_PATH}${BINARY_NAME}"; then
    print_success "Binary is now executable"
else
    print_error "Failed to make binary executable"
    exit 1
fi

# Step 5: Start new screen session
print_step "Starting new screen session '$SCREEN_SESSION_NAME' on Oracle..."
if ssh $ORACLE_USER$ORACLE_HOST "cd $REMOTE_PATH && screen -dmS $SCREEN_SESSION_NAME ./$BINARY_NAME"; then
    print_success "New screen session started"
else
    print_error "Failed to start screen session"
    exit 1
fi

# Step 6: Verify the session is running
print_step "Verifying screen session is running..."
sleep 2  # Give it a moment to start
if ssh $ORACLE_USER$ORACLE_HOST "screen -list | grep -q $SCREEN_SESSION_NAME"; then
    print_success "Screen session '$SCREEN_SESSION_NAME' is running"
    
    # Show screen sessions
    print_step "Current screen sessions on Oracle:"
    ssh $ORACLE_USER$ORACLE_HOST "screen -list"
    
    echo ""
    echo -e "${GREEN}Deployment completed successfully!${NC}"
    echo -e "To attach to the session: ${YELLOW}ssh $ORACLE_USER$ORACLE_HOST -t 'screen -r $SCREEN_SESSION_NAME'${NC}"
    echo -e "To detach from session: ${YELLOW}Ctrl+A, then D${NC}"
    echo -e "To view logs: ${YELLOW}ssh $ORACLE_USER$ORACLE_HOST -t 'screen -r $SCREEN_SESSION_NAME'${NC}"
else
    print_error "Screen session failed to start or is not running"
    print_step "Checking for any error logs..."
    ssh $ORACLE_USER$ORACLE_HOST "cd $REMOTE_PATH && ls -la $BINARY_NAME"
    exit 1
fi