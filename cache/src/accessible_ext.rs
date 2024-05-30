use crate::convertable::Convertable;
use crate::AccessiblePrimitive;
use crate::CacheProperties;
use crate::OdiliaError;
use atspi_common::{ObjectRef, RelationType, Role};
use atspi_proxies::accessible::AccessibleProxy;
use std::collections::HashMap;
use std::future::Future;

pub trait AccessibleExt {
	type Error: std::error::Error;
	fn get_application_ext<'a>(
		&self,
	) -> impl Future<Output = Result<AccessibleProxy<'a>, Self::Error>> + Send
	where
		Self: Sized;
	fn get_parent_ext<'a>(
		&self,
	) -> impl Future<Output = Result<AccessibleProxy<'a>, Self::Error>> + Send
	where
		Self: Sized;
	fn get_children_ext<'a>(
		&self,
	) -> impl Future<Output = Result<Vec<AccessibleProxy<'a>>, Self::Error>> + Send
	where
		Self: Sized;
	fn get_siblings<'a>(
		&self,
	) -> impl Future<Output = Result<Vec<AccessibleProxy<'a>>, Self::Error>> + Send
	where
		Self: Sized;
	fn get_children_indexes<'a>(
		&self,
	) -> impl Future<Output = Result<Vec<i32>, Self::Error>> + Send;
	fn get_siblings_before<'a>(
		&self,
	) -> impl Future<Output = Result<Vec<AccessibleProxy<'a>>, Self::Error>> + Send
	where
		Self: Sized;
	fn get_siblings_after<'a>(
		&self,
	) -> impl Future<Output = Result<Vec<AccessibleProxy<'a>>, Self::Error>> + Send
	where
		Self: Sized;
	fn get_children_from_caret<'a>(
		&self,
		after: bool,
	) -> impl Future<Output = Result<Vec<AccessibleProxy<'a>>, Self::Error>> + Send
	where
		Self: Sized;
	/* TODO: not sure where these should go since it requires both Text as a self interface and
	 * Hyperlink as children interfaces. */
	fn get_next<'a>(
		&self,
		role: Role,
		backward: bool,
	) -> impl Future<Output = Result<Option<AccessibleProxy<'a>>, Self::Error>> + Send
	where
		Self: Sized;
	/// Get all edges for a given accessible object.
	/// This means: all children, siblings, and parent, in that order.
	/// If a direction is specified, then it will only get the appicable matching siblings/children.
	/// This also checks if the element supports the text interface, and then checks if the caret position is contained within the string, if it is, then children are also handled by direction.
	fn edges<'a>(
		&self,
		backward: Option<bool>,
	) -> impl Future<Output = Result<Vec<AccessibleProxy<'a>>, Self::Error>> + Send
	where
		Self: Sized;
	fn get_relation_set_ext<'a>(
		&self,
	) -> impl Future<
		Output = Result<HashMap<RelationType, Vec<AccessibleProxy<'a>>>, Self::Error>,
	> + Send
	where
		Self: Sized;
	fn match_(&self, role: Role) -> impl Future<Output = Result<bool, OdiliaError>> + Send;
}

