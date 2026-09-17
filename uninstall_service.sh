#!/bin/bash
set -euo pipefail

SERVICE_NAME="xcontroller"
BIN_PATH="/usr/local/bin/${SERVICE_NAME}"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
SERVICE_USER="${SERVICE_NAME}"
ENV_DIR="/etc/xcontroller"

# 1. Stop the service if running (don't fail if it isn't)
if systemctl is-active --quiet "${SERVICE_NAME}"; then
  echo "Stopping the service..."
  sudo systemctl stop "${SERVICE_NAME}"
fi

# 2. Disable it (ignore failure if it was never enabled)
echo "Disabling the service..."
sudo systemctl disable "${SERVICE_NAME}" 2>/dev/null || true

# 3. Remove the unit file
if [ -f "${SERVICE_FILE}" ]; then
  echo "Removing ${SERVICE_FILE}..."
  sudo rm -f "${SERVICE_FILE}"
fi

# 4. Reload systemd
sudo systemctl daemon-reload

# 5. Optionally remove the binary
if [ -f "${BIN_PATH}" ]; then
  read -r -p "Remove the binary at ${BIN_PATH}? [y/N] " REMOVE_BINARY
  if [ "${REMOVE_BINARY:-n}" = "y" ] || [ "${REMOVE_BINARY:-n}" = "Y" ]; then
    sudo rm -f "${BIN_PATH}"
    echo "Removed ${BIN_PATH}"
  fi
fi

# 6. Optionally remove the service user (only if no other files belong to it)
if id -u "${SERVICE_USER}" >/dev/null 2>&1; then
  read -r -p "Remove the service user '${SERVICE_USER}'? [y/N] " REMOVE_USER
  if [ "${REMOVE_USER:-n}" = "y" ] || [ "${REMOVE_USER:-n}" = "Y" ]; then
    sudo userdel "${SERVICE_USER}" || true
  fi
fi

# 7. Optionally remove the env directory (may contain the auth token)
if [ -d "${ENV_DIR}" ]; then
  read -r -p "Remove ${ENV_DIR} (may contain XCONTROLLER_AUTH_TOKEN)? [y/N] " REMOVE_ENV
  if [ "${REMOVE_ENV:-n}" = "y" ] || [ "${REMOVE_ENV:-n}" = "Y" ]; then
    sudo rm -rf "${ENV_DIR}"
  fi
fi

echo "Service ${SERVICE_NAME} uninstalled."
