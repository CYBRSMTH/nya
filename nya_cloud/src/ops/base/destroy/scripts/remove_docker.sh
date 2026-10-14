#!/bin/bash
set -euo pipefail

# Remove ALL conflicting Docker apt sources first
sudo rm -f /etc/apt/sources.list.d/download_docker_com_linux_ubuntu.list
sudo rm -f /etc/apt/sources.list.d/docker.list
sudo rm -f /etc/apt/sources.list.d/docker-ce.list
sudo rm -f /etc/apt/keyrings/docker.gpg
sudo rm -f /etc/docker/daemon.json


# Now apt can actually run
sudo systemctl stop docker.socket || true
sudo systemctl stop docker || true
sudo systemctl disable docker.socket || true
sudo systemctl disable docker || true
sudo dpkg --configure -a
sudo apt remove --purge -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin || true
sudo apt autoremove -y

sudo rm -rf /var/lib/docker
sudo rm -rf /var/lib/containerd

sudo gpasswd -d ${USER} docker || true

sudo apt update -y

echo "Docker uninstalled successfully"