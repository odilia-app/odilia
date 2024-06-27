use atspi_common::{
	events::object::StateChangedEvent, AtspiError, EventProperties, State as AtspiState,
};
use derived_deref::{Deref, DerefMut};
use refinement::Predicate;
use std::marker::PhantomData;
use zbus::{names::UniqueName, zvariant::ObjectPath};

pub type StateEnabled<S> = StateChanged<S, True>;
pub type StateDisabled<S> = StateChanged<S, False>;

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct StateChanged<S, E> {
	#[target]
	ev: StateChangedEvent,
	_marker: PhantomData<(S, E)>,
}
impl<S, E> EventProperties for StateChanged<S, E> {
	fn sender(&self) -> UniqueName<'_> {
		self.ev.sender()
	}
	fn path(&self) -> ObjectPath<'_> {
		self.ev.path()
	}
}
impl<S, E> atspi::BusProperties for StateChanged<S, E>
where
	StateChanged<S, E>: TryFrom<StateChangedEvent>,
{
	const DBUS_MEMBER: &'static str = StateChangedEvent::DBUS_MEMBER;
	const DBUS_INTERFACE: &'static str = StateChangedEvent::DBUS_INTERFACE;
	const MATCH_RULE_STRING: &'static str = StateChangedEvent::MATCH_RULE_STRING;
	const REGISTRY_EVENT_STRING: &'static str = StateChangedEvent::REGISTRY_EVENT_STRING;
	type Body = <StateChangedEvent as atspi::BusProperties>::Body;
	fn from_message_parts(or: atspi::ObjectRef, bdy: Self::Body) -> Result<Self, AtspiError> {
		let ev = StateChangedEvent::from_message_parts(or, bdy)?;
		// TODO: we do not have an appropriate event type here; this should really be an OdiliaError.
		// We may want to consider adding a type Error in the BusProperties impl.
		Self::try_from(ev).map_err(|_| AtspiError::InterfaceMatch(String::new()))
	}
	fn body(&self) -> Self::Body {
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
	fn test(outer: &AtspiState) -> bool {
		match *outer {
			AtspiState::Invalid => <Invalid as Predicate<AtspiState>>::test(outer),
			AtspiState::Active => <Active as Predicate<AtspiState>>::test(outer),
			AtspiState::Armed => <Armed as Predicate<AtspiState>>::test(outer),
			AtspiState::Busy => <Busy as Predicate<AtspiState>>::test(outer),
			AtspiState::Checked => <Checked as Predicate<AtspiState>>::test(outer),
			AtspiState::Collapsed => <Collapsed as Predicate<AtspiState>>::test(outer),
			AtspiState::Defunct => <Defunct as Predicate<AtspiState>>::test(outer),
			AtspiState::Editable => <Editable as Predicate<AtspiState>>::test(outer),
			AtspiState::Enabled => <Enabled as Predicate<AtspiState>>::test(outer),
			AtspiState::Expandable => {
				<Expandable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Expanded => <Expanded as Predicate<AtspiState>>::test(outer),
			AtspiState::Focusable => <Focusable as Predicate<AtspiState>>::test(outer),
			AtspiState::Focused => <Focused as Predicate<AtspiState>>::test(outer),
			AtspiState::HasTooltip => {
				<HasTooltip as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Horizontal => {
				<Horizontal as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Iconified => <Iconified as Predicate<AtspiState>>::test(outer),
			AtspiState::Modal => <Modal as Predicate<AtspiState>>::test(outer),
			AtspiState::MultiLine => <MultiLine as Predicate<AtspiState>>::test(outer),
			AtspiState::Multiselectable => {
				<Multiselectable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Opaque => <Opaque as Predicate<AtspiState>>::test(outer),
			AtspiState::Pressed => <Pressed as Predicate<AtspiState>>::test(outer),
			AtspiState::Resizable => <Resizable as Predicate<AtspiState>>::test(outer),
			AtspiState::Selectable => {
				<Selectable as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Selected => <Selected as Predicate<AtspiState>>::test(outer),
			AtspiState::Sensitive => <Sensitive as Predicate<AtspiState>>::test(outer),
			AtspiState::Showing => <Showing as Predicate<AtspiState>>::test(outer),
			AtspiState::SingleLine => {
				<SingleLine as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Stale => <Stale as Predicate<AtspiState>>::test(outer),
			AtspiState::Transient => <Transient as Predicate<AtspiState>>::test(outer),
			AtspiState::Vertical => <Vertical as Predicate<AtspiState>>::test(outer),
			AtspiState::Visible => <Visible as Predicate<AtspiState>>::test(outer),
			AtspiState::ManagesDescendants => {
				<ManagesDescendants as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Indeterminate => {
				<Indeterminate as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::Required => <Required as Predicate<AtspiState>>::test(outer),
			AtspiState::Truncated => <Truncated as Predicate<AtspiState>>::test(outer),
			AtspiState::Animated => <Animated as Predicate<AtspiState>>::test(outer),
			AtspiState::InvalidEntry => {
				<InvalidEntry as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::SupportsAutocompletion => {
				<SupportsAutocompletion as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::SelectableText => {
				<SelectableText as Predicate<AtspiState>>::test(outer)
			}
			AtspiState::IsDefault => <IsDefault as Predicate<AtspiState>>::test(outer),
			AtspiState::Visited => <Visited as Predicate<AtspiState>>::test(outer),
			AtspiState::Checkable => <Checkable as Predicate<AtspiState>>::test(outer),
			AtspiState::HasPopup => <HasPopup as Predicate<AtspiState>>::test(outer),
			AtspiState::ReadOnly => <ReadOnly as Predicate<AtspiState>>::test(outer),
			_ => todo!(),
		}
	}
}

impl_refinement_type!(AtspiState, AtspiState::Invalid, Invalid);
impl_refinement_type!(AtspiState, AtspiState::Active, Active);
impl_refinement_type!(AtspiState, AtspiState::Armed, Armed);
impl_refinement_type!(AtspiState, AtspiState::Busy, Busy);
impl_refinement_type!(AtspiState, AtspiState::Checked, Checked);
impl_refinement_type!(AtspiState, AtspiState::Collapsed, Collapsed);
impl_refinement_type!(AtspiState, AtspiState::Defunct, Defunct);
impl_refinement_type!(AtspiState, AtspiState::Editable, Editable);
impl_refinement_type!(AtspiState, AtspiState::Enabled, Enabled);
impl_refinement_type!(AtspiState, AtspiState::Expandable, Expandable);
impl_refinement_type!(AtspiState, AtspiState::Expanded, Expanded);
impl_refinement_type!(AtspiState, AtspiState::Focusable, Focusable);
impl_refinement_type!(AtspiState, AtspiState::Focused, Focused);
impl_refinement_type!(AtspiState, AtspiState::HasTooltip, HasTooltip);
impl_refinement_type!(AtspiState, AtspiState::Horizontal, Horizontal);
impl_refinement_type!(AtspiState, AtspiState::Iconified, Iconified);
impl_refinement_type!(AtspiState, AtspiState::Modal, Modal);
impl_refinement_type!(AtspiState, AtspiState::MultiLine, MultiLine);
impl_refinement_type!(AtspiState, AtspiState::Multiselectable, Multiselectable);
impl_refinement_type!(AtspiState, AtspiState::Opaque, Opaque);
impl_refinement_type!(AtspiState, AtspiState::Pressed, Pressed);
impl_refinement_type!(AtspiState, AtspiState::Resizable, Resizable);
impl_refinement_type!(AtspiState, AtspiState::Selectable, Selectable);
impl_refinement_type!(AtspiState, AtspiState::Selected, Selected);
impl_refinement_type!(AtspiState, AtspiState::Sensitive, Sensitive);
impl_refinement_type!(AtspiState, AtspiState::Showing, Showing);
impl_refinement_type!(AtspiState, AtspiState::SingleLine, SingleLine);
impl_refinement_type!(AtspiState, AtspiState::Stale, Stale);
impl_refinement_type!(AtspiState, AtspiState::Transient, Transient);
impl_refinement_type!(AtspiState, AtspiState::Vertical, Vertical);
impl_refinement_type!(AtspiState, AtspiState::Visible, Visible);
impl_refinement_type!(AtspiState, AtspiState::ManagesDescendants, ManagesDescendants);
impl_refinement_type!(AtspiState, AtspiState::Indeterminate, Indeterminate);
impl_refinement_type!(AtspiState, AtspiState::Required, Required);
impl_refinement_type!(AtspiState, AtspiState::Truncated, Truncated);
impl_refinement_type!(AtspiState, AtspiState::Animated, Animated);
impl_refinement_type!(AtspiState, AtspiState::InvalidEntry, InvalidEntry);
impl_refinement_type!(AtspiState, AtspiState::SupportsAutocompletion, SupportsAutocompletion);
impl_refinement_type!(AtspiState, AtspiState::SelectableText, SelectableText);
impl_refinement_type!(AtspiState, AtspiState::IsDefault, IsDefault);
impl_refinement_type!(AtspiState, AtspiState::Visited, Visited);
impl_refinement_type!(AtspiState, AtspiState::Checkable, Checkable);
impl_refinement_type!(AtspiState, AtspiState::HasPopup, HasPopup);
impl_refinement_type!(AtspiState, AtspiState::ReadOnly, ReadOnly);
