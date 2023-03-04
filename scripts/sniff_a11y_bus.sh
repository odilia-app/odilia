#!/bin/bash
addr="$(./scripts/get_a11y_bus_address.sh)"
dbus-monitor --address $addr
