#!/bin/bash

# Variables
GITHUB_RELEASE_URL="https://github.com/J040M/xcontroller/releases/latest/download/xcontroller"        # GitHub release URL (you'll pass this as an argument)
BIN_PATH="/usr/local"  # Path where the binary is installed
SERVICE_NAME="xcontroller" # The name of the systemd service (e.g. "my_service")
SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME.service"  # Path to the systemd service file
TEMP_DIR="/tmp/xcontroller"  # Temporary directory for downloading the binary

# Parameters for the binary
WEBSOCKET_PORT=$1
SERIAL_PORT=$2
BAUDRATE=$3
TEST_MODE=$4

# Check if required parameters are provided
if [ -z "$WEBSOCKET_PORT" ] || [ -z "$SERIAL_PORT" ] || [ -z "$BAUDRATE" ] || [ -z "$TEST_MODE" ]; then
  echo "Error: Missing required parameters."
  echo "Usage: ./install_service.sh <websocket_port_value> <serial_port_string> <baudrate_value> <test_mode_boolean>"
  exit 1
fi

# 1. Check if the service is already running and stop it
if systemctl is-active --quiet $SERVICE_NAME; then
  echo "Stopping the service..."
  sudo systemctl stop $SERVICE_NAME
  if [ $? -ne 0 ]; then
    echo "Error: Failed to stop service $SERVICE_NAME"
    exit 1
  fi
else
  echo "Service $SERVICE_NAME is not running, skipping stop."
fi

# 2. Download the binary from GitHub release URL
echo "Downloading the binary from $GITHUB_RELEASE_URL..."
mkdir -p $TEMP_DIR
curl -L -o "$TEMP_DIR/$SERVICE_NAME" $GITHUB_RELEASE_URL
if [ $? -ne 0 ]; then
  echo "Error: Failed to download the binary"
  exit 1
fi

# 3. Install/Update the binary
echo "Installing/updating the binary..."
sudo mv "$TEMP_DIR/$SERVICE_NAME" $BIN_PATH
sudo chmod +x "$BIN_PATH/$SERVICE_NAME"
if [ $? -ne 0 ]; then
  echo "Error: Failed to install/update the binary"
  exit 1
fi

# 4. Install or Update the service
echo "Ensuring the service is installed/updated..."

if [ ! -f $SERVICE_FILE ]; then
  echo "Service file not found. Installing the service..."
  
  # Create the systemd service file
  cat > $SERVICE_FILE <<EOL
[Unit]
Description=xcontroller
After=network.target

[Service]
ExecStart=$BIN_PATH/$SERVICE_NAME -- $WEBSOCKET_PORT $SERIAL_PORT $BAUDRATE $TEST_MODE
Restart=always
User=root  # Adjust this to the user you want the service to run as
Group=root  # Optional, set if needed
StandardOutput=journal  # Logs output to journal (default)

[Install]
WantedBy=multi-user.target
EOL
  sudo systemctl daemon-reload
  sudo systemctl enable $SERVICE_NAME
  echo "Service installed and enabled!"
else
  echo "Service file already exists, updating..."
  sudo systemctl daemon-reload
  echo "Service updated!"
fi

# 5. Start the service
echo "Starting the service..."
sudo systemctl start $SERVICE_NAME
if [ $? -ne 0 ]; then
  echo "Error: Failed to start service $SERVICE_NAME"
  exit 1
fi

# Cleanup
echo "Cleaning up..."
rm -rf $TEMP_DIR
echo "Cleanup complete."

# Success message
echo "Binary installation and service update completed successfully!"
