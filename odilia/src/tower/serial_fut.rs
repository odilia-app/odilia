use std::{
    pin::Pin,
    task::{Poll, Context},
    future::Future,
};
use futures_lite::FutureExt;
use futures::future::MaybeDone;
use tower::{
    Service,
    util::BoxCloneService,
};

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
					_ => {
						continue;
					}
				},
				_ => {
					continue;
				}
			}
		}
		let result = self
			.inner
			.as_mut()
			.get_mut()
			.iter_mut()
			.map(|f| Pin::new(f))
			.map(|e| e.take_output().unwrap())
			.collect();
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
		loop {
			if let Some(s) = this.inner.into_iter().next() {
				match s.call(rc.clone()).poll(cx) {
					Poll::Pending => return Poll::Pending,
					Poll::Ready(result) => {
						this.results.push(result);
					}
				}
			} else {
				break;
			}
		}
		return Poll::Ready(Ok(this.results.to_vec()));
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

