use std::{
	any::type_name,
	marker::PhantomData,
	task::{Context, Poll},
};

use atspi::Role;
use futures_util::future::{err, Either, Ready};
use tower::Service;

use crate::OdiliaError;

pub trait Predicate<T> {
	fn test(x: &T) -> bool;
}
pub struct PredicateService<S, P, T> {
	inner: S,
	predicate: P,
	_marker: PhantomData<T>,
}
impl<S, P, T> Service<T> for PredicateService<S, P, T>
where
	S: Service<T, Error = OdiliaError>,
	P: Predicate<T>,
{
	type Future = Either<S::Future, Ready<Result<S::Response, OdiliaError>>>;
	type Error = S::Error;
	type Response = S::Response;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}
	fn call(&mut self, req: T) -> Self::Future {
		if P::test(&req) {
			Either::Left(self.inner.call(req))
		} else {
			Either::Right(err(OdiliaError::PredicateFailure(
				type_name::<P>().to_string(),
			)))
		}
	}
}

/// List of all "container roles".
/// This is not to be confused with any role that can contain children, or roles without semantic
/// meaning.
/// This constant is for heuristics reasons, in order to determine whether we should speak the
/// entire contents of a subtree or not.
/// Subtrees are rather expensive (latency-wise) to compute constantly, and in some cases, we will
/// get a [`atspi::StateChangedEvent`] which tells us that a document is focused (getting an
/// entire document subtree _can potentially_ take multiple seconds).
///
/// If the role of the item is contained within this list, it is a hint to _not_ try to grab the
/// entire subtree, but rather just to say the name of the container (or sometimes the
/// description).
pub const CONTAINER_ROLES: [Role; 35] = [
	Role::Frame,
	Role::DocumentFrame,
	Role::DocumentWeb,
	Role::Dialog,
	Role::Alert,
	Role::Panel,
	Role::ScrollPane,
	Role::LayeredPane,
	Role::Viewport,
	Role::Filler,
	Role::Section,
	Role::Form,
	Role::Grouping,
	Role::PageTabList,
	Role::ToolBar,
	Role::ToolTip,
	Role::MenuBar,
	Role::Menu,
	Role::List,
	Role::Table,
	Role::Tree,
	Role::TreeTable,
	Role::Table,
	Role::Canvas,
	Role::DocumentFrame,
	Role::Paragraph,
	Role::Application,
	Role::DesktopFrame,
	Role::Header,
	Role::Footer,
	Role::Footnote,
	Role::Subscript,
	Role::Superscript,
	Role::Article,
	Role::Landmark,
];
