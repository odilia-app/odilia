# Contributing

Before submitting a pull request, please make sure your code passes the following checks locally:

- `cargo test` passes without any errors
- `cargo fmt` has properly formatted all files
- `cargo clippy` has been run on all files without any errors or warnings in pedantic mode

These can be added to your pre-commit hooks to automate the checks. Beyond these checks, it is recommended to develop with standard Rust tooling like rust-analyzer. Once your code is passing locally, you can submit a pull request and a maintainer can pass it through the continuous integration checks.

Besides this, we do not have any specific contribution guidelines or codes of conduct for now, however most likely these will be fleshed out as Odilia matures more.

## How To Add A Feature

Odilia has a unique architecture which embeds a lot of code into types.
Here is what you need to know if you want to add a new feature:

### Pre-requisite Knowledge

- [Rust Traits](https://doc.rust-lang.org/beta/book/ch10-02-traits.html)
- [Command pattern](https://en.wikipedia.org/wiki/Command_pattern)
- [Axum's extractor model](https://docs.rs/axum/latest/axum/extract/struct.State.html)
- [newtype](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)

### Odilia

- Odilia uses our own traits to implement something similar to the Axum extractor model.
- We use something similar to the command pattern to separate the behaviour of a command (speak text, change braille display, navigate to a new heading, change state, etc.) from its context (i.e., within a specific function).
- These pattern combined allow us to extract _only the state required to run an individual function_ and _compare output of the function without actually running the commands_. This creates code that is easier to test, debug, and log than passing a huge state struct around to any function that needs it; and it also allows us to run tests without mocking a _real GUI_.
- The most important trait is: `TryFromState`, which defines a fallible conversion from Odilia's `ScreenReaderState` and one additional type (for now, either an `atspi` Event, or a `CommandType`, TODO: input events coming soon).

Here are the steps you will need to create a new feature in Odilia:

- [ ] Find the AT-SPI event that corresponds to the desired feature
    - NOTE: although we don't yet support input events, this will be added in the future.
    - Take a look at the docs for `common/events/` in the `atspi` crate to find the right event.
    - For example, to create a feature that reads out changed text in an aria-live region, you would want the event `TextChangedEvent`.
- [ ] Decide if there are any prerequisites to the feature being triggered (think: focused window only, open tab only, etc.)
- [ ] See if the prerequisite is already defined
    - Check out the `odilia/src/tower/extractors/cache_event.rs` file for types that implement the `Predicate` trait (defined in the `refinement` create).
    - If a new predicate is required, make a newtype that implements your required predicate.
- [ ] Decide what, if any, state is required for your feature (caret position, last focused item, etc.)
    - For example: to know if the current window is focused, the current window must be stored in the state of the screen reader.
- [ ] Check to see if we already have your state info as a type; you can check this at `odilia/src/tower/extractors/*.rs`.
		- If not, make a newtype, add it to the `State` struct, then implement the `TryFromState` trait for the type (so it can be extracted by the `TryFromState` implementation).
		- An example of how to implement `TryFromState` can be found in `odilia/src/tower/extractors/event_property.rs` (this one is generic over any event property)
- [ ] Implement a new async function that takes `PreRequisiteType<EventType>`, and any other extractors (if necessary). That function can return a list of `Command`s that Odilia will act on.
    - Then, add it to the list of `.atspi_listener(fn_name)` calls in `main`.
    - The list of possible `Command`s can be found in the type `OdiliaCommand` enum in `common/src/commands.rs`.
- [ ] If a new `Command` is required, create a newtype, implement the `IntoCommands` trait, add it as a variant to the enum, then finally implement the `CommandType` trait.
    - To add functionality to this command, create an `async fn` that takes `Command(NewCommandType)` and any other extractor types (if necessary). Finally, add it to the list of `.command_listener(fn_name)` calls in `main`.

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
