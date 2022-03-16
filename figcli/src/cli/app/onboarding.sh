#!/usr/bin/env bash

# Fig onboarding shell script.
# Based somewhat on oh my zshell https://github.com/ohmyzsh/ohmyzsh/blob/master/tools/install.sh
set -e

# Force current process to be shell, rather than `env`.
cd ~
TTY=$(tty)
fig hook prompt $$ $TTY 2>&1 1>/dev/null

# Colors
YELLOW=$(tput setaf 3)
MAGENTA=$(tput setaf 5)

# Weights and decoration.
BOLD=$(tput bold)
UNDERLINE=$(tput smul)
HIGHLIGHT=$(tput smso)
NORMAL=$(tput sgr0)

# Structure.
TAB='   '
SEPARATOR="  \n\n  --\n\n\n"

function fig_osc { printf "\033]697;"; printf $@; printf "\007"; }

START_PROMPT="$(fig_osc StartPrompt)"
END_PROMPT="$(fig_osc EndPrompt)"
NEW_CMD="$(fig_osc NewCmd)"
END_CMD="$(fig_osc PreExec)"

DEFAULT_PROMPT="${START_PROMPT}${TAB}$ ${END_PROMPT}${NEW_CMD}"

function prepare_prompt {
  fig_osc "Dir=%s" "${PWD}"
  fig_osc "Shell=bash"
  fig_osc "PID=%d" "$$"
  fig_osc "SessionId=%s" "${TERM_SESSION_ID}"
  fig_osc "TTY=%s" "${TTY}"
}

function reset_prompt {
    (fig hook pre-exec $$ $TTY 2>&1 1>/dev/null)
}

print_special() {
  echo "${START_PROMPT}${TAB}$@${NORMAL}"$'\n'${END_PROMPT}
  reset_prompt
}

press_enter_to_continue() {
  echo ${START_PROMPT} # new line

  if [[ "$1" != "" ]]; then
    read -n 1 -s -r -p "${TAB}${HIGHLIGHT} $1 ${NORMAL}" pressed_key 
  else
    read -n 1 -s -r -p "${TAB}${HIGHLIGHT} Press enter to continue ${NORMAL}" pressed_key 
  fi
  printf ${END_PROMPT}

  while true; do
    # ie if pressed_key = enter
    if [[ "$pressed_key" == "" ]]; then
      echo # new line
      echo # new line
      break
    else 
      read -n 1 -s -r pressed_key
    fi
  done
}

# In case user quits script
exit_script_nice() {
  sed -i='' "s/FIG_ONBOARDING=.*/FIG_ONBOARDING=1/g" ~/.fig/user/config 2> /dev/null

clear 
cat <<EOF

  ${BOLD}${UNDERLINE}Fig's onboarding was quit${NORMAL}
  
  You can redo this onboarding any time. Just run ${BOLD}${MAGENTA}fig onboarding${NORMAL}
   

  Have an issue? Run ${BOLD}${MAGENTA}fig doctor${NORMAL}
  Have feedback? Email ${UNDERLINE}hello@fig.io${NORMAL}


EOF

  trap - SIGINT SIGTERM SIGQUIT # clear the trap
  fig hook event "Quit Shell Onboarding" 2>&1 1>/dev/null
  exit 1
}

# If the user does ctrl + c, run the exit_script function
trap exit_script_nice SIGINT SIGTERM SIGQUIT

# Help text
show_help() {
   # make sure the final EOF is aligned with the end 
less -R <<EOF


   ${BOLD}${MAGENTA}${UNDERLINE}Fig Onboarding Help${NORMAL}
   (press q to quit)



   ${BOLD}The Fig autocomplete box disappeared${NORMAL}
      This can happen if you hit 
         * ${BOLD}esc${NORMAL}
         * the ${BOLD}↑${NORMAL} up arrow too many times (after the up arrow shows your history, Fig hides until the next line)

      ${UNDERLINE}To bring it back${NORMAL}: hit the enter key on an empty line once or twice. It should reappear. 


   ${BOLD}Where is the Fig Menu${NORMAL}
      Click the Fig Icon (◧) in your Mac status bar (top right of your screen)


   ${BOLD}I don't see Fig popup next to my cursor${NORMAL}
      Hmm. Try some of the following to debug.

      1. Hit enter a few times then start typing. Maybe you hid it by hitting the up arrow key too many times.

      2. Make sure the Fig CLI tool is installed:
         * Go to Fig Menu (◧) > Settings > Developer > Install CLI Tool 

      3. Make sure Accessibility is enabled
         * Go to Fig Menu (◧) > Settings > Developer > Request Accessibility Permission
           (This should take you to System Preferences > Security & Privacy > Accessibility)
         * Click the lock icon to unlock (it may prompt for your password)
         * If Fig is unchecked, check it. If Fig is checked, uncheck it then check it again.

      4. Toggle Autocomplete off and on again
         * Go to Fig Menu (◧) > Autocomplete 


      If the problem persists: please let us know! Contact the Fig team at hello@fig.io


   ${BOLD}What does the ↪ symbol / suggestion mean?${NORMAL}
      This lets you run the command that's currently in your Terminal. 
      Sometimes Fig's autocomplete appears when you actually want to run a command. Rather than clicking escape or the up arrow, this lets you run the command by clicking enter.
   


   ${BOLD}I want to quit this onboarding / walkthrough${NORMAL}
      Hit ctrl + c



   ${BOLD}I want to quit Fig${NORMAL}
      * Go to Fig Menu (◧) > Quit Fig

   

   ${BOLD}I want to uninstall Fig${NORMAL}
      * Go to Fig Menu (◧) > Settings > Uninstall Fig
      3. If you're feeling generous, we would love to hear why you uninstalled Fig. hello@fig.io
   


   ${BOLD}What is cd?${NORMAL}
      cd is a shell command that lets you change directories. e.g. cd ~/Desktop will change the current directory in your shell to the Desktop.



EOF
  reset_prompt
}

