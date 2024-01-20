use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyError {
    #[error("connection or monitor related error")]
    Dbus(#[from] zbus::Error),
    #[error("zbus specification defined error")]
    DbusSpec(#[from] zbus::fdo::Error),
}
