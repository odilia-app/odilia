# odilia-common

Common algorithms and data structures shared by multiple Odilia crates.

Part of the [Odilia screen reader project](https://odilia.app).

## WASM Compatibiliy

This crate is compileable to wasm *if* no feature flags are specified.
The `zbus` feature flag is there only to introduce some conversion functions from `zbus::Error` into `OdiliaError` and does not change any part of the data structure.
We recommend the `zbus` feature flag on the host, i.e., when developing the Odilia screen reader, but is not recommended for addons.

## Contributing

[Reach out to us on Github](https://github.com/odilia-app/odilia/issues/new), even if it's just to ask us to walk you through an issue, how to solve it, or where you may be able to start if you'd like to fix an issue.
We always appreciate PRs and issue submissions.

## License

All our code is licensed under the [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html).
