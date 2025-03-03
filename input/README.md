# odilia-input

Input subsystem for the Odilia screen reader.

Part of the [Odilia screen reader project](https://odilia.app).

## Design

This crate currently only opens a socket and accepts updates via JSON.
The design allows anybody to plug into Odilia using their input method.
Although Odilia will eventually get native keyboard, mouse, and touchscreen support, most features can currently be activated directly using this socket mechanism.
For an example of what you may be able to send over the socket, take a look at the `exmaples/` directory.

The socket file will either be placed at: `$XDG_RUNTIME_HOME/odilia/odilia.sock`, or `/run/user/$UID/odilia/odilia.sock`.

## Contributing

Please [create an issue on our Github](https://github.com/odilia-app/odilia/issues/new),
or contribute directly by cloing [our repository](https://github.com/odilia-app/odilia), then [opening a PR via Github](https://github.com/odilia-app/odilia/compare).

## License

All our code is licensed under the [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html).
