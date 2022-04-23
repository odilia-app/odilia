use zbus::dbus_proxy;

#[dbus_proxy(interface = "org.a11y.atspi.Event.Document")]
trait Document {
    /// AttributesChanged signal
    #[dbus_proxy(signal)]
    fn attributes_changed(
        &self,
        arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>),
    ) -> zbus::Result<()>;

    /// ContentChanged signal
    #[dbus_proxy(signal)]
    fn content_changed(
        &self,
        arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>),
    ) -> zbus::Result<()>;

    /// LoadComplete signal
    #[dbus_proxy(signal)]
    fn load_complete(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>))
        -> zbus::Result<()>;

    /// LoadStopped signal
    #[dbus_proxy(signal)]
    fn load_stopped(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// PageChanged signal
    #[dbus_proxy(signal)]
    fn page_changed(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Reload signal
    #[dbus_proxy(signal)]
    fn reload(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;
}
