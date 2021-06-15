controlPath="$1"
user="$2"
host="$3"
mkdir -p /tmp/fig_pty
# Copy all files we update during install.
# TODO(sean) optimize and slim down number of files here to bare min for efficiency.
copy() {
  scp -o PasswordAuthentication=no -o "ControlPath=($controlPath)" ~/.bashrc $user@$host:bashrc
}

copy $user@$host:~/\{.zshrc,.bashrc,.profile,.zprofile,.bash_profile,.bashrc\} /tmp/fig_pty
~/.fig_pty/install.sh /tmp/fig_pty
copy ~/.fig_pty $user@$host:~
copy /tmp/fig_pty/{.zshrc,.bashrc,.profile.zprofile,.bash_profile,.bashrc} $user@$host:~
