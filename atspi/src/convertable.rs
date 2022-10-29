use crate::{
    accessible::AccessibleProxy, action::ActionProxy, application::ApplicationProxy, cache::CacheProxy,
    collection::CollectionProxy, component::ComponentProxy,
    device_event_controller::DeviceEventControllerProxy,
    device_event_listener::DeviceEventListenerProxy, document::DocumentProxy,
    editable_text::EditableTextProxy, hyperlink::HyperlinkProxy, hypertext::HypertextProxy,
    image::ImageProxy, registry::RegistryProxy, selection::SelectionProxy, table::TableProxy,
    table_cell::TableCellProxy, text::TextProxy, value::ValueProxy, Interface,
};
use async_trait::async_trait;
use zbus::{CacheProperties, Error};

enum Interfaces {
    Accessible,
    Action,
    Application,
    Collection,
    Component,
    Document,
    Hypertext,
    Hyperlink,
    Image,
    Selection,
    Table,
    TableCell,
    Text,
    EditableText,
    Cache,
    Value,
    Registry,
    DeviceEventController,
    DeviceEventListener,
}
impl TryFrom<&str> for Interfaces {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "org.a11y.atspi.Accessible" => Ok(Interfaces::Accessible),
            "org.a11y.atspi.Action" => Ok(Interfaces::Action),
            "org.a11y.atspi.Application" => Ok(Interfaces::Application),
            "org.a11y.atspi.Collection" => Ok(Interfaces::Collection),
            "org.a11y.atspi.Component" => Ok(Interfaces::Component),
            "org.a11y.atspi.Document" => Ok(Interfaces::Document),
            "org.a11y.atspi.Hypertext" => Ok(Interfaces::Hypertext),
            "org.a11y.atspi.Hyperlink" => Ok(Interfaces::Hyperlink),
            "org.a11y.atspi.Image" => Ok(Interfaces::Image),
            "org.a11y.atspi.Selection" => Ok(Interfaces::Selection),
            "org.a11y.atspi.Table" => Ok(Interfaces::Table),
            "org.a11y.atspi.TableCell" => Ok(Interfaces::TableCell),
            "org.a11y.atspi.Text" => Ok(Interfaces::Text),
            "org.a11y.atspi.EditableText" => Ok(Interfaces::EditableText),
            "org.a11y.atspi.Cache" => Ok(Interfaces::Cache),
            "org.a11y.atspi.Value" => Ok(Interfaces::Value),
            "org.a11y.atspi.Registry" => Ok(Interfaces::Registry),
            "org.a11y.atspi.DeviceEventController" => Ok(Interfaces::DeviceEventController),
            "org.a11y.atspi.DeviceEventListener" => Ok(Interfaces::DeviceEventListener),
            _ => Err("No interface found for conversion."),
        }
    }
}
impl ToString for Interfaces {
    fn to_string(&self) -> String {
        match self {
            Interfaces::Accessible => "org.a11y.atspi.Accessible",
            Interfaces::Action => "org.a11y.atspi.Action",
            Interfaces::Application => "org.a11y.atspi.Application",
            Interfaces::Collection => "org.a11y.atspi.Collection",
            Interfaces::Component => "org.a11y.atspi.Component",
            Interfaces::Document => "org.a11y.atspi.Document",
            Interfaces::Hypertext => "org.a11y.atspi.Hypertext",
            Interfaces::Hyperlink => "org.a11y.atspi.Hyperlink",
            Interfaces::Image => "org.a11y.atspi.Image",
            Interfaces::Selection => "org.a11y.atspi.Selection",
            Interfaces::Table => "org.a11y.atspi.Table",
            Interfaces::TableCell => "org.a11y.atspi.TableCell",
            Interfaces::Text => "org.a11y.atspi.Text",
            Interfaces::EditableText => "org.a11y.atspi.EditableText",
            Interfaces::Cache => "org.a11y.atspi.Cache",
            Interfaces::Value => "org.a11y.atspi.Value",
            Interfaces::Registry => "org.a11y.atspi.Registry",
            Interfaces::DeviceEventController => "org.a11y.atspi.DeviceEventController",
            Interfaces::DeviceEventListener => "org.a11y.atspi.DeviceEventListener",
        }
        .to_string()
    }
}

#[async_trait]
pub trait Convertable {
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>>;
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>>;
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>>;
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>>;
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>>;
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>>;
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>>;
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>>;
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>>;
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>>;
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>>;
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>>;
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>>;
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>>;
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>>;
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>>;
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>>;
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>>;
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>>;
}

/* REST OF FILE IS ALL GENERATED (kinda) */

#[async_trait]
impl Convertable for AccessibleProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for ApplicationProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for CacheProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for CollectionProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for ComponentProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for DeviceEventControllerProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for DeviceEventListenerProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for DocumentProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for EditableTextProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for HyperlinkProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for HypertextProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for ImageProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for TextProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for TableProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for TableCellProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
#[async_trait]
impl Convertable for ValueProxy<'_> {
    /* no guard due to assumption it is always possible */
    async fn to_accessible<'a>(&'a self) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.connection())
            .destination(self.destination())?
				.cache_properties(CacheProperties::No)
            .path(self.path())?
            .build()
            .await
    }
    async fn to_action<'a>(&'a self) -> zbus::Result<ActionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::Action)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ActionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_application<'a>(&'a self) -> zbus::Result<ApplicationProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Application) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ApplicationProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_collection<'a>(&'a self) -> zbus::Result<CollectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Collection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CollectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_component<'a>(&'a self) -> zbus::Result<ComponentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Component) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ComponentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_document<'a>(&'a self) -> zbus::Result<DocumentProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Document) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DocumentProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hypertext<'a>(&'a self) -> zbus::Result<HypertextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hypertext) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HypertextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_hyperlink<'a>(&'a self) -> zbus::Result<HyperlinkProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Hyperlink) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return HyperlinkProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_image<'a>(&'a self) -> zbus::Result<ImageProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Image) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ImageProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_selection<'a>(&'a self) -> zbus::Result<SelectionProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Selection) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return SelectionProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table<'a>(&'a self) -> zbus::Result<TableProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Table) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_table_cell<'a>(&'a self) -> zbus::Result<TableCellProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::TableCell) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TableCellProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_text<'a>(&'a self) -> zbus::Result<TextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Text) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return TextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_editable_text<'a>(&'a self) -> zbus::Result<EditableTextProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::EditableText)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return EditableTextProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_cache<'a>(&'a self) -> zbus::Result<CacheProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Cache) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return CacheProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_value<'a>(&'a self) -> zbus::Result<ValueProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Value) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return ValueProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_registry<'a>(&'a self) -> zbus::Result<RegistryProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc.get_interfaces().await?.contains(Interface::Registry) {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return RegistryProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_controller<'a>(
        &'a self,
    ) -> zbus::Result<DeviceEventControllerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventController)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventControllerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
    async fn to_device_event_listener<'a>(&'a self) -> zbus::Result<DeviceEventListenerProxy<'a>> {
        let acc = self.to_accessible().await?;
        if acc
            .get_interfaces()
            .await?
            .contains(Interface::DeviceEventListener)
        {
            // you can use self here since converting to accessible does not change the internal
            // variables
            return DeviceEventListenerProxy::builder(self.connection())
                .destination(self.destination())?
				.cache_properties(CacheProperties::No)
                .path(self.path())?
                .build()
                .await;
        }
        Err(Error::InterfaceNotFound)
    }
}
