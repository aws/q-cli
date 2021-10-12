#!/usr/bin/env bash
heading() {
    printf "\n\n===$1===\n"
}


HIGHLIGHT=$(tput smso)
HIGHLIGHT_END=$(tput rmso)

# Structure.
TAB='   '

press_any_key_to_continue() {
  echo # new line
  read -n 1 -s -r -p "${TAB}${HIGHLIGHT} Press any key to continue ${HIGHLIGHT_END}"
  echo # new line
  echo # new line
  clear
}

case "$1" in
  "logs")
    fig settings developer.logging true

    if [ "$#" -eq 1 ]; then
        tail -n0 -qf ~/.fig/logs/*.log
    else
        shift
        array=( "$@" )
        array=( "${array[@]/#/$HOME/.fig/logs/}" ) # prepend ~/.fig/logs
        array=( "${array[@]/%/.log}" ) # append .log
        tail -n0 -qf "${array[@]}"
    fi
    fig settings developer.logging false

    ;;
  "app")
    if [ "$(fig app:running)" -eq 0 ]; then
      echo "Fig app is not currently running..."
      /Applications/Fig.app/Contents/MacOS/fig
      exit 0
    fi

    BUNDLE_PATH=$(lsappinfo info -only "bundlepath" -app com.mschrage.fig | cut -f2 -d= | tr -d '"')
    TERMINAL_EMULATOR=$(lsappinfo info -only "name" -app $(lsappinfo front) | cut -f2 -d= | tr -d '"')
    fig quit > /dev/null
    echo "Running the Fig.app executable directly from $BUNDLE_PATH."
    echo "You will need to grant accessibility permissions to the current terminal ($TERMINAL_EMULATOR)!"

    $BUNDLE_PATH/Contents/MacOS/fig
    ;;
  "sample")
    PID=$(lsappinfo info -only "pid" -app com.mschrage.fig | cut -f2 -d=)
    OUT_FILE=/tmp/fig-sample
    echo "Sampling Fig process ($PID). Writing output to $OUT_FILE"
    sample $PID -f $OUT_FILE
    printf "\n\n\n-------\nFinished writing to $OUT_FILE\n"
    echo  "Please send this file to the Fig Team"
    echo  "Or attach it to a GitHub issue (run 'fig issue')"
    ;;
  "terminal")
    clear
    heading "tty characteristics"
    stty -a
    press_any_key_to_continue
    heading "environment vars"
    env
    ;;
  "dotfiles")
  "$HOME"/.fig/tools/cli/email_dotfiles.sh
    ;;
  # "shell-startup")

  #   echo "Append 'set -x' to /etc/profile. You will be prompted for permission."
  #   printf "\nset -x" | sudo tee -a /etc/profile
  #   open /etc/profile
  #   ;;
  "prefs")
    clear
    heading settings.json
    cat ~/.fig/settings.json
    press_any_key_to_continue

    heading user/config
    cat ~/.fig/user/config
    press_any_key_to_continue

    heading NSUserDefaults
    defaults read com.mschrage.fig
    defaults read com.mschrage.fig.shared

    ;;
  "unix-socket")
    echo Listening on /tmp/fig.socket...
    echo "Note: You will need to restart Fig afterwards"
    rm /tmp/fig.socket && nc -Ulk /tmp/fig.socket
    echo "Remember to restart Fig!"

    ;;
  "verify-codesign")
    codesign -vvvv /Applications/Fig.app
    ;;
  *)
    echo "Not a valid debug command."
    exit 1
    ;;
esac