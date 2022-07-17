// Copyright 2022 Christos Katsakioris
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This crate includes a simple implementation of a tree container, generic over `T`.
//!
//! The [`Tree`] defined in this crate is merely a container structure, and not a general-purpose
//! search-tree.
//! It is meant to store existing, immutable, hierarchical information.
//! Moving around elements in the [`Tree`], including removing them, is not supported at all.
//!
//! One of the main goals of the crate is to provide a tree data structure that is dead-simple to
//! serialize and deserialize.
use serde::{Deserialize, Serialize};

mod iterators;
mod types;

pub use iterators::AncestorIds;
pub use iterators::Ancestors;
pub use iterators::ImmediateDescendantIds;
pub use iterators::ImmediateDescendants;
pub use iterators::LeafIds;
pub use iterators::Leaves;
pub use types::Error;
pub use types::InsertMode;
pub use types::NodeId;

use types::TreeNode;

/// A simple implementation of a tree container structure, generic over the data stored.
///
/// This is a container structure, and not a general-purpose search-tree.
/// It is meant to store existing, immutable, hierarchical information; therefore, moving around
/// elements within it, including removing them, is not supported at all.
///
/// One of its main goals is to be dead-simple to serialize and deserialize.
///
/// # Notes
///
/// - This data structure is not thread-safe (i.e., it is not meant to be used by multiple threads
/// concurrently, unless all accesses are read-only).
/// - A limited number of elements is supported (i.e., `u32::MAX`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tree<T> {
    pub(crate) nodes: Vec<TreeNode<T>>,

    #[serde(skip)]
    next_node_id: u32,
}

