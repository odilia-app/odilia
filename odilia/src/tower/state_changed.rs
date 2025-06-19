use std::marker::PhantomData;

use atspi::{
	events::{
		object::StateChangedEvent, DBusInterface, DBusMatchRule, DBusMember,
		DBusProperties, MessageConversion, RegistryEventString,
	},
	AtspiError, Event, EventProperties, EventTypeProperties, State as AtspiState,
};
use zbus::{names::UniqueName, zvariant::ObjectPath};

use crate::tower::Predicate;

pub type Focused = StateChanged<StateFocused, True>;

#[derive(Debug, Default, Clone)]
pub struct StateChanged<S, E> {
	ev: StateChangedEvent,
	_marker: PhantomData<(S, E)>,
}
impl<S, E> From<StateChanged<S, E>> for Event {
	fn from(sc: StateChanged<S, E>) -> Event {
		sc.ev.into()
	}
}
impl<S, E> EventProperties for StateChanged<S, E> {
	fn sender(&self) -> UniqueName<'_> {
		self.ev.sender()
	}
	fn path(&self) -> ObjectPath<'_> {
		self.ev.path()
	}
}
impl<S, E> EventTypeProperties for StateChanged<S, E> {
	fn member(&self) -> &'static str {
		self.ev.member()
	}
	fn interface(&self) -> &'static str {
		self.ev.interface()
	}
	fn match_rule(&self) -> &'static str {
		self.ev.match_rule()
	}
	fn registry_string(&self) -> &'static str {
		self.ev.registry_string()
	}
}

impl<S, E> DBusMember for StateChanged<S, E>
where
	StateChanged<S, E>: TryFrom<StateChangedEvent>,
{
	const DBUS_MEMBER: &'static str = StateChangedEvent::DBUS_MEMBER;
}
impl<S, E> DBusInterface for StateChanged<S, E>
where
	StateChanged<S, E>: TryFrom<StateChangedEvent>,
{
	const DBUS_INTERFACE: &'static str = StateChangedEvent::DBUS_INTERFACE;
}
impl<S, E> DBusMatchRule for StateChanged<S, E>
where
	StateChanged<S, E>: TryFrom<StateChangedEvent>,
{
	const MATCH_RULE_STRING: &'static str = StateChangedEvent::MATCH_RULE_STRING;
}
impl<S, E> RegistryEventString for StateChanged<S, E>
where
	StateChanged<S, E>: TryFrom<StateChangedEvent>,
{
	const REGISTRY_EVENT_STRING: &'static str = StateChangedEvent::REGISTRY_EVENT_STRING;
}

impl<S, E> DBusProperties for StateChanged<S, E> where StateChanged<S, E>: TryFrom<StateChangedEvent>
{}

impl<'b, S, E> MessageConversion<'b> for StateChanged<S, E>
where
	StateChanged<S, E>: TryFrom<StateChangedEvent>,
{
	type Body<'msg>
		= <StateChangedEvent as MessageConversion<'b>>::Body<'msg>
	where
		Self: 'msg;
	fn from_message_unchecked(
		msg: &zbus::Message,
		header: &zbus::message::Header,
	) -> Result<Self, AtspiError>
	where
		Self: Sized + 'b,
	{
		Self::from_message_unchecked_parts(header.try_into()?, msg.body())
	}
	fn from_message_unchecked_parts(
		or: atspi::ObjectRef,
		bdy: zbus::message::Body,
	) -> Result<Self, AtspiError> {
		let ev = StateChangedEvent::from_message_unchecked_parts(or, bdy)?;
		// TODO: we do not have an appropriate event type here; this should really be an OdiliaError.
		// We may want to consider adding a type Error in the BusProperties impl.
		Self::try_from(ev).map_err(|_| AtspiError::InterfaceMatch(String::new()))
	}
	fn body(&self) -> Self::Body<'_> {
		self.ev.body()
	}
}

impl<S, E> TryFrom<atspi::Event> for StateChanged<S, E>
where
	S: Predicate<AtspiState>,
	E: Predicate<bool>,
{
	type Error = crate::OdiliaError;
	fn try_from(ev: atspi::Event) -> Result<Self, Self::Error> {
		let state_changed_ev: StateChangedEvent = ev.try_into()?;
		StateChanged::<S, E>::try_from(state_changed_ev)
	}
}

