use zbus::dbus_proxy;

#[dbus_proxy(interface = "org.a11y.atspi.Event.Window")]
trait Window {
    /// Activate signal
    #[dbus_proxy(signal)]
    fn activate(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Close signal
    #[dbus_proxy(signal)]
    fn close(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Create signal
    #[dbus_proxy(signal)]
    fn create(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Deactivate signal
    #[dbus_proxy(signal)]
    fn deactivate(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// DesktopCreate signal
    #[dbus_proxy(signal)]
    fn desktop_create(
        &self,
        arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>),
    ) -> zbus::Result<()>;

    /// DesktopDestroy signal
    #[dbus_proxy(signal)]
    fn desktop_destroy(
        &self,
        arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>),
    ) -> zbus::Result<()>;

    /// Destroy signal
    #[dbus_proxy(signal)]
    fn destroy(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Lower signal
    #[dbus_proxy(signal)]
    fn lower(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Maximize signal
    #[dbus_proxy(signal)]
    fn maximize(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Minimize signal
    #[dbus_proxy(signal)]
    fn minimize(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Move signal
    #[dbus_proxy(signal)]
    fn move_(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// PropertyChange signal
    #[dbus_proxy(signal)]
    fn property_change(
        &self,
        arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>),
    ) -> zbus::Result<()>;

    /// Raise signal
    #[dbus_proxy(signal)]
    fn raise(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Reparent signal
    #[dbus_proxy(signal)]
    fn reparent(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Resize signal
    #[dbus_proxy(signal)]
    fn resize(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Restore signal
    #[dbus_proxy(signal)]
    fn restore(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Restyle signal
    #[dbus_proxy(signal)]
    fn restyle(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// Shade signal
    #[dbus_proxy(signal)]
    fn shade(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;

    /// uUshade signal
    #[dbus_proxy(signal)]
    fn u_ushade(&self, arg_1: (&str, u32, u32, zbus::zvariant::Value<'_>)) -> zbus::Result<()>;
}
