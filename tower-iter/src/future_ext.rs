use core::{
  future::Future,
	marker::PhantomData,
	pin::Pin,
	task::{Context, Poll},
};
use tower::Service;
use alloc::vec::Vec;
use futures::future::{JoinAll, join_all};
use pin_project::pin_project;

pub trait FutureExt<O, E>: Future<Output = O> {
  fn ok_join_all<Iter, I>(self) -> OkJoinAll<Self, E, O, Iter, I> 
  where Self: Sized,
        I: Future  {
      OkJoinAll { f: self, res: None, _marker: PhantomData }
  }
	fn wrap_ok(self) -> MapOk<Self, E, O>
	where
		Self: Sized,
	{
		MapOk { f: self, _marker: PhantomData }
	}
}
#[pin_project]
pub struct OkJoinAll<F, E, O, Iter, I> 
where I: Future {
	#[pin]
	f: F,
#[pin]
  res: Option<JoinAll<I>>,
	_marker: PhantomData<(O, E, Iter, I)>,
}
impl<F, E, O, Iter, I> Future for OkJoinAll<F, E, O, Iter, I>
where
	F: Future<Output = Result<Iter, E>>,
  Iter: Iterator<Item = I>,
  I: Future<Output = O>
{
	type Output = Result<Vec<O>, E>;
	fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let mut this = self.as_mut().project();
    if let Some(res) = this.res.as_mut().as_pin_mut() {
        res.poll(cx).map(Ok)
    } else {
      let res = match this.f.poll(cx) {
        Poll::Ready(Ok(o)) => o,
        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
        Poll::Pending => return Poll::Pending,
      };
      let x = Some(join_all(res));
      *this.res = x;
      self.poll(cx)
    }
	}
}

#[pin_project]
pub struct MapOk<F, E, O> {
	#[pin]
	f: F,
	_marker: PhantomData<(O, E)>,
}
impl<F, E, O> Future for MapOk<F, E, O>
where
	F: Future<Output = O>,
{
	type Output = Result<O, E>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let this = self.project();
		match this.f.poll(cx) {
			Poll::Ready(o) => Poll::Ready(Ok(o)),
			Poll::Pending => Poll::Pending,
		}
	}
}

impl<F, O, E> FutureExt<O, E> for F where F: Future<Output = O> {}
