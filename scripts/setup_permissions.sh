#!/bin/sh

set -eu

# Setup permissions to have uinput 
RULES_PATH='/etc/udev/rules.d/99-odilia-sr.rules'
MODULES_PATH='/etc/modules-load.d/uinput.conf'
ODILIA_GROUP='odilia'
RULES_TEXT="KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"$ODILIA_GROUP\", OPTIONS+=\"static_node=uinput\""
## END of config

# get user (even if they're in sudo)
get_username() {
	set +u
	if [ -n "$SUDO_USER" ] ; then
		echo "$SUDO_USER"
	elif [ -n "$DOAS_USER" ] ; then
		echo "${DOAS_USER}"
	else
		id -un
	fi
	set -u
}

user="$(get_username)"

# if odilia group doesn't exist, make it, add correct user to it, and add current user to input group.
# 	technically, we should only need to do one of these, but some distros do not have automatic `input` group permissions.
grep -q "${ODILIA_GROUP}" /etc/group || groupadd "$ODILIA_GROUP"
if [ "$user" = root ]; then
	echo 'Could not determine name of nonprivileged user to add to odilia group.'
	echo 'Please run the command'
	echo "usermod -a -G ${ODILIA_GROUP},input MY_NONPRIVILEGED_USER"
	echo 'with root privileges to allow a user to use odilia.'
	echo "Then log out of that user's account, if necessary"
	echo 'And log in again for permissions to take effect.'
else
	usermod -a -G "${ODILIA_GROUP},input" "$user"
	echo "Warning: Although your permissions are set up correctly, you need to log out to apply these changes."
fi

# get text of file we should have written to
rules_text=""
if [ -f "$RULES_PATH" ]; then
	rules_text=$(cat "$RULES_PATH")
fi
# if module is not already written to, then
if [ ! -f "$MODULES_PATH" ]; then
	mkdir -p -m755 "$(dirname "${MODULES_PATH}")"
	echo "uinput" | tee "$MODULES_PATH"
fi
# if the rule is not up to date, then replace it and reload udev
if [ "$rules_text" != "$RULES_TEXT" ]; then
	mkdir -p -m755 "$(dirname "${RULES_PATH}")"
	echo "$RULES_TEXT" | tee "$RULES_PATH"
	udevadm control --reload-rules
	echo "udev rules updated...you may need to reset to see these changes applied."
else
	echo "udev rules correct"
fi

