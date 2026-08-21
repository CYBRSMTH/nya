#!/bin/bash
set -euo pipefail

sudo apt-get install -y iptables || true
sudo iptables -F
sudo iptables -t nat -F
sudo iptables -t mangle -F
sudo iptables -X

if [ -f /usr/local/bin/k3s-agent-uninstall.sh ]; then
  sudo /usr/local/bin/k3s-agent-uninstall.sh
else
  echo "k3s-agent uninstall script not found, may already be uninstalled"
fi

sudo rm -rf /etc/rancher
sudo rm -rf /var/lib/rancher
