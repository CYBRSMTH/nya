#!/bin/bash
set -euo pipefail

LATEST=$(sudo curl -fsSL https://api.github.com/repos/FiloSottile/mkcert/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
sudo curl -fsSL "https://github.com/FiloSottile/mkcert/releases/download/${LATEST}/mkcert-${LATEST}-linux-amd64" -o /usr/local/bin/mkcert
sudo chmod +x /usr/local/bin/mkcert
sudo mkcert -install
sudo mkdir -p /etc/nya/certs
sudo mkcert \
  -cert-file /etc/nya/certs/{{ domain }}.pem \
  -key-file /etc/nya/certs/{{ domain }}-key.pem \
  "*.{{ domain }}" {{ domain }}