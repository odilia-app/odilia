use crate::{
    accessible::{AccessibleProxy, RelationType, Role},
    collection::MatchType,
    convertable::Convertable,
    InterfaceSet,
};
use async_recursion::async_recursion;
use async_trait::async_trait;
use std::{collections::HashMap, error::Error};
use zbus::CacheProperties;

pub type MatcherArgs = (
    Vec<Role>,
    MatchType,
    HashMap<String, String>,
    MatchType,
    InterfaceSet,
    MatchType,
);

#[async_trait]
pub trait AccessibleExt {
    // Assumes that an accessible can be made from the component parts
    async fn get_id(&self) -> Option<u32>;
    async fn get_parent_ext<'a>(&self) -> zbus::Result<AccessibleProxy<'a>>;
    async fn get_children_ext<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_siblings<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Box<dyn Error>>;
    async fn get_children_indexes<'a>(&self) -> zbus::Result<Vec<i32>>;
    async fn get_siblings_before<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Box<dyn Error>>;
    async fn get_siblings_after<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Box<dyn Error>>;
    async fn get_ancestors<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_ancestor_with_role<'a>(&self, role: Role) -> zbus::Result<AccessibleProxy<'a>>;
    /* TODO: not sure where these should go since it requires both Text as a self interface and
     * Hyperlink as children interfaces. */
    async fn get_children_caret<'a>(&self, after: bool) -> zbus::Result<Vec<AccessibleProxy<'a>>>;
    async fn get_next<'a>(
        &self,
        matcher_args: &MatcherArgs,
        backward: bool,
    ) -> zbus::Result<Option<AccessibleProxy<'a>>>;
    async fn get_relation_set_ext<'a>(
        &self,
    ) -> zbus::Result<HashMap<RelationType, Vec<AccessibleProxy<'a>>>>;
}

// TODO: make match more broad, allow use of other parameters
async fn match_(
    accessible: &AccessibleProxy<'_>,
    matcher_args: &MatcherArgs,
) -> zbus::Result<bool> {
    let roles = &matcher_args.0;
    if roles.len() == 1 {
        Ok(accessible.get_role().await? == *roles.get(0).unwrap())
    } else {
        Ok(false)
    }
}

