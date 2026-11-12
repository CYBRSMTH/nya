#!/bin/bash
set -euo pipefail

sudo docker run -d \
  -p 5000:5000 \
  -e REGISTRY_HTTP_TLS_ENABLE=false \
  -e REGISTRY_STORAGE_DELETE_ENABLED=true \
  --restart always \
  --name registry \
  registry:2

curl -sfL https://get.k3s.io | INSTALL_K3S_TOKEN={{ k3s_token }} sh -s - server \
  --disable traefik \
  --disable servicelb \
  --disable local-storage \
  --advertise-address={{ control_plane_ip }} \
  --node-ip={{ control_plane_ip }}