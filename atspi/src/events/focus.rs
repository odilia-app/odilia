use zbus::dbus_proxy;

#[dbus_proxy(interface = "org.a11y.atspi.Event.Focus")]
trait Focus {
    /// Focus signal
    #[dbus_proxy(signal)]
    fn focus(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;
}