impl AccessibleProxy<'_> {
    #[async_recursion]
    async fn find_inner<'a>(
        &self,
        after_or_before: i32,
        matcher_args: &MatcherArgs,
        backward: bool,
        recur: bool,
    ) -> zbus::Result<Option<AccessibleProxy<'a>>> {
        let children = if backward {
            let mut vec = self.get_children_ext().await?;
            vec.reverse();
            vec
        } else {
            self.get_children_ext().await?
        };
        for child in children {
            let child_index = child.get_index_in_parent().await?;
            if !recur
                && ((child_index <= after_or_before && !backward)
                    || (child_index >= after_or_before && backward))
            {
                continue;
            }
            if match_(&child.clone(), matcher_args).await? {
                return Ok(Some(child));
            }
            /* 0 here is ignored because we are recursive; see the line starting with if !recur */
            if let Some(found_decendant) =
                child.find_inner(0, matcher_args, backward, true).await?
            {
                return Ok(Some(found_decendant));
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl AccessibleExt for AccessibleProxy<'_> {
    async fn get_id(&self) -> Option<u32> {
        let path = self.path();
        if let Some(id) = path.split('/').next_back() {
            if let Ok(uid) = id.parse::<u32>() {
                tracing::debug!("ID: {:?}", uid);
                return Some(uid);
            }
        }
        None
    }
    async fn get_parent_ext<'a>(&self) -> zbus::Result<AccessibleProxy<'a>> {
        let parent_parts = self.parent().await?;
        AccessibleProxy::builder(self.connection())
            .destination(parent_parts.0)?
            .cache_properties(CacheProperties::No)
            .path(parent_parts.1)?
            .build()
            .await
    }
    async fn get_children_indexes<'a>(&self) -> zbus::Result<Vec<i32>> {
        let mut indexes = Vec::new();
        for child in self.get_children_ext().await? {
            indexes.push(child.get_index_in_parent().await?);
        }
        Ok(indexes)
    }
    async fn get_children_ext<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let children_parts = self.get_children().await?;
        let mut children = Vec::new();
        for child_parts in children_parts {
            let acc = AccessibleProxy::builder(self.connection())
                .destination(child_parts.0)?
                .cache_properties(CacheProperties::No)
                .path(child_parts.1)?
                .build()
                .await?;
            children.push(acc);
        }
        Ok(children)
    }
    async fn get_siblings<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Box<dyn Error>>  {
        let parent = self.get_parent_ext().await?;
        let index = self.get_index_in_parent().await?.try_into()?;
        let children: Vec<AccessibleProxy<'a>> = parent
            .get_children_ext()
            .await?
            .into_iter()
            .enumerate()
            .filter_map(|(i, a)| if i != index { Some(a) } else { None })
            .collect();
        Ok(children)
    }
    async fn get_siblings_before<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Box<dyn Error>> {
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
    async fn get_siblings_after<'a>(&self) -> Result<Vec<AccessibleProxy<'a>>, Box<dyn Error>> {
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
    async fn get_ancestors<'a>(&self) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let mut ancestors = Vec::new();
        let mut ancestor = self.get_parent_ext().await?;
        while ancestor.get_role().await? != Role::Frame {
            ancestors.push(ancestor.clone());
            ancestor = ancestor.get_parent_ext().await?;
        }
        Ok(ancestors)
    }
    async fn get_ancestor_with_role<'a>(&self, role: Role) -> zbus::Result<AccessibleProxy<'a>> {
        let mut ancestor = self.get_parent_ext().await?;
        while ancestor.get_role().await? != role && ancestor.get_role().await? != Role::Frame {
            ancestor = ancestor.get_parent_ext().await?;
        }
        Ok(ancestor)
    }
    async fn get_children_caret<'a>(
        &self,
        backward: bool,
    ) -> zbus::Result<Vec<AccessibleProxy<'a>>> {
        let mut children_after_before = Vec::new();
        let caret_pos = self.to_text().await?.caret_offset().await?;
        let children_hyperlink = self.to_accessible().await?.get_children_ext().await?;
        for child in children_hyperlink {
            let hyperlink = child.to_hyperlink().await?;
            if let Ok(start_index) = hyperlink.start_index().await {
                if (start_index <= caret_pos && backward) || (start_index >= caret_pos && !backward)
                {
                    children_after_before.push(child);
                }
            // include all children which do not identify their positions, for some reason
            } else {
                children_after_before.push(child);
            }
        }
        Ok(children_after_before)
    }
    async fn get_next<'a>(
        &self,
        matcher_args: &MatcherArgs,
        backward: bool,
    ) -> zbus::Result<Option<AccessibleProxy<'a>>> {
        // TODO if backwards, check here
        let caret_children = self.get_children_caret(backward).await?;
        for child in caret_children {
            if match_(&child.clone(), matcher_args).await? {
                return Ok(Some(child));
            } else if let Some(found_sub) =
                child.find_inner(0, matcher_args, backward, true).await?
            {
                return Ok(Some(found_sub));
            }
        }
        let mut last_parent_index = self.get_index_in_parent().await?;
        if let Ok(mut parent) = self.get_parent_ext().await {
            while parent.get_role().await? != Role::InternalFrame {
                let found_inner_child = parent
                    .find_inner(last_parent_index, matcher_args, backward, false)
                    .await?;
                if found_inner_child.is_some() {
                    return Ok(found_inner_child);
                }
                last_parent_index = parent.get_index_in_parent().await?;
                parent = parent.get_parent_ext().await?;
            }
        }
        Ok(None)
    }
    async fn get_relation_set_ext<'a>(
        &self,
    ) -> zbus::Result<HashMap<RelationType, Vec<AccessibleProxy<'a>>>> {
        let raw_relations = self.get_relation_set().await?;
        let mut relations = HashMap::new();
        for relation in raw_relations {
            let mut related_vec = Vec::new();
            for related in relation.1 {
                let accessible = AccessibleProxy::builder(self.connection())
                    .destination(related.0)?
                    .cache_properties(CacheProperties::No)
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
