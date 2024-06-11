use futures::future::MaybeDone;
use futures_lite::FutureExt;
use std::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};
use tower::{util::BoxCloneService, Service};

/// `SerialFuture` is a way to link a variable number of dependent futures.
/// You can race!() two `SerialFuture`s, which will cause the two, non-dependent chains of futures to poll concurrently, while completing the individual serial futures, well, serially.
///
/// Why not just use `.await`? like so:
///
/// ```rust,norun
/// return (fut1.await, fut2.await, ...);
/// ```
///
/// Because you may have a variable number of functions that need to run in series.
/// Think of handlers for an event, if you want the results to be deterministic, you will need to run the event listeners in series, even if multiple events could come in and each trigger its own set of listeners to execute syhncronously, the set of all even listeners can technically be run concorrently.
pub struct SerialFutures<F>
where
	F: Future,
{
	// TODO: look into MaybeDone
	inner: Pin<Box<[MaybeDone<F>]>>,
}
impl<F> Unpin for SerialFutures<F> where F: Future {}

impl<F> Future for SerialFutures<F>
where
	F: futures::TryFuture + Unpin,
{
	type Output = Result<Vec<F::Output>, F::Error>;
	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		for mfut in self.inner.as_mut().get_mut() {
			match mfut {
				MaybeDone::Future(fut) => match fut.poll(cx) {
					Poll::Pending => return Poll::Pending,
					Poll::Ready(_) => {
						continue;
					}
				},
				MaybeDone::Done(_) => {
					continue;
				}
				MaybeDone::Gone => {
					panic!("The value produced in this serial future has disappeared before it was taken by the result. This should never happen!");
				}
			}
		}
		let result: Vec<_> = self
			.inner
			.as_mut()
			.get_mut()
			.iter_mut()
			.map(Pin::new)
			// all will now be in their done state due to above checks
			.filter_map(|e| e.take_output())
			.collect();
		debug_assert_eq!(
			self.inner.len(),
			result.len(),
			"The results are a differentl;y sized array to the [Box<MaybeDone>]!"
		);
		Poll::Ready(Ok(result))
	}
}

pub struct SerialHandlers<I, O, E> {
	inner: Vec<BoxCloneService<I, O, E>>,
}

#[pin_project::pin_project]
pub struct SerialServiceFuture<I, O, E> {
	req: I,
	inner: Vec<BoxCloneService<I, O, E>>,
	results: Vec<Result<O, E>>,
}

impl<I, O, E> Future for SerialServiceFuture<I, O, E>
where
	I: Clone,
	O: Clone,
	E: Clone,
	// Assuming YourInnerType implements a call function.
{
	type Output = Result<Vec<Result<O, E>>, E>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.as_mut().project();
		let rc = this.req.clone();
		for s in &mut *this.inner {
			match s.call(rc.clone()).poll(cx) {
				Poll::Pending => return Poll::Pending,
				Poll::Ready(result) => {
					this.results.push(result);
				}
			}
		}
		Poll::Ready(Ok(this.results.clone()))
	}
}

impl<I, O, E> Service<I> for SerialHandlers<I, O, E>
where
	I: Clone + Send + Sync,
	O: Send + Clone,
	E: Send + Clone,
{
	type Response = Vec<Result<O, E>>;
	type Error = E;
	type Future = SerialServiceFuture<I, O, E>;
	fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), E>> {
		for service in &mut self.inner {
			let _ = service.poll_ready(ctx)?;
		}
		Poll::Ready(Ok(()))
	}

	fn call(&mut self, req: I) -> Self::Future {
		let len = self.inner.len();
		let ic = self.inner.clone();
		let inner = std::mem::replace(&mut self.inner, ic);
		SerialServiceFuture { inner, req, results: Vec::with_capacity(len) }
	}
}
