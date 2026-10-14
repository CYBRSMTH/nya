#!/bin/bash
set -euo pipefail

# Remove certs
sudo rm -f /etc/nya/certs/{{ domain }}.pem
sudo rm -f /etc/nya/certs/{{ domain }}-key.pem

# Remove mkcert CA
mkcert -uninstall || true

# Remove mkcert binary
sudo rm -f /usr/local/bin/mkcert

# Remove certs directory if empty
[ -d /etc/nya/certs ] && sudo rmdir --ignore-fail-on-non-empty /etc/nya/certs || true