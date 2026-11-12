#!/bin/bash
set -euo pipefail

if [ ! -f /etc/rancher/k3s/k3s.yaml ]; then
  echo "k3s not found, skipping ingress cleanup"
  exit 0
fi

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

# Remove TLS secret
sudo -E kubectl delete secret {{ secret_name }} \
  --namespace ingress-nginx \
  --ignore-not-found

# Uninstall nginx-ingress
sudo -E helm uninstall ingress-nginx \
  --namespace ingress-nginx \
  --ignore-not-found \
  --timeout 30s \
  --wait=false

# Uninstall metallb
sudo -E helm uninstall metallb \
  --namespace metallb-system \
  --ignore-not-found \
  --timeout 30s \
  --wait=false

# Strip finalizers so namespaces don't get stuck terminating
for ns in ingress-nginx metallb-system; do
  sudo -E kubectl get namespace $ns -o json 2>/dev/null \
    | tr -d "\n" \
    | sed "s/\"finalizers\": \[[^]]\+\]/\"finalizers\": []/" \
    | sudo -E kubectl replace --raw /api/v1/namespaces/$ns/finalize -f - 2>/dev/null || true
done

# Remove namespaces
sudo -E kubectl delete namespace ingress-nginx --ignore-not-found --wait=false
sudo -E kubectl delete namespace metallb-system --ignore-not-found --wait=false