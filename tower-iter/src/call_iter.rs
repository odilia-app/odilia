use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Poll, Context};
use core::future::Future;
use core::future::ready;
use core::future::Ready;
use tower::Service;
use tower::ServiceExt as OtherServiceExt;
use futures::future::TryFutureExt;
use futures::future::FutureExt;
use futures::future::join;
use futures::future::{Join, Then};

impl<S, Req, Res, E, F> FullServiceFut<S, Req, Res, E, F> {
    fn new(s: S, req: Req) -> Self {
        FullServiceFut::ServiceNotReady(ReadyFutTake::new(s, req))
    }
}

#[pin_project::pin_project(project = PinnedFsf)]
pub enum FullServiceFut<S, Req, Res, E, F>{
    ServiceNotReady(#[pin]ReadyFutTake<S, Req>),
    ServicePending(#[pin]ServiceCall<F, Res, E>)
}
impl<S, Req, Res, E, F> Future for FullServiceFut<S, Req, Res, E, F> 
where F: Future<Output = Result<Res, E>>,
S: Service<Req, Response = Res, Future = F, Error = E> {
    type Output = F::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let next = match self.as_mut().project() {
            PinnedFsf::ServiceNotReady(ready_fut) => {
                match ready_fut.poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready((Err(e), req)) => return Poll::Ready(Err(e)),
                    Poll::Ready((Ok(svc), req)) => Self::ServicePending(ServiceCall::new(svc, req)),
                }
            },
            PinnedFsf::ServicePending(svc_fut) => {
                return svc_fut.poll(cx);
            },
        };
        Pin::set(&mut self, next);
        Poll::Pending
    }
}

pub trait ServiceExt<I>: Service<I> {
    fn service_call<Res, E, F>(self, input: I) -> FullServiceFut<Self, I, Res, E, F>
    where Self: Sized + Service<I, Response = Res, Error = E, Future = F> {
        FullServiceFut::new(self, input)
    }
}
impl<S, I> ServiceExt<I> for S
where S: Service<I> {}

#[pin_project::pin_project]
pub struct ReadyFut<S, I> {
    inner: Option<S>,
    _marker: PhantomData<I>,
}
impl<S, I> ReadyFut<S, I> {
    pub fn new(s: S) -> Self {
        ReadyFut {
            inner: Some(s),
            _marker: PhantomData,
        }
    }
}
impl<S, I> Future for ReadyFut<S, I> 
where S: Service<I> {
    type Output = Result<S, S::Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let Some(ref mut inner) = self.inner else {
            // SAFETY: This is allowed beacuase futures that are polled after a return of
            // Poll::Ready may panic; quote from [`std::future::Future`]
            //
            // Once a future has completed (returned Ready from poll), calling its poll method again may panic, block forever, or cause other kinds of problems; the Future trait places no requirements on the effects of such a call. However, as the poll method is not marked unsafe, Rust’s usual rules apply: calls must never cause undefined behavior (memory corruption, incorrect use of unsafe functions, or the like), regardless of the future’s state.
            panic!("You can not poll this function when the `inner` Service value is none!");
        };
        match inner.poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Ready(Ok(())) => Poll::Ready(Ok(
                // SAFETY: THis will never pnaic, because [`self.inner`] can not be constructed
                // without a value; `inner` is private.
                core::mem::take(&mut self.inner)
                    .unwrap()
            )),
        }
    }
}
#[pin_project::pin_project]
pub struct ReadyFutTake<S, I> {
    inner: Option<(S, I)>
}
impl<S, I> ReadyFutTake<S, I> {
    pub fn new(s: S, i: I) -> Self {
        ReadyFutTake {
            inner: Some((s, i))
        }
    }
}
impl<S, I> Future for ReadyFutTake<S, I> 
where S: Service<I> {
    type Output = (Result<S, S::Error>, I);
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let Some((ref mut inner, _)) = self.inner else {
            // SAFETY: These can only be none if this is polled after returning Poll::Ready;  quote from [`std::future::Future`]:
            //
            // Once a future has completed (returned Ready from poll), calling its poll method again may panic, block forever, or cause other kinds of problems; the Future trait places no requirements on the effects of such a call. However, as the poll method is not marked unsafe, Rust’s usual rules apply: calls must never cause undefined behavior (memory corruption, incorrect use of unsafe functions, or the like), regardless of the future’s state.
            panic!("You can not poll this function when the `inner` Service value is none!");
        };
        match inner.poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => {
                // SAFETY: These can only be none if this is polled after returning Poll::Ready.
                let Some((_owned_svc, owned_input)) = core::mem::take(&mut self.inner) else {
                    panic!("The items can not be none!");
                };
                return Poll::Ready((Err(e), owned_input));
            },
            Poll::Ready(Ok(())) => {
                // SAFETY: These can only be none if this is polled after returning Poll::Ready.
                let Some((owned_svc, owned_input)) = core::mem::take(&mut self.inner) else {
                    panic!("The items can not be none!");
                };
                return Poll::Ready((Ok(owned_svc), owned_input));
            },
        }
    }
}

pub struct MapServiceCall<Iter, S, I> {
    inner: Iter,
    _marker: PhantomData<(S, I)>,
}
impl<Iter, S, I> Iterator for MapServiceCall<Iter, S, I> 
where Iter: Iterator<Item = (S, I)>,
S: Service<I> {
    type Item = FullServiceFut<S, I, S::Response, S::Error, S::Future>;
    fn next(&mut self) -> Option<Self::Item> {
        let (svc, input) = self.inner.next()?;
        Some(FullServiceFut::new(svc, input))
    }
}

pub struct MapReady<Iter, S, I> {
    inner: Iter,
    _marker: PhantomData<(S, I)>,
}
impl<Iter, S, I> Iterator for MapReady<Iter, S, I>
where S: Service<I>,
Iter: Iterator<Item = S>{
    type Item = ReadyFut<S, I>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut s: S = self.inner.next()?;
        Some(ReadyFut::new(s))
    }
}

impl<F, Res, E> ServiceCall<F, Res, E> {
    pub fn new<S, Req>(mut s: S, req: Req) -> Self 
    where S: Service<Req, Future = F> {
        ServiceCall {
            f: s.call(req),
            _marker: PhantomData,
        }
    }
}

#[pin_project::pin_project]
pub struct ServiceCall<F, Res, E> {
#[pin]
    f: F,
    _marker: PhantomData<Result<Res, E>>,
}
impl<F, Res, E> Future for ServiceCall<F, Res, E> 
where F: Future<Output = Result<Res, E>> {
    type Output = Result<Res, E>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().f.poll(cx)
    }
}

pub trait MapMExt: Iterator + Sized {
  fn map_service_call<S, I>(self) -> MapServiceCall<Self, S, I> {
      MapServiceCall { inner: self, _marker: PhantomData }
  }
}
impl<I> MapMExt for I where I: Iterator + Sized {}

pub struct MapM<Iter, S, I, O> {
	inner: Iter,
	_marker: PhantomData<fn(S, I) -> O>,
}

impl<Iter, S, I, O> Iterator for MapM<Iter, S, I, O>
where
	Iter: Iterator<Item = (S, I)>,
	S: Service<I, Response = O> + ServiceExt<I>,
{
	type Item = S::Future;
	fn next(&mut self) -> Option<Self::Item> {
		let (mut s, i) = self.inner.next()?;
		Some(
        s.call(i)
    )
	}
}
