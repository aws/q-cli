mkdir -p ~/.fig/

curl -Ls fig.sh/shell-integration.tar.gz | tar -xz -C ~/.fig &&
  ~/.fig/tools/install_integrations.sh --minimal &&
  echo Successfully installed fig shell integrations ||
  echo Failed to download and extract shell integration

[ -n "$FISH_VERSION" ] && source ~/.fig/shell/post.fish || source ~/.fig/shell/post.sh
