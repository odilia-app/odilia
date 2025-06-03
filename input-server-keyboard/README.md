# `odilia-input-server-keyboard`

Control the Odilia screen reader with your keyboard.
For security reasons, this is a separate process that communicates with Odilia via a Unix socket.

## Running Tests

When you run the tests for this crate, you can use `cargo test` to run the basic tests.

For the `proptest`-based tests, enable the `proptest` feature flag.
It is highly recommended to also enable `--release` mode, otherwise these tests take a _very_ long time to run.

There are additional things you may want to set, like `PROPTEST_MAX_SHRINK_ITERS` (to get small reproducable examples).
Here is what we use in our dev environment:

```bash
PROPTEST_MAX_SHRINK_ITERS=10000 PROPTEST_CASES=1000 cargo test --release --all-features
```
