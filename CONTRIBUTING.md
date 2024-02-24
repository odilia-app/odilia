# Contributing

Before submitting a pull request, please make sure your code passes the following checks locally:

- `cargo test` passes without any errors
- `cargo fmt` has properly formatted all files
- `cargo clippy` has been run on all files without any errors or warnings in pedantic mode

These can be added to your pre-commit hooks to automate the checks. Beyond these checks, it is recommended to develop with standard Rust tooling like rust-analyzer. Once your code is passing locally, you can submit a pull request and a maintainer can pass it through the continuous integration checks.

Besides this, we do not have any specific contribution guidelines or codes of conduct for now, however most likely these will be fleshed out as Odilia matures more.

## Performance Benchmarking

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
``

Now, you can run the following commands to produce flamegraphes for individual benchmarks with the following command:

```bash
cargo flamegraph --bench load_test -- --bench [individual_bench_name]
```
