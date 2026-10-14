#!/bin/bash
set -euo pipefail

sudo systemctl stop bind9 || true
sudo apt remove --purge bind9 bind9utils -y
sudo apt autoremove -y
sudo rm -f /etc/bind/named.conf.local
sudo rm -f /etc/bind/named.conf.options
sudo rm -rf /etc/bind/zones
