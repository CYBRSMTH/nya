#!/bin/bash
set -euo pipefail

sudo apt-get install -y iptables || true
sudo iptables -F
sudo iptables -t nat -F
sudo iptables -t mangle -F
sudo iptables -X

sudo docker stop registry || true
sudo docker rm registry || true

if [ -f /usr/local/bin/k3s-uninstall.sh ]; then
  sudo /usr/local/bin/k3s-uninstall.sh
else
  echo "k3s uninstall script not found, may already be uninstalled"
fi

sudo rm -rf /etc/rancher
sudo rm -rf /var/lib/rancher
