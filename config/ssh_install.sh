host="$1"
mkdir -p /tmp/fig_pty
# Copy all files we update during install.
# TODO(sean) optimize and slim down number of files here to bare min for efficiency.
scp -q $host:~/{.zshrc,.bashrc,.profile,.zprofile,.bash_profile,.bashrc} /tmp/fig_pty >/dev/null 2>&1
~/.fig_pty/install.sh /tmp/fig_pty
scp -qr ~/.fig_pty $host:~
scp -qr /tmp/fig_pty/{.zshrc,.bashrc,.profile.zprofile,.bash_profile,.bashrc} $host:~ >/dev/null 2>&1
