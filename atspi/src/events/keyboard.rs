use zbus::dbus_proxy;

#[dbus_proxy(interface = "org.a11y.atspi.Event.Keyboard")]
trait Keyboard {
    /// Modifiers signal
    #[dbus_proxy(signal)]
    fn modifiers(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;
}
