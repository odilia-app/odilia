# odilia-input

Input subsystem for the Odilia screen reader.

Part of the [Odilia screen reader project](https://odilia.app).

## Design

This crate uses the [rdev crate](https://crates.io/crates/rdev), a Rust wrapper around the [Linux / BSD evdev
interface](https://freedesktop.org/software/libevdev/doc/latest/). It spawns a new thread which monitors for input
events, then sends them to the main program using an asynchronous [Tokio::sync::mpsc
channel](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html).

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

This is a very young project, we appreciate any and all contributions! However, please be aware there is a very llarge
learning curve to helping with this project, particularly due to the lack of documentation, or **complete**
documentation, of many of the libraries and technologies that comprise the Linux accessibility stack. For this reason,
we are currently focused on learning as much as we can, and writing code to take advantage of it, and we don't have lots
of time to mentor new contributors or review pull requests.

Once the ground-work has been layed, accepting contributions should get much easier. We are greatful for your
cooperation in this regard!

## License

All our code is licensed under the [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html).
