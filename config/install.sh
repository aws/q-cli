#!/usr/bin/env bash
if [ -z "$1" ]; then
  ROOT="${HOME%/}"
else
  ROOT="$1"
fi

fig_source() {
  echo "[ -s ~/.fig_pty/$1 ] && source ~/.fig_pty/$1"
}

fig_append() {
  if [ -f "$2" ] && ! grep -q "source ~/.fig_pty/$1" "$2"; then
    echo "$(fig_source $1)" >> "$2"
  fi
}

fig_prepend() {
  if [ -f "$2" ] && ! grep -q "source ~/.fig_pty/$1" "$2"; then
    echo -e "$(fig_source $1)\n$(cat $2)" > $2
  fi
}

append_to_profiles() {
  for profile in .profile .zprofile .bash_profile .bashrc .zshrc; do
    fig_prepend pre.sh "$ROOT/$profile"
    fig_append post.sh "$ROOT/$profile"
  done

  mkdir -p "$ROOT/.config/fish"
  fish_config="$ROOT/.config/fish/config"
  touch "$fish_config"
  fig_prepend pre.fish "$fish_config"
  fig_append post.fish "$fish_config"
}

append_to_profiles