### Core Script ###
clear

# Make absolutely sure that settings listener has been launched!
(fig settings init 2>&1 1>/dev/null)

# Done using http://patorjk.com/software/taag/#p=testall&f=Graffiti&t=fig
# Font name = ANSI Shadow
cat <<'EOF'


   ███████╗██╗ ██████╗ 
   ██╔════╝██║██╔════╝ 
   █████╗  ██║██║  ███╗
   ██╔══╝  ██║██║   ██║
   ██║     ██║╚██████╔╝
   ╚═╝     ╚═╝ ╚═════╝  ....is now installed!


EOF


## you can also use <<-'EOF' to strip tab character from start of each line
cat <<EOF 
   Hey! Welcome to ${MAGENTA}${BOLD}Fig${NORMAL}.

   This quick walkthrough will show you how Fig works.


   Stuck? Type ${BOLD}help${NORMAL}. 
   Want to quit? Hit ${BOLD}ctrl + c${NORMAL}

EOF

fig hook event "Started Shell Onboarding" 2>&1 1>/dev/null
press_enter_to_continue

clear

cat <<EOF
   
   ${BOLD}${MAGENTA}Fig${NORMAL} suggests commands, options, and arguments as you type.

   ${BOLD}Autocomplete Basics${NORMAL}

     * To filter: just start typing
     * To navigate: use the ${BOLD}↓${NORMAL} & ${BOLD}↑${NORMAL} arrow keys
     * To select: hit ${BOLD}enter${NORMAL} or ${BOLD}tab${NORMAL}
     * To hide: press ${BOLD}esc${NORMAL}, or scroll ${BOLD}↑${NORMAL} past the top suggestion to shell history

EOF

press_enter_to_continue
clear

(fig hook init $$ $TTY 2>&1 1>/dev/null)
cat <<EOF

   ${BOLD}Example${NORMAL}
   Try typing ${BOLD}cd${NORMAL} then space. Autocomplete will suggest the folders in your
   home directory.

   
   ${BOLD}To Continue...${NORMAL}
   cd into the "${BOLD}.fig/${NORMAL}" folder

EOF

prepare_prompt

while true; do
  input=""

  read -e -p "$DEFAULT_PROMPT" input
  echo $END_CMD # New line after output
  reset_prompt
  case "${input}" in
    cd*)
      cd ~/.fig
      print_special "${BOLD}Awesome!${NORMAL}"
      echo
      print_special "${UNDERLINE}Quick Tip${NORMAL}: Selecting a suggestion with a ${BOLD}🟥 red icon${NORMAL} and ${BOLD}↪${NORMAL} symbol 
              will immediately execute a command"
      press_enter_to_continue
      break
      ;;
    "continue") break ;;
    "c") break ;;
    "") print_special "Type ${BOLD}cd .fig/${NORMAL} to continue" ;;
    help|HELP|--help|-h)
      show_help
      print_special "Type ${BOLD}cd .fig/${NORMAL} to continue"
      ;;
    *)
      print_special "${YELLOW}Whoops. Looks like you tried something other than cd."
      print_special "Type ${BOLD}cd .fig/${NORMAL} to continue"
      ;;
  esac
done

