use crate::tower::Handler;
use odilia_common::command::OdiliaCommand as Command;
use odilia_common::errors::OdiliaError;
use std::future::Future;

type Request = Command;
type Response = ();
type Error = OdiliaError;

