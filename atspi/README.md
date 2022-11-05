# AT-SPI for Rust

[![crates.io badge](https://img.shields.io/crates/v/atspi)](https://crates.io/crates/atspi)
[![docs.rs badge](https://docs.rs/atspi/badge.svg)](https://docs.rs/atspi)

Higher level, asynchronous Rust bindings to [AT-SPI2](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/), using
[zbus](https://crates.io/crates/zbus).

Part of the [Odilia screen reader project](https://odilia.app).

## Design

* Fully documented, with `#[deny(missing_docs)]`
	* Or at least, it will be by 1.0
* Fully safe, with `#[deny(unsafe_code)]`
* Fantastic code style with `#[deny(clippy:all, clippy::pedantic, clippy::cargo)]`

These bindings are currently general purpose, and can be used in any project. While we intend to try keeping it that
way, the design of this crate will likely be affected by the design of Odilia, particularly by it's addon API, which
requires that addons from different languages be able to access objects from this crate.

This crate makes use of the [zbus crate](https://crates.io/crates/zbus) for [dbus
communication](https://www.freedesktop.org/wiki/Software/dbus/). We use the asynchronous zbus API, so to use atspi, you
will need to run an async executer like [tokio](https://crates.io/crates/tokio) or
[async-std](https://crates.io/crates/async-std). The `async-io` and `tokio` features are exposed and will be passed
through to zbus.

## Usage

Add this to `Cargo.toml`:

```toml
[dependencies]
atspi = "0.2.1"
```

## Contributing

We love people who add functionality, find bugs, or improve code quality!
You can clone the repository and make modifications just by `git clone`-ing the repository like so:

```bash
$ git clone https://github.com/odilia-app/odilia
$ cd odilia/atspi
$ cargo build
```

At this time, you need to download our entire `odilia` repository to make modifications to the `atspi` crate.
This will be changed in the future.

## License

The `atspi` library is licensed as [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0.html) or [MIT](https://mit-license.org/).
