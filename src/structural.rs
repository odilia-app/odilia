use odilia_common::types::Accessible;
use crate::state::ScreenReaderState;
use atspi::accessible::AccessibleProxy;
use eyre;
use atspi::accessible::Role;
use zbus::{
  Connection,
  zvariant::{
    ObjectPath,
    OwnedObjectPath,
  },
  names::UniqueName,
};
use zbus;

pub async fn make_accessible(connection: &Connection, accessible: Accessible) -> zbus::Result<AccessibleProxy<'static>> {
  AccessibleProxy::builder(&connection)
    .destination(UniqueName::try_from(accessible.0)?)?
    .path(ObjectPath::try_from(accessible.1)?)?
    .build()
    .await
}

pub async fn parent_of(state: &ScreenReaderState, accessible: &AccessibleProxy<'_>) -> zbus::Result<AccessibleProxy<'static>> {
  Ok(
    make_accessible(
      state.atspi.connection(),
      accessible.parent().await?
  ).await?)
}

/* Can't use map because of async */
pub async fn chlidren_of(state: &ScreenReaderState, accessible: AccessibleProxy<'_>) -> zbus::Result<Vec<AccessibleProxy<'static>>> {
  let children_raw = accessible.get_children().await?;
  let mut children = Vec::new();
  for child in children_raw {
    children.push(make_accessible(state.atspi.connection(), child).await?);
  }
  Ok(children)
}

pub async fn get_ancestor_with_role(state: &ScreenReaderState, accessible: &AccessibleProxy<'_>, role: Role) -> zbus::Result<AccessibleProxy<'static>> {
  let mut ancestor = parent_of(state, accessible).await?;
  let mut ancestor_role = ancestor.get_role().await?;
  while ancestor_role != role && ancestor_role != Role::RootPane {
    ancestor = parent_of(state, &ancestor).await?;
    ancestor_role = ancestor.get_role().await?;
    tracing::debug!("Ancestor with role: {:?}", ancestor_role);
  }
  Ok(ancestor)
}
