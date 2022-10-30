# Odilia Screen Reader

<!-- Todo: Add badges here -->

## Welcome to Odilia!

Accessibility on Linux has historically been under-developed, under-maintained, and therefore not up to modern standards
for many blind people. We're hear to help with that!

Odilia is a new screen reader for the Linux desktop. It's written in [Rust](https://rust-lang.org), for maximum
performance and stability.

## Status: Alpha

This is **absolutely not stable or production ready in any way!** Everything is in a very early stage and we're breaking
things on a daily basis!

## Building

To build odilia:

```sh
git clone https://github.com/odilia-app/odilia
cd odilia
cargo build --release
# At this point the compiled program is at ./target/release/odilia
# Optionally, run this to install Odilia to ~/.cargo/bin:
cargo install --path .
./setup-permissions.sh
```

### Udev Permissions

Odilia uses the Linux kernel's [evdev interface](https://freedesktop.org/software/libevdev/doc/latest/) to listen for
and redirect events from input devices, such as your keyboard and mouse.

Evdev is normally a privileged interface, since any application that can access it could use it for malicious purposes,
for example, creating a keylogger. For this reason, to run Odilia, you must give yourself access to evdev. This can be
done by running the [setup-permissions.sh shell
script](https://github.com/odilia-app/odilia/blob/main/setup-permissions.sh) included with Odilia. The script adds some
udev rules, then creates an odilia group. Any users added to this group and the `input` group will be able to run
Odilia.

## Contributing

We are excited to accept new contributions to this project; in fact, we already have!
Sometimes there may be missing documentation or lack of examples.
Please, reach out to us, [make an issue](https://github.com/odilia-app/odilia), or a [pull request](https://github.com/odilia-app/odilia/pulls) and we will continue to improve Odilia with your help.
Thank you to all who have contibuted so far, and who will continue to contribute in the future!

We do not have any specific contribution guidelines or code of conducts.
These will be fleshed out as Odilia matures.

## License

All our code is licensed under the [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html).
THe only exception is the `atspi` library, which is licensed as [LGPL v3](https://www.gnu.org/licenses/lgpl-3.0.html).
