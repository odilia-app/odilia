# Odilia Screen Reader

[![Build CI](https://github.com/odilia-app/odilia/actions/workflows/ci.yml/badge.svg)](https://github.com/odilia-app/odilia/actions)
[![codecov](https://codecov.io/gh/odilia-app/odilia/branch/main/graph/badge.svg?token=BM4SQ9BLK4)](https://codecov.io/gh/odilia-app/odilia)

## Welcome to Odilia

Odilia is a screen reader for the Linux desktop.
It's written in [Rust](https://rust-lang.org), for maximum performance and stability.

## Status: Beta

This is **absolutely not production ready in any way!**  
Everything is in a fairly early stage and we're changing things on a daily basis.
However, Odilia is _somewhat_ useable, and will not crash randomly or cause weird behaviour in other applications.  
Try it out! See if it works for you!

## Prerequisites

The MSRV for Odilia is `1.81.0`.
**At this time, Odilia also requires nightly. This restriction will be lifted in the future.**

You will need to have `speech-dispatcher` installed and running before you can start Odilia.
To test that speech dispatcher is indeed working properly, try running this command:

```shell
spd-say "hello, world!"
```

if you heard a voice saying "hello, world!", you can proceed to installing.
Otherwise, check if sound is working on the computer in general.

## Build and install

To build odilia, copy paste the following on your command line . The following snippet will clone, build and install it for you, all at once without user interaction. The final binaries will be located in `~/.cargo/bin`

```shell
git clone https://github.com/odilia-app/odilia  && \
cd odilia && \
cargo install --path odilia
```

## Running

Simply type `odilia` in your terminal!

## Community

You can find us in the following places:

- [Discord](https://discord.gg/RVpRb9nS6K)
- IRC: irc.libera.chat
  - #odilia-dev (development)
  - #odilia-general (general)
  - #odilia-offtopic (off-topic)
- Matrix: stealthy.club
  - #odilia-dev (development)
  - #odilia-general (general)
  - #odilia-offtopic (off-topic)

## Contributing

We are excited to accept new contributions to this project; in fact, we already have! Sometimes there may be missing documentation or lack of examples. Please, reach out to us, [make an issue](https://github.com/odilia-app/odilia), or a [pull request](https://github.com/odilia-app/odilia/pulls) and we will continue to improve Odilia with your help. By the way, a huge thank you to all who have contributed so far, and who will continue to do so in the future!

See [CONTRIBUTING.md](./CONTRIBUTING.md) for more detail on how to contribute.

## License

The Odilia screen reader is licensed under the [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html).
