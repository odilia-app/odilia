# SRaaS (Screen Reader as a Service)

This document describes in some moderate detail both _why_ we chose to use `tower` (the Rust service-based architecture) for the infrastructure of Odilia, and also _how_ to contribute to various parts of the system.

## Why

Think about screen readers, what do they even do?

Well, for the most part, they receive events from the system (user input, accessibility events), and then produce output for the system (speak text, update braille display, synthesize user input events).
All of these are things which _should_ be performed asyncronously (i.e., concurrently) since it is all based around IO.

Since `tower` is a generic service/layer system for dealing with these kinds of asyncronous handling at various levels, let's explore the current architecture of Odilia and what kind of services we have created to deal with the unique challenges that a screen reader has to face.
TODO

```
Current tree of services:
TryIntoCommands is a trait which means it can be converted into Result<Vec<Command>, Error>

TryInto(Event) -> Result<E, Error>
    CacheLayer(E) -> (E, Arc<Cache>)
        AsyncTryInto(E, Arc<Cache>) -> CacheEvent<E>
            Handler(CacheEvent<E>, State):
               fn(CacheEvnet<E>, ...impl async TryFromState) 
            -> O: TryIntoCommand
                for cmd in O -> Resul<Vec<Command>, Result>:
                    run_command(cmd)

Desired tree:

TryInto(Event) -> Result<E, Error>
    StateLayer(E), -> (E, Arc<State>)
        AsyncTryInto(E, Arc<State>) -> (CacheEvemt<E>, ...impl async TryFromState)
            Handler(CacheEvemt<E>, ...impl async TryFromState) -> O: TryIntoCommands
                for cmd in O::try_into_commands():
                    run_command(cmd)

This is more generic since it can convert _any_ type which needs state, including the cache, or any other part, for whatever reason.
This also makes it more complicated because `CacheEvent<E>` needs to actually pass `E` and state into a conversion function to work, whereas some types can be converted without passing any additional information into the state for lookup.

It gets a little complicated here with variable argument lists. Let's hope we can find the solution.
```

## TODOs

- [ ] Document all the various services and layers and why they are there.
