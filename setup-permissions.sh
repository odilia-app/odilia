#!/bin/sh

set -eu

# Setup permissions to have uinput 
RULES_PATH='/etc/udev/rules.d/99-odilia-sr.rules'
MODULES_PATH='/etc/modules-load.d/uinput.conf'
ODILIA_GROUP='odilia'
RULES_TEXT="KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"$ODILIA_GROUP\", OPTIONS+=\"static_node=uinput\""
## END of config

# get user (even if they're in sudo)
user=$(logname)

# get all users
groups=$(cat /etc/group | cut -d: -f1)
# if odilia user doesn't exist, make it, add currect user to it, and add current user to input group.
# 	techincally, we should only need to do one of these, but some distros do not have automatic `input` group permissions.
groupadd "$ODILIA_GROUP"
usermod -a -G "$ODILIA_GROUP" $user
usermod -a -G "input" $user
echo "Warning: Although your permissions are set up correctly, you need to log out to apply these changes."

# get text of file we should have written to
rules_text=""
if [ -f "$RULES_PATH" ]; then
	rules_text=$(cat "$RULES_PATH")
fi
# if module is not already written to, then
if [ ! -f "$MODULES_PATH" ]; then
	echo "uinput" | tee "$MODULES_PATH"
fi
# if the rule is not up to date, then replace it and reload udev
if [[ "$rules_text" != "$RULES_TEXT" ]]; then
	echo "$RULES_TEXT" | tee "$RULES_PATH"
	udevadm control --reload-rules
	echo "udev rules updated...you may need to reset to see these changes applied."
else
	echo "udev rules correct"
fi