impl<T> Tree<T> {
    /// Allocate a new empty [`Tree`].
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_node_id: 0,
        }
    }

    /// Allocate a new empty [`Tree`], allocating as much as possible a priori.
    pub fn with_capacity(size: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(size),
            next_node_id: 0,
        }
    }

    /// Returns the number or elements currently stored in the [`Tree`].
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the [`Tree`] has no elements stored; `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        0 == self.nodes.len()
    }

    /// Returns a reference to the root element stored of the [`Tree`], if it exists; `None`
    /// otherwise.
    pub fn root(&self) -> Option<&T> {
        self.nodes.get(0).map(|tn| &tn.data)
    }

    /// Returns a reference to the element stored in the [`Tree`] under the provided [`NodeId`], if
    /// it exists; `None` otherwise.
    pub fn get_by_id(&self, id: &NodeId) -> Option<&T> {
        self.nodes.get(*id as usize).map(|tn| &tn.data)
    }

    /// Returns an iterator over the [`NodeId`]s that correspond to the immediate descendant
    /// (i.e., the children) elements of the element stored in the [`Tree`] under the provided
    /// `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNodeId`] if the provided [`NodeId`] does not correspond to an
    /// element currently stored in the [`Tree`].
    #[inline]
    pub fn immediate_descendant_ids(&self, id: &NodeId) -> Result<ImmediateDescendantIds, Error> {
        ImmediateDescendantIds::try_new(self, id)
    }

    /// Returns an iterator over the immediate descendant (i.e., the children) elements of the
    /// element stored in the [`Tree`] under the provided `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNodeId`] if the provided [`NodeId`] does not correspond to an
    /// element currently stored in the [`Tree`].
    #[inline]
    pub fn immediate_descendants(&self, id: &NodeId) -> Result<ImmediateDescendants<T>, Error> {
        ImmediateDescendants::try_new(self, id)
    }

    /// Returns an iterator over the [`NodeId`]s that correspond to the elements at the leaves of
    /// the [`Tree`], which are also descendants of the provided `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNodeId`] if the provided [`NodeId`] does not correspond to an
    /// element currently stored in the [`Tree`].
    ///
    /// # Note
    ///
    /// If the provided `id` corresponds to a leaf node, the iterator yields only that `id`.
    #[inline]
    pub fn leaf_descendant_ids(&self, id: &NodeId) -> Result<LeafIds<T>, Error> {
        LeafIds::try_new(self, id)
    }

    /// Returns an iterator over the elements stored at the leaves of the [`Tree`], which are also
    /// descendants of the provided `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNodeId`] if the provided [`NodeId`] does not correspond to an
    /// element currently stored in the [`Tree`].
    ///
    /// # Note
    ///
    /// If the provided `id` corresponds to a leaf node, the iterator yields only the element that
    /// corresponds to that `id`.
    #[inline]
    pub fn leaf_descendants(&self, id: &NodeId) -> Result<Leaves<T>, Error> {
        Leaves::try_new(self, id)
    }

    /// Returns an iterator over the [`NodeId`]s that correspond to the ancestor (i.e., the parent)
    /// elements of the element stored in the [`Tree`] under `id`.
    ///
    /// # Note
    ///
    /// The underlying algorithm's space and time complexities both are `Θ(|V|)`.
    #[inline]
    pub fn ancestor_ids(&self, id: &NodeId) -> AncestorIds<T> {
        AncestorIds::new(self, id)
    }

    /// Returns an iterator over the ancestor (i.e., the parent) elements of the element stored in
    /// the [`Tree`] under `id`.
    ///
    /// # Note
    ///
    /// The underlying algorithm's space and time complexities both are `Θ(|V|)`.
    #[inline]
    pub fn ancestors(&self, id: &NodeId) -> Ancestors<T> {
        Ancestors::new(self, id)
    }

    /// Returns the [`NodeId`] of the immediate ancestor (i.e., the parent) element of the element
    /// stored in the [`Tree`] under `id`, or `None` for the root element.
    ///
    /// # Note
    ///
    /// The underlying algorithm's time complexity is `O(|V|)`.
    pub fn parent_id(&self, id: &NodeId) -> Option<NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .find(|&(_, tn)| tn.children.is_some() && tn.children.as_ref().unwrap().contains(id))
            .map(|(parent_id, _)| parent_id as NodeId)
    }

    /// Returns the immediate ancestor (i.e., the parent) element of the element stored in the
    /// [`Tree`] under `id`, or `None` for the root element.
    ///
    /// # Note
    ///
    /// The underlying algorithm's time complexity is `O(|V|)`.
    pub fn parent(&self, id: &NodeId) -> Option<&T> {
        self.nodes
            .iter()
            .find(|&tn| tn.children.is_some() && tn.children.as_ref().unwrap().contains(id))
            .map(|tn| &tn.data)
    }

    /// Returns a `Vec` of the [`NodeId`]s that correspond to the children of the element stored in
    /// the [`Tree`] under the provided [`NodeId`], if they exist; `None` otherwise.
    #[cfg(test)]
    #[deprecated]
    pub fn children_ids(&self, id: &NodeId) -> Option<Vec<NodeId>> {
        self.nodes
            .get(*id as usize)
            .and_then(|tn| tn.children.clone())
    }

    /// Returns a `Vec` of the children elements of the element stored in the [`Tree`] under the
    /// provided [`NodeId`], if they exist; `None` otherwise.
    #[cfg(test)]
    #[deprecated]
    pub fn children(&self, id: &NodeId) -> Option<Vec<&T>> {
        self.nodes.get(*id as usize).and_then(|tn| {
            tn.children.as_ref().and_then(|children_ids| {
                children_ids
                    .iter()
                    .map(|child_id| self.get_by_id(child_id))
                    .collect()
            })
        })
    }

    /// Returns the [`NodeId`]s of the leaves of the [`Tree`] that are descendants of the provided
    /// `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNodeId`] if the provided [`NodeId`] does not correspond to an
    /// element currently stored in the [`Tree`].
    ///
    /// # Note
    ///
    /// If the provided `id` is a leaf, the method returns only that [`NodeId`].
    #[cfg(test)]
    #[deprecated]
    pub fn leaves_ids(&self, id: &NodeId) -> Result<Vec<NodeId>, Error> {
        let mut ret = Vec::new();

        let mut stack = vec![(
            *id,
            self.nodes
                .get(*id as usize)
                .ok_or(Error::InvalidNodeId(*id))?,
        )];
        while let Some((id, tn)) = stack.pop() {
            if let Some(children) = tn.children.as_ref() {
                // SAFETY: We safely `unwrap` because `child_id` is retrieved from the `TreeNode`,
                // which has been sanitized during insertions (and the `Tree` is immutable).
                stack.extend(
                    children
                        .iter()
                        .map(|child_id| (*child_id, self.nodes.get(*child_id as usize).unwrap())),
                );
            } else {
                ret.push(id)
            }
        }

        Ok(ret)
    }

    /// Returns the data of the leaves of the [`Tree`] that are descendants of the provided `id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidNodeId`] if the provided [`NodeId`] does not correspond to an
    /// element currently stored in the [`Tree`].
    ///
    /// # Note
    ///
    /// If the provided `id` is a leaf, the method returns only the data that correspond to `id`.
    #[cfg(test)]
    #[deprecated]
    pub fn leaves(&self, id: &NodeId) -> Result<Vec<&T>, Error> {
        #[allow(deprecated)] // because this method is itself deprecated too
        self.leaves_ids(id)?
            .iter()
            .map(|leaf_id| {
                self.get_by_id(leaf_id)
                    .ok_or(Error::InvalidNodeId(*leaf_id))
            })
            .collect()
    }

    /// Insert the provided `element` into the [`Tree`].
    ///
    /// The provided `mode` regulates the insertion of the new element as root, or as a common node
    /// with a parent.
    /// The provided `mode` indicates whether the new `element` should be added as a root of the
    /// [`Tree`], or as a common node under a parent [`NodeId`] (passed via [`InsertMode`]).
    ///
    /// # Errors
    ///
    /// - Returns [`Error::RootReplacement`] if an attempt is made to add a root element in the
    /// [`Tree`] while there is one already.
    /// - Returns [`Error::NonExistentParent`] if the parent's [`NodeId`] (provided by the caller)
    /// does not correspond to an element currently stored in the [`Tree`].
    pub fn insert(&mut self, element: T, mode: InsertMode) -> Result<NodeId, Error> {
        // If next_node_id equals to 0 but the Tree is not empty, then this Tree must have been
        // constructed via deserialization, where next_node_id is ignored. Therefore, it must
        // be calculated again: it should be equal to the length of the Tree (because 0-indexed).
        // Note that this is not expected to be a common case in our targeted scenarios, since we
        // do not expect the need to insert any new nodes after deserializing such a Tree.
        if 0 == self.next_node_id && !self.is_empty() {
            self.next_node_id = self.len() as u32;
        }

        // Fail fast if attempted to change root after first insertion
        if matches!(mode, InsertMode::AsRoot) && 0 != self.next_node_id {
            return Err(Error::RootReplacement);
        }

        // Update self.nodes
        if let InsertMode::Under(&parent_id) = mode {
            // We reach every node through its parent, therefore the latter should already be
            // present in our `node_map`; if not, return an error.
            if parent_id >= self.next_node_id {
                return Err(Error::NonExistentParent(parent_id));
            }

            // SAFETY: We checked that `parent_id < self.next_node_id`, therefore some previous
            // insertion has resized `self.node_map` and `self.children` to accommodate at least
            // `parent_id` entries; hence the unchecked indexing.
            self.nodes[parent_id as usize].add_child_id(&self.next_node_id);
        }
        self.nodes.push(element.into());

        // Update self.next_node_id
        self.next_node_id += 1;

        Ok(self.next_node_id - 1)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{InsertMode, Tree};

    #[test]
    fn t1() -> Result<()> {
        let mut t = Tree::new();
        let n0 = t.insert(0, InsertMode::AsRoot)?;

        let n1 = t.insert(1, InsertMode::Under(&n0))?;
        let n2 = t.insert(2, InsertMode::Under(&n0))?;

        let n3 = t.insert(3, InsertMode::Under(&n1))?;
        let n4 = t.insert(4, InsertMode::Under(&n1))?;
        let n5 = t.insert(5, InsertMode::Under(&n2))?;
        let n6 = t.insert(6, InsertMode::Under(&n2))?;

        let _n7 = t.insert(7, InsertMode::Under(&n3))?;
        let _n8 = t.insert(8, InsertMode::Under(&n3))?;
        let _n9 = t.insert(9, InsertMode::Under(&n4))?;
        let _n10 = t.insert(10, InsertMode::Under(&n4))?;
        let _n11 = t.insert(11, InsertMode::Under(&n5))?;
        let _n12 = t.insert(12, InsertMode::Under(&n5))?;
        let _n13 = t.insert(13, InsertMode::Under(&n6))?;
        let _n14 = t.insert(14, InsertMode::Under(&n6))?;

        let expected = r#"{"nodes":[{"data":0,"desc":[1,2]},{"data":1,"desc":[3,4]},{"data":2,"desc":[5,6]},{"data":3,"desc":[7,8]},{"data":4,"desc":[9,10]},{"data":5,"desc":[11,12]},{"data":6,"desc":[13,14]},{"data":7},{"data":8},{"data":9},{"data":10},{"data":11},{"data":12},{"data":13},{"data":14}]}"#;
        assert_eq!(serde_json::to_string(&t)?, expected);

        eprintln!(
            "Serialized Tree (pretty):\n{}",
            serde_json::to_string_pretty(&t)?
        );

        Ok(())
    }
}
