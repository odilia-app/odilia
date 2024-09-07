# Contributing

Before submitting a pull request, please make sure your code passes the following checks locally:

- `cargo test` passes without any errors
- `cargo fmt` has properly formatted all files
- `cargo clippy` has been run on all files without any errors or warnings in pedantic mode

These can be added to your pre-commit hooks to automate the checks. Beyond these checks, it is recommended to develop with standard Rust tooling like rust-analyzer. Once your code is passing locally, you can submit a pull request and a maintainer can pass it through the continuous integration checks.

Besides this, we do not have any specific contribution guidelines or codes of conduct for now, however most likely these will be fleshed out as Odilia matures more.

## How To Add A Feature

Odilia has a unique architecture which embeds alot of code into types.
Here is what you need to know if you want to add a new feature:

- [ ] Find the AT-SPI event that corresponds to the desired feature
    - Take a look at the docs for `common/events/` and try to find the right event.
    - For example, to create a feature that reads out changed text in an aria-live region, you would want the event `TextChangedEvent`.
- [ ] Decide if there are any pre-requisites to the feature being triggerwd (think: focused window only, open tab only, etc.)
- [ ] See if the prerequisite is already defined
    - Check out the `odilia/src/tower/cache_event.rs` file for types that implement the `Predicate` trait (defined in the `refinement` create).
- [ ] Decide what, if any, state is required for your feature (caret position, last focused item, etc.)
    - For example: to know if the current window is focused, the current window must be stored in the state of the screen reader.
- [ ] Check to see if we already have your state info as a type; you can check this at `odilia/src/state.rs`.
    - If not, make a [newtype](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction), add it to the `State` struct, then implement the `FromAsyncState` trait for the type (so it can be extracted by the `FromAsyncState` implementation).
- [ ] Implement a new async function that takes `PreRequisiteType<EventType>`, and `StateType` (if necessary). That function can return a list of `Command`s that Odilia will act on.
    - Then, add it to the list of `.atspi_listener(fn_name)` calls in `main`.
    - The list of possible `Command`s can be found in the type `OdiliaCommand` enum in `common/src/commands.rs`.
- [ ] If a new `Command` is required, create a newtype, implement the `IntoCommands` trait, add it as a variant to the enum, then finally implement the `CommandType` trait.
    - To add funcionality to this command, create an `async fn` that takes `Command(NewCommandType)` and `NewStateType` (if necessary). Finally, add it to the list of `.command_listener(fn_name)` calls in `main`.

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
```

Now, you can run the following commands to produce flamegraphes for individual benchmarks with the following command:

```bash
cargo flamegraph --bench load_test -- --bench [individual_bench_name]
```
