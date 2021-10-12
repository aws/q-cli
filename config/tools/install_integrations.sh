#!/usr/bin/env bash

TMP_DIR=${TMPDIR:-'/tmp/'}

error() {
  echo "Error: $@" >&2
  exit 1
}

# TODO(sean) add backup for ssh config (not currently done in this file)
fig_backup() {
  full_path=$1
  name=$2
  if [[ -e "${full_path}" ]]; then
    backup_dir="${HOME}/.fig.dotfiles.bak/${date_string}"
    mkdir -p $backup_dir

    cp $full_path "${backup_dir}/${name}" || error "Failed to backup file $1"
  fi
}

fig_source() {
  printf "\n#### FIG ENV VARIABLES ####\n"
  if [[ -n $2 ]]; then
    printf "# $2\n"
  fi
  printf "[ -s ~/.fig/$1 ] && source ~/.fig/$1\n"
  printf "#### END FIG ENV VARIABLES ####\n"
}

fig_append() {
  # Appends line to a config file to source file from the ~/.fig directory.
  # Usage: fig_append fig.sh path/to/rc
  # Don't append to files that don't exist to avoid creating file and
  # changing shell behavior.
  if [ -f "$2" ] && ! grep -q "source ~/.fig/$1" "$2"; then
    echo "$(fig_source $1 "Please make sure this block is at the end of this file.")" >> "$2"
  fi
}

fig_prepend() {
  # Prepends line to a config file to source file from the ~/.fig directory.
  # Usage: fig_prepend fig_pre.sh path/to/rc
  # Don't prepend to files that don't exist to avoid creating file and
  # changing shell behavior.
  if [ -f "$2" ] && ! grep -q "source ~/.fig/$1" "$2"; then
    echo "$(fig_source $1 "Please make sure this block is at the start of this file.")" | cat - "$2" > "${TMP_DIR}fig_prepend" && cat "${TMP_DIR}fig_prepend" > "$2"
  fi
}

# Add the fig.sh to your profiles so it can be sourced on new terminal window load
append_to_profiles() {
  # Make sure one of [.bash_profile|.bash_login|.profile] exists to ensure fig
  # is sourced on login shells. We choose .profile to be as minimally
  # disruptive to existing user set up as possible.
  # https://superuser.com/questions/320065/bashrc-not-sourced-in-iterm-mac-os-x
  touch "${HOME}/.profile"

  # Replace old sourcing in profiles.
  for rc in .profile .zprofile .bash_profile; do
    if [[ -e "${HOME}/${rc}" ]]; then
      # Ignore failures if we don't find old contents.
      sed -i '' 's/~\/.fig\/exports\/env.sh/~\/.fig\/fig.sh/g' "${HOME}/${rc}" 2> /dev/null || :
    fi
  done
  

  # Create .zshrc/.bashrc regardless of whether it exists or not
  touch "${HOME}/.zshrc" "${HOME}/.bashrc"

  # Don't modify files until all are backed up.
  for rc in .profile .zprofile .bash_profile .bash_login .bashrc .zshrc; do
    fig_backup "${HOME}/${rc}" "${rc}"
  done

  for rc in .profile .zprofile .bash_profile .bash_login .bashrc .zshrc; do
    if [[ $1 != "--no-prepend" ]]; then
        fig_prepend shell/pre.sh "${HOME}/${rc}"
    fi
    fig_append fig.sh "${HOME}/${rc}"
  done

  # Handle fish separately.
  mkdir -p ~/.config/fish/conf.d

  # Use 00_/99_ prefix to load script earlier/later in fish startup.
  ln -sf ~/.fig/shell/pre.fish ~/.config/fish/conf.d/00_fig_pre.fish
  ln -sf ~/.fig/shell/post.fish ~/.config/fish/conf.d/99_fig_post.fish

  # Remove deprecated fish config file.
  if [[ -f ~/.config/fish/conf.d/fig.fish ]]; then
    rm ~/.config/fish/conf.d/fig.fish
  fi
}

install_tmux_integration() {
  TMUX_INTEGRATION=$'\n# Fig Tmux Integration: Enabled\nsource-file ~/.fig/tmux\n# End of Fig Tmux Integration'

  # If ~/.tmux.conf.local exists, append integration here to workaround conflict with oh-my-tmux.
  if [[ -s "${HOME}/.tmux.conf.local" ]]; then
    fig_backup "${HOME}/.tmux.conf.local" ".tmux.conf.local"
    if ! grep -q 'source-file ~/.fig/tmux' ~/.tmux.conf.local; then 
      echo "${TMUX_INTEGRATION}" >> ~/.tmux.conf.local
    fi
  elif [[ -s "${HOME}/.tmux.conf" ]]; then
    fig_backup "${HOME}/.tmux.conf" ".tmux.conf"
    if ! grep -q 'source-file ~/.fig/tmux' ~/.tmux.conf; then 
      echo "${TMUX_INTEGRATION}" >> ~/.tmux.conf
    fi
  fi
}

if [[ $1 == "--minimal" ]]; then
  append_to_profiles --no-prepend
else
  append_to_profiles
  install_tmux_integration
fi
