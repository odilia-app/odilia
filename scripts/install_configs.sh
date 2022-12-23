#!/bin/bash
# create global config dir
mkdir -p /etc/odilia
cp -rv config.toml /etc/odilia
cp -rv sohkd/sohkdrc /etc/odilia
cp -rv sohkd/stop-speech.conf /etc/odilia
cp -rv sohkd/common-keys.conf /etc/odilia
cp -rv sohkd/arrow-granularities.conf /etc/odilia
# add local override directory
mkdir -p $XDG_CONFIG_DIR/odilia
