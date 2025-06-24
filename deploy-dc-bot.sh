#!/bin/bash

# Deploy script for dc-bot
# Builds locally and deploys to Oracle server as systemd service

set -e # Exit on any error

# Configuration
ORACLE_HOST="Oracle"
ORACLE_USER="ubuntu@" # Add your username if needed (e.g., "user@Oracle")
REMOTE_PATH="bot/"
BINARY_NAME="dc-bot"
SERVICE_NAME="dc-bot"
SERVICE_USER="ubuntu" # Change this if you want to run as a different user

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
# if on macOS, export ENV variables for cross-compilation
if [[ "$OSTYPE" == "darwin"* ]]; then
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
    export PKG_CONFIG=/opt/homebrew/bin/pkg-config-wrapper
fi
if cargo build --release --target x86_64-unknown-linux-gnu; then
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

# Step 2: Stop existing systemd service on Oracle
print_step "Stopping existing systemd service '$SERVICE_NAME' on Oracle..."
if ssh $ORACLE_USER$ORACLE_HOST "sudo systemctl stop $SERVICE_NAME" 2>/dev/null; then
    print_success "Existing service stopped"
else
    print_warning "Service was not running or doesn't exist yet"
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

# Step 5: Get the actual home directory path on remote server
print_step "Getting remote user's home directory..."
REMOTE_HOME=$(ssh $ORACLE_USER$ORACLE_HOST "echo \$HOME")
print_success "Remote home directory: $REMOTE_HOME"

# Step 6: Create systemd service file
print_step "Creating systemd service file..."
SERVICE_FILE_CONTENT="[Unit]
Description=Discord Bot Service
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=$SERVICE_USER
ExecStart=${REMOTE_HOME}/${REMOTE_PATH}${BINARY_NAME} -c ${REMOTE_HOME}/${REMOTE_PATH}/config.json -r ${REMOTE_HOME}/${REMOTE_PATH}/db.redb
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=$SERVICE_NAME

# Environment variables (add your bot's env vars here)
# Environment=DISCORD_TOKEN=your_token_here
# Environment=DATABASE_URL=your_db_url_here
Environment=XDG_CACHE_HOME=${REMOTE_HOME}/${REMOTE_PATH}/.cache
Environment=SYSTEMD_COLOR=1
Environment=RUST_LOG_STYLE=always
Environment=RUST_LOG=dc_bot=debug

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=${REMOTE_HOME}/${REMOTE_PATH}

[Install]
WantedBy=multi-user.target"

# Upload service file
echo "$SERVICE_FILE_CONTENT" | ssh $ORACLE_USER$ORACLE_HOST "sudo tee /etc/systemd/system/${SERVICE_NAME}.service > /dev/null"
print_success "Service file created"

# Step 7: Reload systemd and enable service
print_step "Reloading systemd daemon..."
if ssh $ORACLE_USER$ORACLE_HOST "sudo systemctl daemon-reload"; then
    print_success "Systemd daemon reloaded"
else
    print_error "Failed to reload systemd daemon"
    exit 1
fi

print_step "Enabling service to start on boot..."
if ssh $ORACLE_USER$ORACLE_HOST "sudo systemctl enable $SERVICE_NAME"; then
    print_success "Service enabled for auto-start"
else
    print_error "Failed to enable service"
    exit 1
fi

# Step 8: Start the service
print_step "Starting service '$SERVICE_NAME'..."
if ssh $ORACLE_USER$ORACLE_HOST "sudo systemctl start $SERVICE_NAME"; then
    print_success "Service started"
else
    print_error "Failed to start service"
    exit 1
fi

# Step 9: Verify the service is running
print_step "Verifying service status..."
sleep 3 # Give it a moment to start

SERVICE_STATUS=$(ssh $ORACLE_USER$ORACLE_HOST "sudo systemctl is-active $SERVICE_NAME" 2>/dev/null || echo "failed")

if [ "$SERVICE_STATUS" = "active" ]; then
    print_success "Service '$SERVICE_NAME' is running successfully"

    # Show service status
    print_step "Service status:"
    ssh $ORACLE_USER$ORACLE_HOST "sudo systemctl status $SERVICE_NAME --no-pager -l"

    echo ""
    echo -e "${GREEN}Deployment completed successfully!${NC}"
    echo ""
    echo -e "${YELLOW}Useful commands:${NC}"
    echo -e "View status:      ${BLUE}ssh $ORACLE_USER$ORACLE_HOST 'sudo systemctl status $SERVICE_NAME'${NC}"
    echo -e "View logs:        ${BLUE}ssh $ORACLE_USER$ORACLE_HOST 'sudo journalctl -u $SERVICE_NAME -f --output cat'${NC}"
    echo -e "Stop service:     ${BLUE}ssh $ORACLE_USER$ORACLE_HOST 'sudo systemctl stop $SERVICE_NAME'${NC}"
    echo -e "Start service:    ${BLUE}ssh $ORACLE_USER$ORACLE_HOST 'sudo systemctl start $SERVICE_NAME'${NC}"
    echo -e "Restart service:  ${BLUE}ssh $ORACLE_USER$ORACLE_HOST 'sudo systemctl restart $SERVICE_NAME'${NC}"
    echo -e "Disable service:  ${BLUE}ssh $ORACLE_USER$ORACLE_HOST 'sudo systemctl disable $SERVICE_NAME'${NC}"
else
    print_error "Service failed to start or is not running"
    print_step "Checking service logs for errors..."
    ssh $ORACLE_USER$ORACLE_HOST "sudo journalctl -u $SERVICE_NAME --no-pager -l"
    exit 1
fi
