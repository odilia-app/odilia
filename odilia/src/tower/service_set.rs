use std::collections::BTreeMap;
use odilia_common::command::{OdiliaCommandDiscriminants, OdiliaCommand};
use odilia_common::command::CommandTypeDynamic;
use odilia_common::errors::OdiliaError;
use tower::Service;
use tower::util::BoxCloneService;
use tower::service_fn;
use futures::future::err;
use std::future::Future;
use std::task::{Context, Poll};

/// A series of services which are executed in the order they are placed in the [`ServiceSet::new`]
/// initializer.
/// Useful when creating a set of handler functions that need to be run without concurrency.
///
/// Note that although calling the [`ServiceSet::call`] function seems to return a
/// `Result<Vec<S::Response, S::Error>, S::Error>`, the outer error is gaurenteed never to be
/// returned and can safely be unwrapped _from the caller function_.
#[derive(Clone)]
pub struct ServiceSet<S> {
    services: Vec<S>,
}
impl<S> Default for ServiceSet<S> {
    fn default() -> Self {
        ServiceSet {
            services: vec![],
        }
    }
}
impl<S> ServiceSet<S> {
    pub fn new<I: IntoIterator<Item = S>>(services: I) -> Self {
        ServiceSet {
            services: services.into_iter().collect(),
        }
    }
    pub fn push(&mut self, svc: S) {
        self.services.push(svc);
    }
}

impl<S, Req> Service<Req> for ServiceSet<S> 
where S: Service<Req> + Clone,
    Req: Clone {
    type Response = Vec<Result<S::Response, S::Error>>;
    type Error = S::Error;
    type Future = impl Future<Output = Result<Self::Response, Self::Error>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for mut svc in &mut self.services {
            let _ = svc.poll_ready(cx)?;
        }
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: Req) -> Self::Future {
        let services = self.services.clone();
        async move {
            let mut results = vec![];
            for mut svc in services {
                let result = svc.call(req.clone()).await;
                results.push(result);
            }
            Ok(results)
        }
    }
}