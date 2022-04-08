use std::ops::Deref;

use zbus::Address;

use crate::{bus::BusProxy, registry::RegistryProxy};

/// A connection to the at-spi bus
pub struct Connection {
    registry: RegistryProxy<'static>,
}

impl Connection {
    /// Open a new connection to the bus
    pub async fn open() -> zbus::Result<Self> {
        // Grab the a11y bus address from the session bus
        let a11y_bus_addr = {
        let session_bus = zbus::Connection::session().await?;
        let proxy = BusProxy::new(&session_bus).await?;
        proxy.get_address().await?.into()
    };

        // Connect to the a11y bus
        let bus = zbus::ConnectionBuilder::address(Address::Unix(a11y_bus_addr))?.build().await?;
        // The Proxy holds a strong reference to a Connection, so we only need to store the proxy
        let registry = RegistryProxy::new(&bus).await?;

        Ok(Self { registry })
    }
}

impl Deref for Connection {
    type Target = RegistryProxy<'static>;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}
