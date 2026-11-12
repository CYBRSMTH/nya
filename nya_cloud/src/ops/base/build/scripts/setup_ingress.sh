#!/bin/bash
set -euo pipefail

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

sudo -E helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
sudo -E helm repo add metallb https://metallb.github.io/metallb
sudo -E helm repo update
sudo -E helm upgrade --install metallb metallb/metallb \
  --namespace metallb-system \
  --create-namespace

sudo -E kubectl rollout status deployment/metallb-controller -n metallb-system --timeout=120s
sudo -E kubectl delete validatingwebhookconfigurations metallb-webhook-configuration --ignore-not-found

cat <<EOF | sudo -E kubectl apply -f -
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: default-pool
  namespace: metallb-system
spec:
  addresses:
  - {{ metallb_ip_range }}
---
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: default
  namespace: metallb-system
EOF

sudo -E helm upgrade --install ingress-nginx ingress-nginx/ingress-nginx \
  --namespace ingress-nginx \
  --create-namespace \
  --set controller.kind=DaemonSet \
  --set controller.service.type=LoadBalancer

sudo -E kubectl create secret tls {{ secret_name }} \
  --key /etc/nya/certs/{{ domain }}-key.pem \
  --cert /etc/nya/certs/{{ domain }}.pem \
  --namespace ingress-nginx