# AT-SPI for Rust

[![crates.io badge](http://meritbadge.herokuapp.com/atspi)](https://crates.io/crates/atspi)
[![docs.rs badge](https//docs.rs/atspi/badge.svg)](https://docs.rs/atspi)

Higher level, asynchronous Rust bindings to [AT-SPI2](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/), using
[zbus][https://crates.io/crates/zbus].

Part of the [Odilia screen reader project](https://odilia.app).

## Design

These bindings are currently general purpose, and can be used in any project. While we intend to try keeping it that
way, the design of this crate will likely be affected by the design of Odilia, particularly by it's addon API, which
requires that addons from different languages be able to access objects from this crate.

This crate makes use of the [zbus crate](https://crates.io/crates/zbus) for [dbus
communication](https://www.freedesktop.org/wiki/Software/dbus/). We use the asynchronous zbus API, so to use atspi, you
will need to run an async executer like [tokio](https://crates.io/crates/tokio) or
[async-std](https://crates.io/crates/async-std). The async-io` and `tokio` features are exposed and will be passed
through to zbus.

## Usage

Add this to `Cargo.toml`:

```toml
[dependencies]
atspi = "0.0.1"
```

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
