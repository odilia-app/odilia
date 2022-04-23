use zbus::dbus_proxy;

#[dbus_proxy(interface = "org.a11y.atspi.Event.Mouse")]
trait Mouse {
    /// Abs signal
    #[dbus_proxy(signal)]
    fn abs(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Button signal
    #[dbus_proxy(signal)]
    fn button(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Rel signal
    #[dbus_proxy(signal)]
    fn rel(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;
}
