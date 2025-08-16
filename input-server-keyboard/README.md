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

## How Does It Work?

- We capture all kernel input events via special permissions given to users from `evdev`.
- Generally, we pass through all events, unless one of the following cases are triggered:
    1. The global activation key is pressed. This key, set to capslock by default, enables most of Odilia's commands. Any key pressed with the global activation key will be captured and not re-emitted.
    2. Part of a key combination is pressed that _could_ lead to an event being triggered. This means that if you have an event like `Shift+h`, then `Shift` will _never_ be passed through to the application as it might be tha start of a key combo.
    3. 2) is performed only if the requested mode is active.
- We understand that \#2 is quite limiting, but we have some ways around this. For example, if you wanted to have `Ctrl+Shift+h` to be the action "move left and select the previous word", this could still be done without disruption to the user by making it only active in certain modes.
- For example, you might only activate this combination in browse mode, since most editors will enable this with `Ctrl+Shift+Left Arrow` in any edit box.
- I believe that this type of user interaction (replaying keys when they end up not matching a potential binding) are holding back screen reader users, because it inherently adds latency to their interaction with the computer. It also massivley expands the scope of what needs to be handled by the keyboard process.

