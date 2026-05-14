#!/bin/bash
set -euo pipefail

# Variables
GITHUB_RELEASE_URL="https://github.com/J040M/xcontroller/releases/latest/download/xcontroller"
SERVICE_NAME="xcontroller"
BIN_PATH="/usr/local/bin/${SERVICE_NAME}"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
TEMP_DIR="$(mktemp -d -t "${SERVICE_NAME}.XXXXXX")"
SERVICE_USER="${SERVICE_NAME}"
SERVICE_GROUP="dialout"

trap 'rm -rf "${TEMP_DIR}"' EXIT

# Parameters for the binary
WEBSOCKET_PORT="${1:-}"
SERIAL_PORT="${2:-}"
BAUDRATE="${3:-}"
TEST_MODE="${4:-}"

# Check if required parameters are provided
if [ -z "${WEBSOCKET_PORT}" ] || [ -z "${SERIAL_PORT}" ] || [ -z "${BAUDRATE}" ] || [ -z "${TEST_MODE}" ]; then
  echo "Error: Missing required parameters."
  echo "Usage: ./install_service.sh <websocket_port> <serial_port> <baudrate> <test_mode>"
  exit 1
fi

# 1. Stop the service if it's already running
if systemctl is-active --quiet "${SERVICE_NAME}"; then
  echo "Stopping the service..."
  sudo systemctl stop "${SERVICE_NAME}"
fi

# 2. Download the binary from GitHub release URL
echo "Downloading the binary from ${GITHUB_RELEASE_URL}..."
curl -fL --retry 3 -o "${TEMP_DIR}/${SERVICE_NAME}" "${GITHUB_RELEASE_URL}"

# Print the SHA256 of the downloaded binary so the operator can spot-check
# it against the release page before letting it run as a service.
echo "Downloaded binary SHA256:"
sha256sum "${TEMP_DIR}/${SERVICE_NAME}"

# 3. Install the binary atomically with correct mode and ownership
echo "Installing binary to ${BIN_PATH}..."
sudo install -o root -g root -m 0755 "${TEMP_DIR}/${SERVICE_NAME}" "${BIN_PATH}"

# 4. Create a dedicated unprivileged service user (in dialout for serial access)
if ! id -u "${SERVICE_USER}" >/dev/null 2>&1; then
  echo "Creating service user ${SERVICE_USER}..."
  sudo useradd --system --shell /usr/sbin/nologin --no-create-home \
    --groups "${SERVICE_GROUP}" "${SERVICE_USER}"
else
  echo "Service user ${SERVICE_USER} already exists; ensuring ${SERVICE_GROUP} membership..."
  sudo usermod -aG "${SERVICE_GROUP}" "${SERVICE_USER}"
fi

# 5. Write the systemd unit
echo "Writing systemd unit ${SERVICE_FILE}..."
sudo tee "${SERVICE_FILE}" >/dev/null <<EOL
[Unit]
Description=xcontroller 3D printer controller
After=network.target

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_GROUP}
ExecStart=${BIN_PATH} ${WEBSOCKET_PORT} ${SERIAL_PORT} ${BAUDRATE} ${TEST_MODE}
EnvironmentFile=-/etc/xcontroller/xcontroller.env
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOL

# 6. Reload, enable, start
sudo systemctl daemon-reload
sudo systemctl enable "${SERVICE_NAME}"
sudo systemctl restart "${SERVICE_NAME}"

cat <<EOF

Installed.

Optional: put security knobs in /etc/xcontroller/xcontroller.env (chmod 0600):
  XCONTROLLER_BIND_ADDR=0.0.0.0
  XCONTROLLER_AUTH_TOKEN=<a-long-random-string>
  XCONTROLLER_MAX_CLIENTS=8
  XCONTROLLER_MAX_UPLOAD_BYTES=67108864

Then: sudo systemctl restart ${SERVICE_NAME}

Logs: journalctl -u ${SERVICE_NAME} -f
EOF
