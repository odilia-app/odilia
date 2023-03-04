#!/bin/bash
# create global config dir
mkdir -p /etc/odilia
cp -rv config.toml /etc/odilia
# add local override directory
mkdir -p $XDG_CONFIG_DIR/odilia
