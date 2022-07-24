use async_recursion::async_recursion;
use async_trait::async_trait;
use crate::accessible::{
    AccessibleProxy,
    Role
};
use crate::convertable::Convertable;
use std::future::Future;

#[async_trait]
pub trait AccessiblePlus {
    // Assumes that an accessible can be made from the component parts
    async fn get_parent_plus<'a>(&self) -> zbus::Result<AccessibleProxy<'a>>;
    async fn get_children_plus<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_siblings<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_siblings_before<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_siblings_after<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_ancestors<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_ancestor_with_role<'a>(&self, role: Role) -> zbus::Result<AccessibleProxy<'a>>;
    /* TODO: not sure where these should go since it requires both Text as a self interface and
     * Hyperlink as children interfaces. */
    async fn get_children_caret<'a>(&self, after: bool) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_next<T, F, 'a>(&self, matcher: T, backward: bool) -> zbus::Result<Option<AccessibleProxy<'a>>>
        where T: Fn(AccessibleProxy<'a>) -> F + Send + Sync + Copy,
              F: Future<Output=bool> + Send;
}

impl AccessibleProxy<'_> {
    #[async_recursion]
    async fn find_inner<T, F, 'a>(&self, after_or_before: i32, matcher: T, backward: bool, recur: bool) -> zbus::Result<Option<AccessibleProxy<'a>>> 
    where T: Fn(AccessibleProxy<'a>) -> F + Send + Sync + Copy,
          F: Future<Output=bool> + Send,
    {
        tracing::debug!("Find inner in role: {:?}", self.get_role().await?);
        tracing::debug!("Get children");
        let children = match backward {
            true => {
                let mut tmp = self.get_children_plus().await?;
                tmp.reverse();
                tmp
            },
            false => self.get_children_plus().await?
        };
        tracing::debug!("Children received");
        for child in children {
            tracing::debug!("Child: {:?}", child.path());
            tracing::debug!("Children: {:?}", child.get_role().await?);
            let child_index = child.get_index_in_parent().await?;
            tracing::debug!("Child index received.");
            if !recur &&
                ((child_index<= after_or_before && !backward) ||
                 (child_index >= after_or_before && backward)) {
                continue;
            }
            tracing::debug!("Does it match?");
            if matcher(child.clone()).await {
                tracing::debug!("\tYes");
                return Ok(Some(child));
            }
            tracing::debug!("\tNo");
            /* 0 here is ignored because we are recursive; see the line starting with if !recur */
            tracing::debug!("Go into inner again");
            if let Some(found_decendant) = child.find_inner(0, matcher, backward, true).await? {
                return Ok(Some(found_decendant));
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl AccessiblePlus for AccessibleProxy<'_> {
    async fn get_parent_plus<'a>(&self) -> zbus::Result<AccessibleProxy<'a>> {
        tracing::debug!("Get parent parts");
        let parent_parts = self.parent().await?;
        tracing::debug!("Parent parts received");
        AccessibleProxy::builder(self.connection())
            .destination(parent_parts.0)?
            .path(parent_parts.1)?
            .build()
            .await
    }
    async fn get_children_plus<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        tracing::debug!("Get child parts");
        let children_parts = self.get_children().await?;
        tracing::debug!("Child parts received");
        let mut children = Vec::new();
        for child_parts in children_parts {
            tracing::debug!("Create child struct");
            let acc = AccessibleProxy::builder(self.connection())
                .destination(child_parts.0)?
                .path(child_parts.1)?
                .build()
                .await?;
            tracing::debug!("Child struct successful");
            tracing::debug!("Try push");
            children.push(acc);
            tracing::debug!("Push success");
        }
        tracing::debug!("Sending ok back from children+");
        Ok(children)
    }
    async fn get_siblings<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let parent = self.get_parent_plus().await?;
        let index = self.get_index_in_parent().await? as usize;
        let children: Vec<AccessibleProxy<'a>> = parent.get_children_plus().await?
            .into_iter()
            .enumerate()
            .filter_map(|(i, a)| {
                tracing::debug!("Working on accessible element {i}");
                if i != index { Some(a) } else { None }
            })
            .collect();
        Ok(children)
    }
    async fn get_siblings_before<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let parent = self.get_parent_plus().await?;
        let index = self.get_index_in_parent().await? as usize;
        let children: Vec<AccessibleProxy<'a>> = parent.get_children_plus().await?
            .into_iter()
            .enumerate()
            .filter_map(|(i, a)| if i < index { Some(a) } else { None })
            .collect();
        Ok(children)
    }
    async fn get_siblings_after<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let parent = self.get_parent_plus().await?;
        let index = self.get_index_in_parent().await? as usize;
        let children: Vec<AccessibleProxy<'a>> = parent.get_children_plus().await?
            .into_iter()
            .enumerate()
            .filter_map(|(i, a)| if i > index { Some(a) } else { None })
            .collect();
        Ok(children)
    }
    async fn get_ancestors<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let mut ancestors = Vec::new();
        let mut ancestor = self.get_parent_plus().await?;
        while ancestor.get_role().await? != Role::Frame {
            ancestors.push(ancestor.clone());
            ancestor = ancestor.get_parent_plus().await?;
        }
        Ok(ancestors)
    }
    async fn get_ancestor_with_role<'a>(&self, role: Role) -> zbus::Result<AccessibleProxy<'a>> {
        let mut ancestor = self.get_parent_plus().await?;
        while ancestor.get_role().await? != role && ancestor.get_role().await? != Role::Frame {
            tracing::debug!("ROLE: {:?}", ancestor.get_role().await?);
            ancestor = ancestor.get_parent_plus().await?;
        }
        Ok(ancestor)
    }
    async fn get_children_caret<'a>(&self, before: bool) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let mut children_after_before = Vec::new();
        let caret_pos = self.to_text().await?.caret_offset().await?;
        let children_hyperlink = self.to_accessible().await?.get_children_plus().await?;
        for child in children_hyperlink {
            let hyperlink = child.to_hyperlink().await?;
            if let Ok(start_index) = hyperlink.start_index().await {
                if (start_index <= caret_pos && before) || (start_index >= caret_pos && !before) {
                    children_after_before.push(child);
                }
            // include all children which do not identify their positions, for some reason
            } else {
                children_after_before.push(child);
            }
        }
        Ok(children_after_before)
    }
    async fn get_next<T, F, 'a>(&self, matcher: T, backward: bool) -> zbus::Result<Option<AccessibleProxy<'a>>> 
        where T: Fn(AccessibleProxy<'a>) -> F + Send + Sync + Copy,
              F: Future<Output=bool> + Send
    {
        // TODO if backwards, check here
        let caret_children = self.get_children_caret(backward).await?;
        for child in caret_children {
            if matcher(child.clone()).await {
                return Ok(Some(child));
            }
        }
        if let Ok(mut parent) = self.get_parent_plus().await {
            tracing::debug!("Parent role {:?}", parent.get_role().await?);
            while parent.get_role().await? != Role::Frame {
                tracing::debug!("Parent role {:?}", parent.get_role().await?);
                tracing::debug!("----INNER START----");
                let found_inner_child = parent.find_inner(parent.get_index_in_parent().await?, matcher, backward, false).await?;
                tracing::debug!("----INNER END-----");
                if found_inner_child.is_some() {
                    return Ok(found_inner_child);
                }
                tracing::debug!("Set parent");
                parent = parent.get_parent_plus().await?;
                tracing::debug!("Parent set");
            }
        }
        Ok(None)
    }
}
