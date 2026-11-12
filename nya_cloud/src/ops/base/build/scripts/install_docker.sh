#!/bin/bash
set -euo pipefail

sudo apt update -y 
sudo apt install apt-transport-https ca-certificates curl software-properties-common -y
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --batch --no-tty --dearmor -o /etc/apt/keyrings/docker.gpg
# arch is hardcoded for now, will need to parameterize this for arm64 support in the future
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list
sudo apt update -y
sudo apt install docker-ce -y
sudo usermod -aG docker ${USER}