impl<S, E> TryFrom<StateChangedEvent> for StateChanged<S, E>
where
	S: Predicate<AtspiState>,
	E: Predicate<bool>,
{
	type Error = crate::OdiliaError;
	fn try_from(ev: StateChangedEvent) -> Result<Self, Self::Error> {
		if <Self as Predicate<StateChangedEvent>>::test(&ev) {
			Ok(Self { ev, _marker: PhantomData })
		} else {
			Err(crate::OdiliaError::PredicateFailure(format!("The type {ev:?} is not compatible with the predicate requirements state = {:?} and enabled = {:?}", std::any::type_name::<S>(), std::any::type_name::<E>())))
		}
	}
}

impl<S, E> Predicate<StateChangedEvent> for StateChanged<S, E>
where
	S: Predicate<AtspiState>,
	E: Predicate<bool>,
{
	fn test(ev: &StateChangedEvent) -> bool {
		<S as Predicate<AtspiState>>::test(&ev.state)
			&& <E as Predicate<bool>>::test(&ev.enabled)
	}
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct True;
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct False;

impl Predicate<bool> for True {
	fn test(b: &bool) -> bool {
		*b
	}
}
impl Predicate<bool> for False {
	fn test(b: &bool) -> bool {
		!*b
	}
}

macro_rules! impl_refinement_type {
	($enum:ty, $variant:expr, $name:ident) => {
		#[allow(unused)]
		#[derive(Debug, Clone, Copy)]
		pub struct $name;
		impl Predicate<$enum> for $name {
			fn test(outer: &$enum) -> bool {
				&$variant == outer
			}
		}
	};
}
#[allow(unused)]
pub struct AnyState;

impl Predicate<AtspiState> for AnyState {
	#[allow(clippy::too_many_lines)]
	fn test(outer: &AtspiState) -> bool {
		match *outer {
			AtspiState::Invalid => <StateInvalid as Predicate<AtspiState>>::test(outer),
			AtspiState::Active => <StateActive as Predicate<AtspiState>>::test(outer),
			AtspiState::Armed => <StateArmed as Predicate<AtspiState>>::test(outer),
			AtspiState::Busy => <StateBusy as Predicate<AtspiState>>::test(outer),
			AtspiState::Checked => <StateChecked as Predicate<AtspiState>>::test(outer),
			AtspiState::Collapsed => {
				<StateCollapsed as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Defunct => <StateDefunct as Predicate<AtspiState>>::test(outer),
			AtspiState::Editable => {
				<StateEditable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Enabled => <StateEnabled as Predicate<AtspiState>>::test(outer),
			AtspiState::Expandable => {
				<StateExpandable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Expanded => {
				<StateExpanded as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Focusable => {
				<StateFocusable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Focused => <StateFocused as Predicate<AtspiState>>::test(outer),
			AtspiState::HasTooltip => {
				<StateHasTooltip as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Horizontal => {
				<StateHorizontal as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Iconified => {
				<StateIconified as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Modal => <StateModal as Predicate<AtspiState>>::test(outer),
			AtspiState::MultiLine => {
				<StateMultiLine as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Multiselectable => {
				<StateMultiselectable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Opaque => <StateOpaque as Predicate<AtspiState>>::test(outer),
			AtspiState::Pressed => <StatePressed as Predicate<AtspiState>>::test(outer),
			AtspiState::Resizable => {
				<StateResizable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Selectable => {
				<StateSelectable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Selected => {
				<StateSelected as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Sensitive => {
				<StateSensitive as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Showing => <StateShowing as Predicate<AtspiState>>::test(outer),
			AtspiState::SingleLine => {
				<StateSingleLine as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Stale => <StateStale as Predicate<AtspiState>>::test(outer),
			AtspiState::Transient => {
				<StateTransient as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Vertical => {
				<StateVertical as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Visible => <StateVisible as Predicate<AtspiState>>::test(outer),
			AtspiState::ManagesDescendants => {
				<StateManagesDescendants as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Indeterminate => {
				<StateIndeterminate as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Required => {
				<StateRequired as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Truncated => {
				<StateTruncated as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Animated => {
				<StateAnimated as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::InvalidEntry => {
				<StateInvalidEntry as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::SupportsAutocompletion => {
				<StateSupportsAutocompletion as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::SelectableText => {
				<StateSelectableText as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::IsDefault => {
				<StateIsDefault as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Visited => <StateVisited as Predicate<AtspiState>>::test(outer),
			AtspiState::Checkable => {
				<StateCheckable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::HasPopup => {
				<StateHasPopup as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::ReadOnly => {
				<StateReadOnly as Predicate<AtspiState>>::test(outer)
			}
			_ => todo!(),
		}
	}
}

impl_refinement_type!(AtspiState, AtspiState::Invalid, StateInvalid);
impl_refinement_type!(AtspiState, AtspiState::Active, StateActive);
impl_refinement_type!(AtspiState, AtspiState::Armed, StateArmed);
impl_refinement_type!(AtspiState, AtspiState::Busy, StateBusy);
impl_refinement_type!(AtspiState, AtspiState::Checked, StateChecked);
impl_refinement_type!(AtspiState, AtspiState::Collapsed, StateCollapsed);
impl_refinement_type!(AtspiState, AtspiState::Defunct, StateDefunct);
impl_refinement_type!(AtspiState, AtspiState::Editable, StateEditable);
impl_refinement_type!(AtspiState, AtspiState::Enabled, StateEnabled);
impl_refinement_type!(AtspiState, AtspiState::Expandable, StateExpandable);
impl_refinement_type!(AtspiState, AtspiState::Expanded, StateExpanded);
impl_refinement_type!(AtspiState, AtspiState::Focusable, StateFocusable);
impl_refinement_type!(AtspiState, AtspiState::Focused, StateFocused);
impl_refinement_type!(AtspiState, AtspiState::HasTooltip, StateHasTooltip);
impl_refinement_type!(AtspiState, AtspiState::Horizontal, StateHorizontal);
impl_refinement_type!(AtspiState, AtspiState::Iconified, StateIconified);
impl_refinement_type!(AtspiState, AtspiState::Modal, StateModal);
impl_refinement_type!(AtspiState, AtspiState::MultiLine, StateMultiLine);
impl_refinement_type!(AtspiState, AtspiState::Multiselectable, StateMultiselectable);
impl_refinement_type!(AtspiState, AtspiState::Opaque, StateOpaque);
impl_refinement_type!(AtspiState, AtspiState::Pressed, StatePressed);
impl_refinement_type!(AtspiState, AtspiState::Resizable, StateResizable);
impl_refinement_type!(AtspiState, AtspiState::Selectable, StateSelectable);
impl_refinement_type!(AtspiState, AtspiState::Selected, StateSelected);
impl_refinement_type!(AtspiState, AtspiState::Sensitive, StateSensitive);
impl_refinement_type!(AtspiState, AtspiState::Showing, StateShowing);
impl_refinement_type!(AtspiState, AtspiState::SingleLine, StateSingleLine);
impl_refinement_type!(AtspiState, AtspiState::Stale, StateStale);
impl_refinement_type!(AtspiState, AtspiState::Transient, StateTransient);
impl_refinement_type!(AtspiState, AtspiState::Vertical, StateVertical);
impl_refinement_type!(AtspiState, AtspiState::Visible, StateVisible);
impl_refinement_type!(AtspiState, AtspiState::ManagesDescendants, StateManagesDescendants);
impl_refinement_type!(AtspiState, AtspiState::Indeterminate, StateIndeterminate);
impl_refinement_type!(AtspiState, AtspiState::Required, StateRequired);
impl_refinement_type!(AtspiState, AtspiState::Truncated, StateTruncated);
impl_refinement_type!(AtspiState, AtspiState::Animated, StateAnimated);
impl_refinement_type!(AtspiState, AtspiState::InvalidEntry, StateInvalidEntry);
impl_refinement_type!(AtspiState, AtspiState::SupportsAutocompletion, StateSupportsAutocompletion);
impl_refinement_type!(AtspiState, AtspiState::SelectableText, StateSelectableText);
impl_refinement_type!(AtspiState, AtspiState::IsDefault, StateIsDefault);
impl_refinement_type!(AtspiState, AtspiState::Visited, StateVisited);
impl_refinement_type!(AtspiState, AtspiState::Checkable, StateCheckable);
impl_refinement_type!(AtspiState, AtspiState::HasPopup, StateHasPopup);
impl_refinement_type!(AtspiState, AtspiState::ReadOnly, StateReadOnly);