impl AccessibleExt for AccessibleProxy<'_> {
	type Error = OdiliaError;
	async fn get_application_ext<'a>(&self) -> Result<AccessibleProxy<'a>, Self::Error>
	where
		Self: Sized,
	{
		let or: ObjectRef = self.get_application().await?;
		let io: AccessiblePrimitive = or.into();
		Ok(io.into_accessible(self.as_ref().connection()).await?)
	}
	async fn get_parent_ext<'a>(&self) -> Result<AccessibleProxy<'a>, Self::Error>
	where
		Self: Sized,
	{
		let or: ObjectRef = self.parent().await?;
		let io: AccessiblePrimitive = or.into();
		Ok(io.into_accessible(self.as_ref().connection()).await?)
	}
	async fn get_children_indexes<'a>(&self) -> Result<Vec<i32>, Self::Error> {
		let mut indexes = Vec::new();
		for child in self.get_children_ext().await? {
			indexes.push(child.get_index_in_parent().await?);
		}
		Ok(indexes)
	}
	async fn get_children_ext<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Self::Error>
	where
		Self: Sized,
	{
		let children_refs = self.get_children().await?;
		let mut children = Vec::new();
		for child_refs in children_refs {
			let acc = AccessibleProxy::builder(self.as_ref().connection())
				.destination(child_refs.name)?
				.cache_properties(CacheProperties::No)
				.path(child_refs.path)?
				.build()
				.await?;
			children.push(acc);
		}
		Ok(children)
	}
	async fn get_siblings<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Self::Error>
	where
		Self: Sized,
	{
		let parent = self.get_parent_ext().await?;
		let pin = self.get_index_in_parent().await?;
		let index = pin.try_into()?;
		// Clippy false positive: Standard pattern for excluding index item from list.
		#[allow(clippy::if_not_else)]
		let children: Vec<AccessibleProxy<'a>> = parent
			.get_children_ext()
			.await?
			.into_iter()
			.enumerate()
			.filter_map(|(i, a)| if i != index { Some(a) } else { None })
			.collect();
		Ok(children)
	}
	async fn get_siblings_before<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Self::Error>
	where
		Self: Sized,
	{
		let parent = self.get_parent_ext().await?;
		let index = self.get_index_in_parent().await?.try_into()?;
		let children: Vec<AccessibleProxy<'a>> = parent
			.get_children_ext()
			.await?
			.into_iter()
			.enumerate()
			.filter_map(|(i, a)| if i < index { Some(a) } else { None })
			.collect();
		Ok(children)
	}
	async fn get_siblings_after<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Self::Error>
	where
		Self: Sized,
	{
		let parent = self.get_parent_ext().await?;
		let index = self.get_index_in_parent().await?.try_into()?;
		let children: Vec<AccessibleProxy<'a>> = parent
			.get_children_ext()
			.await?
			.into_iter()
			.enumerate()
			.filter_map(|(i, a)| if i > index { Some(a) } else { None })
			.collect();
		Ok(children)
	}
	async fn get_children_from_caret<'a>(
		&self,
		backward: bool,
	) -> Result<Vec<AccessibleProxy<'a>>, Self::Error>
	where
		Self: Sized,
	{
		let mut children_from_cursor = Vec::new();
		let text_iface = self.to_text().await?;
		let caret_pos = text_iface.caret_offset().await?;
		let children_hyperlink = self.get_children_ext().await?;
		for child in children_hyperlink {
			let hyperlink = child.to_hyperlink().await?;
			if let Ok(start_index) = hyperlink.start_index().await {
				if (start_index <= caret_pos && backward)
					|| (start_index >= caret_pos && !backward)
				{
					children_from_cursor.push(child);
				}
			// include all children which do not identify their positions, for some reason
			} else {
				children_from_cursor.push(child);
			}
		}
		Ok(children_from_cursor)
	}
	async fn edges<'a>(
		&self,
		backward: Option<bool>,
	) -> Result<Vec<AccessibleProxy<'a>>, Self::Error>
	where
		Self: Sized,
	{
		let mut edge_elements = Vec::new();
		let children = match backward {
			Some(backward) => {
				if let Ok(caret_children) =
					self.get_children_from_caret(backward).await
				{
					caret_children
				} else {
					self.get_children_ext().await?
				}
			}
			None => self.get_children_ext().await?,
		};
		children.into_iter().for_each(|child| edge_elements.push(child));
		let siblings = match backward {
			Some(false) => self.get_siblings_before().await?,
			Some(true) => self.get_siblings_after().await?,
			None => self.get_siblings().await?,
		};
		siblings.into_iter().for_each(|sibling| edge_elements.push(sibling));
		let parent = self.get_parent_ext().await?;
		edge_elements.push(parent);
		Ok(edge_elements)
	}
	async fn get_next<'a>(
		&self,
		role: Role,
		backward: bool,
	) -> Result<Option<AccessibleProxy<'a>>, Self::Error>
	where
		Self: Sized,
	{
		let mut visited = Vec::new();
		let mut stack: Vec<AccessibleProxy<'_>> = Vec::new();
		let edges = self.edges(Some(backward)).await?;
		edges.into_iter().for_each(|edge| stack.push(edge));
		while let Some(item) = stack.pop() {
			// TODO: properly bubble up error
			let Ok(identifier) = ObjectRef::try_from(&item) else {
				return Ok(None);
			};
			// the top of the hirearchy for strctural navigation.
			if visited.contains(&identifier) {
				continue;
			}
			visited.push(identifier);
			if item.get_role().await? == Role::InternalFrame {
				return Ok(None);
			}
			// if it matches, then return it
			if item.match_(role).await? {
				return Ok(Some(item));
			}
			// if it doesnt match, add all edges
			self.edges(Some(backward))
				.await?
				.into_iter()
				.for_each(|edge| stack.push(edge));
		}
		Ok(None)
	}
	async fn get_relation_set_ext<'a>(
		&self,
	) -> Result<HashMap<RelationType, Vec<AccessibleProxy<'a>>>, Self::Error>
	where
		Self: Sized,
	{
		let raw_relations = self.get_relation_set().await?;
		let mut relations = HashMap::new();
		for relation in raw_relations {
			let mut related_vec = Vec::new();
			for related in relation.1 {
				let related_ap: AccessiblePrimitive = related.into();
				let ap: AccessibleProxy<'_> = related_ap
					.into_accessible(self.as_ref().connection())
					.await?;
				related_vec.push(ap);
			}
			relations.insert(relation.0, related_vec);
		}
		Ok(relations)
	}
	// TODO: make match more broad, allow use of other parameters; also, support multiple roles, since right now, multiple will just exit immediately with false
	async fn match_(&self, role: Role) -> Result<bool, OdiliaError> {
		Ok(self.get_role().await? == role)
	}
}
