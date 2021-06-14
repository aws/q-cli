mv $HOME/.ssh/config $HOME/.ssh/old
cp $HOME/.fig_pty/ssh_config $HOME/.ssh/config

rm -rf ~/.fig_pty
cp -r $PWD/config/ $HOME/.fig_pty

make
cp $PWD/fig_pty $HOME/.fig_pty/bin/

~/.fig_pty/install.sh
