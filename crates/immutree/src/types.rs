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

use serde::{Deserialize, Serialize};

/// An error type returned by calls to the API exposed by this crate.
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    /// Returned on an attempt to insert a root element in the [`Tree`], while a root element
    /// already exists (the root element can only be the first insertion).
    ///
    /// [`Tree`]: super::Tree
    #[error("Tree's root can be set only at the first insertion")]
    RootReplacement,

    /// Returned when the user-provided [`NodeId`] for a parent [`Tree`] node does not actually
    /// reside in the [`Tree`].
    ///
    /// [`Tree`]: super::Tree
    #[error("Parent NodeId '{0}' does not exist in the Tree")]
    NonExistentParent(NodeId),

    /// Returned when a user-provided [`NodeId`] does not correspond to an element stored in the
    /// [`Tree`].
    ///
    /// [`Tree`]: super::Tree
    #[error("NodeId '{0}' does not exist in the Tree")]
    InvalidNodeId(NodeId),
}

/// The type of the unique ID assigned to each node at the time of insertion in the [`Tree`].
///
/// It is also needed by various methods when there is a need to refer to a specific element in the
/// [`Tree`].
///
/// [`Tree`]: super::Tree
pub type NodeId = u32;

/// This is supplied to [`Tree::insert`], aiming to regulate the insertion of a new element in the
/// [`Tree`].
///
/// [`Tree`]: super::Tree
/// [`Tree::insert`]: super::Tree::insert
pub enum InsertMode<'insertion> {
    /// This is used when the new element should be inserted in the [`Tree`] as its root node.
    ///
    /// Note that the root node of the [`Tree`] can be changed only on the first insertion of an
    /// element in it.
    ///
    /// [`Tree`]: super::Tree
    AsRoot,

    /// This is used when the new element should be inserted in the [`Tree`] as a child of another
    /// node.
    /// A reference to that node's (i.e., the parent's) [`NodeId`] (which must have been acquired
    /// via an earlier element insertion) must be also supplied.
    ///
    /// [`Tree`]: super::Tree
    Under(&'insertion NodeId),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct TreeNode<T> {
    pub(super) data: T,

    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub(crate) children: Option<Vec<NodeId>>,
}

impl<T> TreeNode<T> {
    pub(super) fn add_child_id(&mut self, id: &NodeId) {
        self.children.get_or_insert(vec![]).push(*id)
    }
}

impl<T> From<T> for TreeNode<T> {
    fn from(data: T) -> Self {
        Self {
            data,
            children: None,
        }
    }
}