(fig hook init $$ $TTY 2>&1 1>/dev/null)
clear 
cat <<EOF

   ${BOLD}Another Example${NORMAL}
   Fig can insert text and move your cursor around.

   ${BOLD}To Continue...${NORMAL}

   Run ${BOLD}git commit -m 'hello'${NORMAL}

   
   (Don't worry, this will ${BOLD}not${NORMAL} actually run the git command)

EOF

prepare_prompt
while true; do
  input=""
  read -e -p "$DEFAULT_PROMPT" input
  printf $END_CMD
  echo # New line after output
  case "${input}" in
    "git commit"*)
      reset_prompt
      print_special "${BOLD}Nice work!${NORMAL}"
      press_enter_to_continue
      reset_prompt
      break
      ;;
    "continue") break ;;
    "c") break ;;
    "")
      print_special "Try running ${BOLD}git commit -m 'hello'${NORMAL} to continue. Otherwise, just type ${BOLD}continue"
      ;;
    help|HELP|--help|-h)
      show_help
      print_special "Try running ${BOLD}git commit -m 'hello'${NORMAL} to continue. Otherwise, just type ${BOLD}continue"
      ;;
    *)
      print_special "${YELLOW}Whoops. Looks like you tried something other than ${BOLD}git commit${NORMAL}."
      print_special "Try running ${BOLD}git commit -m 'hello'${NORMAL} to continue. Otherwise, just type ${BOLD}continue"
      ;;
  esac
done

clear 

(fig hook init $$ $TTY 2>&1 1>/dev/null)
cat <<EOF
   
   ${BOLD}Last Step: The ${MAGENTA}Fig${NORMAL} ${BOLD}CLI${NORMAL}

   fig              your home for everything Fig
   fig doctor       check if Fig is properly configured
   fig settings     update preferences (keybindings, UI, and more)
   fig tweet        share your terminal set up with the world!
   fig update       check for updates
   fig --help       a summary of Fig commands with examples


   ${BOLD}To Continue...${NORMAL} 

   Run ${MAGENTA}${BOLD}fig${NORMAL} to see how you can customize Fig
   (You can also type ${UNDERLINE}continue${NORMAL})

EOF

prepare_prompt
while true; do
  input=""
  read -e -p "$DEFAULT_PROMPT" input
  echo # New line after output
  case "${input}" in

    "fig")
      fig > /dev/null
clear
cat <<EOF

   ${BOLD}Awesome!${NORMAL}

   You can use ${MAGENTA}${BOLD}Fig${NORMAL} to:

    * ${BOLD}Customize autocomplete${NORMAL}:
        height, width, theme, fuzzy search, keybindings, etc.

    * ${BOLD}Enable 3rd party shell plugins${NORMAL}:
        prompts, autosuggestions, themes & more

    * ${BOLD}Manage and sync your dotfiles/shell configuration${NORMAL}
EOF
      press_enter_to_continue
      break
      ;;
    "continue"*) break ;;
    "c") break ;;
    ""|help|HELP|--help|-h)
      show_help
cat <<EOF

   ${BOLD}To Continue...${NORMAL} 

   Run ${MAGENTA}${BOLD}fig${NORMAL}
   (You can also type ${UNDERLINE}continue${NORMAL})

EOF
      ;;
    *)
      print_special "${YELLOW}Whoops. Looks like you tried something unexpected."
cat <<EOF

   ${BOLD}To Continue...${NORMAL} 

   Run ${MAGENTA}${BOLD}fig${NORMAL}
   (You can also type ${UNDERLINE}continue${NORMAL})

EOF
      ;;
  esac
done

clear 
cat <<EOF

   ${BOLD}Want to share Fig?${NORMAL}
   
      Run ${MAGENTA}${BOLD}fig tweet${NORMAL} or ${MAGENTA}${BOLD}fig invite${NORMAL} (you get 5 invites!)


   ${BOLD}Want to contribute?${NORMAL}

      * Check out our docs: ${UNDERLINE}fig.io/docs/getting-started${NORMAL}
      * Submit a pull request: ${UNDERLINE}github.com/withfig/autocomplete${NORMAL}

EOF

# Tell use how to open urls based on terminal type
# https://superuser.com/questions/683962/how-to-identify-the-terminal-from-a-script
if [[ "${TERM_PROGRAM}" == "iTerm.app" ]]; then
  echo "   ${UNDERLINE}Hint${NORMAL}: Hold cmd + click to open URLs"
else
  echo "   ${UNDERLINE}Hint${NORMAL}: Hold cmd + double-click to open URLs"
fi
echo


# Make sure we are using OSX sed rather than GNU version
sed -i='' "s/FIG_ONBOARDING=.*/FIG_ONBOARDING=1/g" ~/.fig/user/config 2> /dev/null
fig hook event "Completed Shell Onboarding" 2>&1 1>/dev/null

echo
press_enter_to_continue 'Press enter to finish'
echo
echo

# Done using http://patorjk.com/software/taag/#p=testall&f=Graffiti&t=fig
# Font name = Ivrit
clear

cat <<EOF
   ${BOLD}Almost done!${NORMAL}

   1. You should run ${MAGENTA}${BOLD}fig doctor${NORMAL} right now. 
      This checks for common bugs and fixes them!

   2. Fig won't work in any terminal sessions you currently have running,
      only new ones. (You might want to restart your terminal emulator)

   3. FYI we've saved a backup of your dotfiles to ~/.fig.dotfiles.bak

EOF
