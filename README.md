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

## Building

To build odilia:
Copy paste the following on your command line to clone, build and install Odilia on your behalf in `~/.cargo/bin`,

```shell
git clone https://github.com/odilia-app/odilia  && \
cd odilia && \
cargo build --release && \
cargo install --path odilia
```

Odilia requires `uinput` access, the kernel's provisioning to emulate input devices. Furthermore, Odilia requires the user to be in the 'odilia' and 'input' groups.
Lastly Odilia requires appropriate `evdev` rules, For more information see also:  [Udev Permissions](Sudev-permissions).
This script will enable these on your behalf:

```shell
sudo ./scripts/setup_permissions.sh
```

This script will populate `/etc/odilia` with several configuration files.

```shell
sudo ./scripts/install_configs.sh`
```

You will also want to compile and install sohkd.
(Copy and paste the following on your command line. )

```shell
cd sohkd && \
cargo build && \
cp ../target/debug/sohkd ~/.cargo/bin/
```

### Udev Permissions

Odilia uses the Linux kernel's [evdev interface](https://freedesktop.org/software/libevdev/doc/latest/) to listen for
and redirect events from input devices, such as your keyboard and mouse.

Evdev is normally a privileged interface, since any application that can access it could use it for malicious purposes,
for example, creating a keylogger. For this reason, to run Odilia, you must give yourself access to evdev. This can be
done by running the [scripts/setup-permissions.sh shell
script](https://github.com/odilia-app/odilia/blob/main/setup-permissions.sh) included with Odilia. The script adds some
udev rules, then creates an odilia group. Any users added to this group and the `input` group will be able to run
Odilia.

## Running

To run Odilia, you should use our script.
This will ask for your password (if you have sudo permissions) and then launch both Odilia and the key daemon in quiet mode.

The following assumes you are in the workspace root. If you are still in `sohkd`:  ``shell cd .. ```
(Copy and paste the following on your command line. )

```shell
./scripts/odilia & \
./scripts/debug_start_sohkd.sh
```

## Community

You can find us at the following places:

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

We are excited to accept new contributions to this project; in fact, we already have!
Sometimes there may be missing documentation or lack of examples.
Please, reach out to us, [make an issue](https://github.com/odilia-app/odilia), or a [pull request](https://github.com/odilia-app/odilia/pulls) and we will continue to improve Odilia with your help.
Thank you to all who have contibuted so far, and who will continue to contribute in the future!

We do not have any specific contribution guidelines or codes of conduct.
These will be fleshed out as Odilia matures.

## License

The Odilia screen reader is licensed under the [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html).
