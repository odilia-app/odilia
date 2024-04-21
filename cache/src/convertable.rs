use async_trait::async_trait;
use atspi::Interface;
use atspi_proxies::{
	accessible::AccessibleProxy, action::ActionProxy, application::ApplicationProxy,
	collection::CollectionProxy, component::ComponentProxy, document::DocumentProxy,
	editable_text::EditableTextProxy, hyperlink::HyperlinkProxy, hypertext::HypertextProxy,
	image::ImageProxy, selection::SelectionProxy, table::TableProxy,
	table_cell::TableCellProxy, text::TextProxy, value::ValueProxy, AtspiProxy,
};
use std::ops::Deref;
use zbus::{
	blocking::Proxy as ProxyBlocking, blocking::ProxyBuilder as ProxyBuilderBlocking,
	CacheProperties, Error, Proxy, ProxyBuilder, ProxyDefault,
};

#[allow(clippy::module_name_repetitions)]
#[async_trait]
pub trait Convertable {
	type Error: std::error::Error;

	/// Creates an [`Self::Accessible`] from the existing accessible item.
	/// # Errors
	///
	/// This may fail based on the implementation of.
	/// Generally, it fails if the accessible item does not implement to accessible interface.
	/// This shouldn't be possible, but this function may fail for other reasons.
	/// For example, to convert a [`zbus::Proxy`] into a [`Self::Accessible`], it may fail to create the new [`atspi_proxies::accessible::AccessibleProxy`].
	async fn to_accessible(&self) -> Result<AccessibleProxy, Self::Error>;
	/// Creates an [`Self::Action`] from the existing accessible item.
	/// # Errors
	///
	/// This may fail based on the implementation.
	/// Generally, it fails if the accessible item does not implement to action interface.
	async fn to_action(&self) -> Result<ActionProxy, Self::Error>;
	/// Creates an [`Self::Application`] from the existing accessible item.
	/// # Errors
	///
	/// This may fail based on the implementation.
	/// Generally, it fails if the accessible item does not implement to application interface.
	async fn to_application(&self) -> Result<ApplicationProxy, Self::Error>;
	/// Creates an [`Collection`] from the existing accessible item.
	/// # Errors
	///
	/// This may fail based on the implementation.
	/// GenerallyProxy, it fails if the accessible item does not implement to collection interface.
	async fn to_collection(&self) -> Result<CollectionProxy, Self::Error>;
	/// Creates an [`Component`] from the existing accessible item.
	/// # Errors
	///
	/// This may fail based on the implementation.
	/// GenerallyProxy, it fails if the accessible item does not implement to component interface.
	async fn to_component(&self) -> Result<ComponentProxy, Self::Error>;
	async fn to_document(&self) -> Result<DocumentProxy, Self::Error>;
	async fn to_hypertext(&self) -> Result<HypertextProxy, Self::Error>;
	async fn to_hyperlink(&self) -> Result<HyperlinkProxy, Self::Error>;
	async fn to_image(&self) -> Result<ImageProxy, Self::Error>;
	async fn to_selection(&self) -> Result<SelectionProxy, Self::Error>;
	async fn to_table(&self) -> Result<TableProxy, Self::Error>;
	async fn to_table_cell(&self) -> Result<TableCellProxy, Self::Error>;
	async fn to_text(&self) -> Result<TextProxy, Self::Error>;
	async fn to_editable_text(&self) -> Result<EditableTextProxy, Self::Error>;
	async fn to_value(&self) -> Result<ValueProxy, Self::Error>;
}

#[inline]
async fn convert_to_new_type<
	'a,
	'b,
	T: From<Proxy<'b>> + ProxyDefault,
	U: Deref<Target = Proxy<'a>> + ProxyDefault,
>(
	from: &U,
) -> zbus::Result<T> {
	// first thing is first, we need to creat an accessible to query the interfaces.
	let accessible = AccessibleProxy::builder(from.connection())
		.destination(from.destination())?
		.cache_properties(CacheProperties::No)
		.path(from.path())?
		.build()
		.await?;
	// if the interface we're trying to convert to is not available as an interface; this can be problematic because the interface we're passing in could potentially be different from what we're converting to.
	let new_interface_name = Interface::try_from(<T as ProxyDefault>::INTERFACE)
		.map_err(|_| Error::InterfaceNotFound)?;
	if !accessible.get_interfaces().await?.contains(new_interface_name) {
		return Err(Error::InterfaceNotFound);
	}
	// otherwise, make a new Proxy with the related type.
	let path = from.path().to_owned();
	let dest = from.destination().to_owned();
	ProxyBuilder::<'b, T>::new_bare(from.connection())
		.interface(<T as ProxyDefault>::INTERFACE)?
		.destination(dest)?
		.cache_properties(CacheProperties::No)
		.path(path)?
		.build()
		.await
}

#[async_trait]
impl<'a, T: Deref<Target = Proxy<'a>> + ProxyDefault + Sync> Convertable for T {
	type Error = zbus::Error;
	/* no guard due to assumption it is always possible */
	async fn to_accessible(&self) -> zbus::Result<AccessibleProxy> {
		convert_to_new_type(self).await
	}
	async fn to_action(&self) -> zbus::Result<ActionProxy> {
		convert_to_new_type(self).await
	}
	async fn to_application(&self) -> zbus::Result<ApplicationProxy> {
		convert_to_new_type(self).await
	}
	async fn to_collection(&self) -> zbus::Result<CollectionProxy> {
		convert_to_new_type(self).await
	}
	async fn to_component(&self) -> zbus::Result<ComponentProxy> {
		convert_to_new_type(self).await
	}
	async fn to_document(&self) -> zbus::Result<DocumentProxy> {
		convert_to_new_type(self).await
	}
	async fn to_hypertext(&self) -> zbus::Result<HypertextProxy> {
		convert_to_new_type(self).await
	}
	async fn to_hyperlink(&self) -> zbus::Result<HyperlinkProxy> {
		convert_to_new_type(self).await
	}
	async fn to_image(&self) -> zbus::Result<ImageProxy> {
		convert_to_new_type(self).await
	}
	async fn to_selection(&self) -> zbus::Result<SelectionProxy> {
		convert_to_new_type(self).await
	}
	async fn to_table(&self) -> zbus::Result<TableProxy> {
		convert_to_new_type(self).await
	}
	async fn to_table_cell(&self) -> zbus::Result<TableCellProxy> {
		convert_to_new_type(self).await
	}
	async fn to_text(&self) -> zbus::Result<TextProxy> {
		convert_to_new_type(self).await
	}
	async fn to_editable_text(&self) -> zbus::Result<EditableTextProxy> {
		convert_to_new_type(self).await
	}
	async fn to_value(&self) -> zbus::Result<ValueProxy> {
		convert_to_new_type(self).await
	}
}
