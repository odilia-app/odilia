# Odilia Screen Reader

[![Build CI](https://github.com/odilia-app/odilia/actions/workflows/ci.yml/badge.svg)](https://github.com/odilia-app/odilia/actions)
[![codecov](https://codecov.io/gh/odilia-app/odilia/branch/main/graph/badge.svg?token=BM4SQ9BLK4)](https://codecov.io/gh/odilia-app/odilia)

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

## Running

Simply type `odilia` in your terminal!

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

### Performance Benchmarking

If you'd like detailed performance benchmarks, we recommend using the `flamegraph` package to show performance bottlenecks.
There is also `hotspot`, a C++ program available in the AUR, and some major package repos, which can display how much time is spent in various portions of the program in an accessible (GUI) way.

First, install the subcommand with:

```bash
$ cargo install flamegraph
```

If needed, install Hotspot from the AUR/your package repo, as well as `perf` which is required to produce the flame graph.

```bash
$ paru/yay -S hotspot perf
```

Finally, add the following to the root `Cargo.toml`:

```toml
[profile.bench]
debug = true
```

Now, you can run the following commands to produce flamegraphes for individual benchmarks with the following command:

```bash
cargo flamegraph --bench load_test -- --bench [individual_bench_name]
```

## License

The Odilia screen reader is licensed under the [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html).
