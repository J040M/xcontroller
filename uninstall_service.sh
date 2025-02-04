#!/bin/bash

BIN_PATH="/usr/local/"  # Path where the binary is installed
SERVICE_NAME="xcontroller" # The name of the systemd service (e.g. "my_service")
SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME.service"  # Path to the systemd service file

# 1. Stop the service
echo "Stopping the service..."
sudo systemctl stop $SERVICE_NAME
if [ $? -ne 0 ]; then
  echo "Error: Failed to stop service $SERVICE_NAME"
  exit 1
fi

# 2. Disable the service (prevent it from starting at boot)
echo "Disabling the service..."
sudo systemctl disable $SERVICE_NAME
if [ $? -ne 0 ]; then
  echo "Error: Failed to disable service $SERVICE_NAME"
  exit 1
fi

# 3. Remove the systemd service file
echo "Removing the service file..."
sudo rm -f $SERVICE_FILE
if [ $? -ne 0 ]; then
  echo "Error: Failed to remove service file $SERVICE_FILE"
  exit 1
fi

# 4. Reload systemd to remove references to the deleted service
echo "Reloading systemd..."
sudo systemctl daemon-reload

# 5. (Optional) Remove the binary
echo "Do you want to remove the binary ($BIN_PATH/$SERVICE_NAME)? (y/n)"
read -r REMOVE_BINARY
if [ "$REMOVE_BINARY" == "y" ]; then
  echo "Removing the binary..."
  sudo rm -f $BIN_PATH/$SERVICE_NAME
  if [ $? -ne 0 ]; then
    echo "Error: Failed to remove binary "
    exit 1
  fi
fi

# Success message
echo "Service $SERVICE_NAME has been uninstalled successfully!"

