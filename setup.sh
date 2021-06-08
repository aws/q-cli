ln -sf $PWD/config/ $HOME/.fig_pty
ln -sf $HOME/.fig_pty/ssh_config $HOME/.ssh/config

make
ln -sf $PWD/fig_pty $HOME/.fig_pty/bin/

~/.fig_pty/install.sh
