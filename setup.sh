ln -sf $PWD/config/ $HOME/.fig_pty
ln -sf $HOME/.fig_pty/ssh_config $HOME/.ssh/config

make
cp fig_pty $HOME/.fig_pty/bin/
