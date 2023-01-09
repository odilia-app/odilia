pkttyagent -p $(echo $$) | pkexec env XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR  ~/.cargo/bin/sohkd --debug --config /etc/odilia/sohkdrc
