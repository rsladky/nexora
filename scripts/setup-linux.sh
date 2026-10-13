#!/usr/bin/env bash
# Autorise l'accès à /dev/uinput (contrôle souris/clavier sous Wayland) pour l'utilisateur courant.
set -euo pipefail
echo 'KERNEL=="uinput", GROUP="input", MODE="0660", OPTIONS+="static_node=uinput"' |
  sudo tee /etc/udev/rules.d/99-nexora-uinput.rules
sudo usermod -aG input "$USER"
sudo udevadm control --reload-rules && sudo udevadm trigger
echo "Reconnecte ta session pour appliquer le groupe 'input'."
