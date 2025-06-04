#!/bin/bash
set -euo pipefail
xvfb-run dunst --screen 0 600x400x8 &

# Start DBus session and grab env
#export DBUS_SESSION_BUS_ADDRESS="$(dbus-daemon --session --print-address --fork)"

# Run the actual test binary (which Cargo/cross mounts & invokes)
exec "$@"

