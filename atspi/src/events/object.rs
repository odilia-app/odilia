use zbus::dbus_proxy;

#[dbus_proxy(interface = "org.a11y.atspi.Event.Object")]
trait Object {
    /// ActiveDescendantChanged signal
    #[dbus_proxy(signal)]
    fn active_descendant_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// AttributesChanged signal
    #[dbus_proxy(signal)]
    fn attributes_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// BoundsChanged signal
    #[dbus_proxy(signal)]
    fn bounds_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// ChildrenChanged signal
    #[dbus_proxy(signal)]
    fn children_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// ColumnDeleted signal
    #[dbus_proxy(signal)]
    fn column_deleted(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// ColumnInserted signal
    #[dbus_proxy(signal)]
    fn column_inserted(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// ColumnReordered signal
    #[dbus_proxy(signal)]
    fn column_reordered(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// LinkSelected signal
    #[dbus_proxy(signal)]
    fn link_selected(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// ModelChanged signal
    #[dbus_proxy(signal)]
    fn model_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// PropertyChange signal
    #[dbus_proxy(signal)]
    fn property_change(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// RowDeleted signal
    #[dbus_proxy(signal)]
    fn row_deleted(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// RowInserted signal
    #[dbus_proxy(signal)]
    fn row_inserted(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// RowReordered signal
    #[dbus_proxy(signal)]
    fn row_reordered(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// SelectionChanged signal
    #[dbus_proxy(signal)]
    fn selection_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// StateChanged signal
    #[dbus_proxy(signal)]
    fn state_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// TextAttributesChanged signal
    #[dbus_proxy(signal)]
    fn text_attributes_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// TextBoundsChanged signal
    #[dbus_proxy(signal)]
    fn text_bounds_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// TextCaretMoved signal
    #[dbus_proxy(signal)]
    fn text_caret_moved(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// TextChanged signal
    #[dbus_proxy(signal)]
    fn text_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// TextSelectionChanged signal
    #[dbus_proxy(signal)]
    fn text_selection_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;

    /// VisibleDataChanged signal
    #[dbus_proxy(signal)]
    fn visible_data_changed(&self, event: super::Event<'_>) -> zbus::Result<()>;
}
