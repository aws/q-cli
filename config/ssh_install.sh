host="$1"
mkdir -p /tmp/fig_pty
scp -q $host:~/{.zshrc,.bashrc,.profile,.zprofile,.bash_profile,.bashrc} /tmp/fig_pty
~/.fig_pty/install.sh /tmp/fig_pty
scp -qr ~/.fig_pty $host:~
scp -qr /tmp/fig_pty/{.zshrc,.bashrc,.profile.zprofile,.bash_profile,.bashrc} $host:~
