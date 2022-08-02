use async_recursion::async_recursion;
use async_trait::async_trait;
use crate::accessible::{
    RelationType,
    AccessibleProxy,
    Role
};
use crate::convertable::Convertable;
use std::{
    future::Future,
    collections::HashMap,
};

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
              F: Future<Output=zbus::Result<bool>> + Send;
    async fn get_relation_set_plus<'a>(&self) -> zbus::Result<HashMap<RelationType, Vec<AccessibleProxy<'a>>>>;
}

impl AccessibleProxy<'_> {
    #[async_recursion]
    async fn find_inner<T, F, 'a>(&self, after_or_before: i32, matcher: T, backward: bool, recur: bool) -> zbus::Result<Option<AccessibleProxy<'a>>> 
    where T: Fn(AccessibleProxy<'a>) -> F + Send + Sync + Copy,
          F: Future<Output=zbus::Result<bool>> + Send,
    {
        let children = match backward {
            false => self.get_children_plus().await?,
            true => {
              let mut vec = self.get_children_plus().await?;
              vec.reverse();
              vec
            }
         };
        for child in children {
            let child_index = child.get_index_in_parent().await?;
            if !recur &&
                ((child_index <= after_or_before && !backward) ||
                 (child_index >= after_or_before && backward)) {
                continue;
            }
            if matcher(child.clone()).await? {
                return Ok(Some(child));
            }
            /* 0 here is ignored because we are recursive; see the line starting with if !recur */
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
        let parent_parts = self.parent().await?;
        AccessibleProxy::builder(self.connection())
            .destination(parent_parts.0)?
            .path(parent_parts.1)?
            .build()
            .await
    }
    async fn get_children_plus<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let children_parts = self.get_children().await?;
        let mut children = Vec::new();
        for child_parts in children_parts {
            let acc = AccessibleProxy::builder(self.connection())
                .destination(child_parts.0)?
                .path(child_parts.1)?
                .build()
                .await?;
            children.push(acc);
        }
        Ok(children)
    }
    async fn get_siblings<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let parent = self.get_parent_plus().await?;
        let index = self.get_index_in_parent().await? as usize;
        let children: Vec<AccessibleProxy<'a>> = parent.get_children_plus().await?
            .into_iter()
            .enumerate()
            .filter_map(|(i, a)| {
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
            ancestor = ancestor.get_parent_plus().await?;
        }
        Ok(ancestor)
    }
    async fn get_children_caret<'a>(&self, backward: bool) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let mut children_after_before = Vec::new();
        let caret_pos = self.to_text().await?.caret_offset().await?;
        let children_hyperlink = self.to_accessible().await?.get_children_plus().await?;
        for child in children_hyperlink {
            let hyperlink = child.to_hyperlink().await?;
            if let Ok(start_index) = hyperlink.start_index().await {
                if (start_index <= caret_pos && backward) || (start_index >= caret_pos && !backward) {
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
              F: Future<Output=zbus::Result<bool>> + Send
    {
        // TODO if backwards, check here
        let caret_children = self.get_children_caret(backward).await?;
        for child in caret_children {
            if matcher(child.clone()).await? {
                return Ok(Some(child));
            } else if let Some(found_sub) = child.find_inner(0, matcher, backward, true).await? {
                return Ok(Some(found_sub));
            }
        }
        let mut last_parent_index = self.get_index_in_parent().await?;
        if let Ok(mut parent) = self.get_parent_plus().await {
            while parent.get_role().await? != Role::InternalFrame {
                let found_inner_child = parent.find_inner(last_parent_index, matcher, backward, false).await?;
                if found_inner_child.is_some() {
                    return Ok(found_inner_child);
                }
                last_parent_index = parent.get_index_in_parent().await?;
                parent = parent.get_parent_plus().await?;
            }
        }
        Ok(None)
    }
    async fn get_relation_set_plus<'a>(&self) -> zbus::Result<HashMap<RelationType, Vec<AccessibleProxy<'a>>>> {
        let raw_relations = self.get_relation_set().await?;
        let mut relations = HashMap::new();
        for relation in raw_relations {
            let mut related_vec = Vec::new();
            for related in relation.1 {
                let accessible = AccessibleProxy::builder(self.connection())
                    .destination(related.0)?
                    .path(related.1)?
                    .build()
                    .await?;
                related_vec.push(accessible);
            }
            relations.insert(relation.0, related_vec);
        }
        Ok(relations)
    }
}
