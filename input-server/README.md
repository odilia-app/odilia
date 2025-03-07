# `odilia-input-server`

THis input server is designed to work with the [Odilia screen reader](https://odilia.app/).
It acts as a separate process to Odilia to adhere to the [principle of least priveledge](https://en.wikipedia.org/wiki/Principle_of_least_privilege).
It supports a limited set of key combinations listed below:

## Key Binding Types

### Technical Details

Prefixed here refers to an activation key prefix, usually CapsLock.
Modality refers to whether the key is modal (is only usable in a particular mode).

- Prefixed capturing:
	- `capslock + h` to go to next heading
- Unprefixed capturing:
	- `h` to go to next heading
- Conditional capturing:
	- _Not available_
	- In order to do this correctly, we need the ability to re-emit a backlog of events upon deciding whether an event is not a valid combo.
	- Imagine you want the control key to be captured if it is immediately released (say, for example, to stop speech). But you still want to be able to use system combos like control+c, control+v, control+p, etc.
	- This would require us to store backlogged state, then re-emit the events once further information was found.
	- This requires:
		1. Additional privileges
		2. Has potential performance and input validation concerns.
		3. Has caused us a variety of issues in the past. [1](https://github.com/odilia-app/odilia/issues/20)[2](https://github.com/odilia-app/odilia/issues/63)
	- So, we plan on avoiding this if at all possible.
- Key-repeating combos:
	- `capslock (press+release) times 2` for toggling capslock
	- This is not permitted for a variety of reasons:
	- 1: With the right ambiguous situation (`capslock` action 1, `capslock (x2)` for openning settings), it causes unavoidable latency by at least the minimum waiting time for repeat key to be pressed. [More than a couple dozen milliseconds is noticable to users](https://dl.acm.org/doi/fullHtml/10.1145/3678299.3678331)
	- 2: Passthrough would be impossible. Imagine now you want to use `capslock (x2)` to toggle capslock. This would require either knowing the future, or re-emitting the pressed and released events upon release (in case `capslock + f` were also a valid combo).
	- This will not be supported for the forseeable future.
	- That said, we may be interested in adding "held combos", which do not have these same problems. For example "capslock (hold)" to activate capslock.
		- This has the advantage of not introducing time-dependent delays, it just:
		- Waits for a delta to be elapsed since first press, if released, reset it.
		- If delta elapsed, pass through the press, ignore all other press events of the key until a release event.
		- When the relase event comes along, pass it through too.
		- Since held keys are passed through upon release as well, You've created a way to pass through capslock.
		- This is theoretical, and would require a lot of testing to ensure it does not break user expectations.
- Unprefixed passthrough:
	- `ctrl` for stopping speech
	- This is possible because control is _not captured_ by the input server.
	- This allows the input daemon to keep less state, and not require the ability to re-emit keys.

