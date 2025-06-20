#!/bin/bash
set -euo pipefail
xvfb-run dunst --screen 0 600x400x8 &
#su root -c 'ydotoold -P 777 &'
ydotoold &

# Start DBus session and grab env
#export DBUS_SESSION_BUS_ADDRESS="$(dbus-daemon --session --print-address --fork)"

export RUST_BACKTRACE=1
#su root -c 'usermod -aG nogroup root'
groups
cat /etc/passwd | grep '^[a-z0-9: -]\+' --color -o
chmod 660 /dev/uniput
ls -alh /dev/input*
ls -alh /dev/uinput*
#su root -c 'chown -R 0:0 /dev/'

# Run the actual test binary (which Cargo/cross mounts & invokes)
exec "$@"

