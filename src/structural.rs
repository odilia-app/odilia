use async_recursion::async_recursion;
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
  tracing::debug!("Creating accessible: {:?}", accessible.1);
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
pub async fn get_children(state: &ScreenReaderState, accessible: &AccessibleProxy<'_>) -> zbus::Result<Vec<AccessibleProxy<'static>>> {
  let children_raw = accessible.get_children().await?;
  let mut children = Vec::new();
  for child in children_raw {
    children.push(make_accessible(state.atspi.connection(), child).await?);
  }
  Ok(children)
}

pub async fn get_siblings(state: &ScreenReaderState, accessible: &AccessibleProxy<'_>) -> zbus::Result<Vec<AccessibleProxy<'static>>> {
    let parent = parent_of(state, accessible).await?;
    let index = accessible.get_index_in_parent().await? as usize;
    let children: Vec<AccessibleProxy<'static>> = get_children(state, &parent).await?
        .into_iter()
        .enumerate()
        .filter_map(|(i,a)| if i != index { Some(a) } else { None })
        .collect();
    Ok(children)
}

pub async fn get_siblings_after(state: &ScreenReaderState, accessible: &AccessibleProxy<'_>) -> zbus::Result<Vec<AccessibleProxy<'static>>> {
    let parent = parent_of(state, accessible).await?;
    let index = accessible.get_index_in_parent().await? as usize;
    let children: Vec<AccessibleProxy<'static>> = get_children(state, &parent).await?
        .into_iter()
        .enumerate()
        .filter_map(|(i,a)| if i > index { Some(a) } else { None })
        .collect();
    Ok(children)
}
pub async fn get_siblings_before(state: &ScreenReaderState, accessible: &AccessibleProxy<'_>) -> zbus::Result<Vec<AccessibleProxy<'static>>> {
    let parent = parent_of(state, accessible).await?;
    let index = accessible.get_index_in_parent().await? as usize;
    let children: Vec<AccessibleProxy<'static>> = get_children(state, &parent).await?
        .into_iter()
        .enumerate()
        .filter_map(|(i,a)| if i < index { Some(a) } else { None })
        .collect();
    Ok(children)
}

#[async_recursion(?Send)]
pub async fn find_with_role(state: &ScreenReaderState, accessible: &AccessibleProxy<'_>, role: Role, backward: bool) -> zbus::Result<Option<AccessibleProxy<'static>>> {
    if accessible.get_role().await? == Role::ScrollPane {
        return Ok(None);
    }
    // ba = before/after
    let ba_siblings = match backward {
        true => get_siblings_before(state, accessible).await?,
        false => get_siblings_after(state, accessible).await?,
    };
    for sibling in ba_siblings {
        tracing::debug!("ROLE: {:?}", sibling.get_role().await?);
        if sibling.get_role().await? == role {
            return Ok(Some(sibling));
        }
        if let Ok(Some(decendent)) = find_with_role(state, &sibling, role, backward).await {
            tracing::debug!("Decendant was found.");
            return Ok(Some(decendent));
        }
    }
    let parent = parent_of(state, accessible).await?;
    find_with_role(state, &parent, role, backward).await
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
