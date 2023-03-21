# `odilia-cache`

A caching layer for the [Odilia screen reader](https://odilia.app/).
This crate is a subdirectory of the [main Odilia repo](https://github.com/odilia-app/odilia).

This crate's primary structure is:

```rust
Arc<DashMap<AccessibleId, Arc<RwLock<CacheItem>>>>
```

Various parts of this structure help with different features provided by this crate.
The `Arc<DashMap<_>>` allow the cache to be used across thread.
And the value of the map being `Arc<RwLock<CacheItem>>` allows us to reference other cache items directly, as well as being able to look them up by ID in the hashmap.

## Hacking

If you are looking to make Odilia a more performant screen reader, this is the place.
Please feel free to work with us on implementing tests, and improving benchmarks.
Or, [suggest ideas on Github](https://github.com/odilia-app/odilia/issues/new) if you'd like to suggest ideas before implementing them.

We can also help you around the code base if you need a gentle introduction.

## Performance

There are some benchmarks written for Odilia.
Most of them are related to the speed of referencing various items from the cache.

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

