# Odilia Screen Reader

[![Build CI](https://github.com/odilia-app/odilia/actions/workflows/ci.yml/badge.svg)](https://github.com/odilia-app/odilia/actions)

## Welcome to Odilia

Odilia is a screen reader for the Linux desktop.
It's written in [Rust](https://rust-lang.org), for maximum performance and stability.

## Status: Beta

This is **absolutely not production ready in any way!**  
Everything is in a fairly early stage and we're changing things on a daily basis.
However, Odilia is *somewhat* useable, and will not crash randomly or cause weird behaviour in other applications.  
Try it out! See if it works for you!

## Prerequisites

Perhaps unnecessarily, but you will need to have `speech-dispatcher` installed and running before you can start Odilia.

to test that speech dispatcher is indeed working properly, try running this command:

```shell
spd-say "hello, world!"
```

if you heard a voice saying "hello, world!", you can proceed to installing. Otherwise, check if sound  is working on the computer in general, try looking for error outputs especially in the logs, then consult your distro packagers and ask them about how it's supposed to be set up and what could be wrong, perhaps you have to do something else before it's fully workingh, for example adding yourself to a speech group or other distro specific mechanisms

## build and install

To build odilia, copy paste the following on your command line . The following snippet will clone, build and install it for you, all at once without user interaction. The final binaries will be located in `~/.cargo/bin`

```shell
git clone https://github.com/odilia-app/odilia  && \
cd odilia && \
cargo build --release && \
cargo install --path odilia
```

Odilia requires `uinput` access, the kernel's provisioning to emulate input devices. Furthermore, it requires the user to be in the 'odilia' and 'input' groups, as well as appropriate `evdev` rules to be installed on the device. For more information, see also: [Udev Permissions](Sudev-permissions).
This script will set those on your behalf as well

```shell
sudo ./scripts/setup_permissions.sh
```

This script will populate `/etc/odilia` with several configuration files, which are required for the well functioning of the screenreader. You have to run it as root because it has to write to /etc, which is protected from regular user access, no other reasons are involved

```shell
sudo ./scripts/install_configs.sh
```

You will also want to compile and install sohkd.
(Copy and paste the following on your command line. )

```shell
cd sohkd && \
cargo build --release && \
cargo install --path .
```

## Running

To run Odilia, you should use our script.
This will ask for your password (if you have root permissions) and then launch both Odilia and the hotkey daemon in quiet mode.

```shell
./scripts/odilia
```

## Udev Permissions

Odilia uses the Linux kernel's [evdev interface](https://freedesktop.org/software/libevdev/doc/latest/) to listen for and redirect events from input devices, such as your keyboard and mouse.

Evdev is normally a privileged interface, since any application that can access it could use it for malicious purposes, for example, creating a keylogger. For this reason, to run Odilia, you must give yourself access to evdev. This can be done by running the [scripts/setup-permissions.sh shell
script](https://github.com/odilia-app/odilia/blob/main/setup-permissions.sh) included with Odilia. The script adds some udev rules, then creates an odilia group. Any users added to this group and the `input` group will be able to run Odilia.

## Community

You can find us in the following places:

* [Discord](https://discord.gg/RVpRb9nS6K)
* IRC: irc.libera.chat
  * #odilia-dev (development)
  * #odilia (general)
  * #odilia-offtopic (off-topic)
* Matrix: stealthy.club
  * #odilia-dev (development)
  * #odilia (general)
  * #odilia-offtopic (off-topic)

## Contributing

We are excited to accept new contributions to this project; in fact, we already have! Sometimes there may be missing documentation or lack of examples. Please, reach out to us, [make an issue](https://github.com/odilia-app/odilia), or a [pull request](https://github.com/odilia-app/odilia/pulls) and we will continue to improve Odilia with your help. By  the way, a huge thank you to all who have contributed so far, and who will continue to do so in the future!

We do not have any specific contribution guidelines or codes of conduct for now, however most likely these will be fleshed out as Odilia matures more.

## License

The Odilia screen reader is licensed under the [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html).
