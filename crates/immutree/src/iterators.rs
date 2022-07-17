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

use core::iter::FusedIterator;
use core::slice::Iter;

use super::{Error, NodeId, Tree, TreeNode};

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////  ImmediateDescendantIds
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// An iterator over the [`NodeId`]s that correspond to the immediate descendant (i.e., the
/// children) elements of a specific element stored in the [`Tree`].
#[derive(Debug, Clone)]
pub struct ImmediateDescendantIds<'tree>(Option<Iter<'tree, NodeId>>);

impl<'tree> ImmediateDescendantIds<'tree> {
    pub(super) fn try_new<T>(tree: &'tree Tree<T>, id: &NodeId) -> Result<Self, Error> {
        Ok(Self(
            tree.nodes
                .get(*id as usize)
                .ok_or(Error::InvalidNodeId(*id))?
                .children
                .as_ref()
                .map(|children_ids| children_ids.iter()),
        ))
    }
}

impl<'tree> Iterator for ImmediateDescendantIds<'tree> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|inner| inner.next().copied())
    }
}

impl<'tree> FusedIterator for ImmediateDescendantIds<'tree> {}

impl<'tree> DoubleEndedIterator for ImmediateDescendantIds<'tree> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|inner| inner.next_back().copied())
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////  ImmediateDescendants
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// An iterator over the immediate descendant (i.e., the children) elements of a specific element
/// stored in the [`Tree`].
#[derive(Debug, Clone)]
pub struct ImmediateDescendants<'tree, T> {
    tree: &'tree Tree<T>,
    inner: ImmediateDescendantIds<'tree>,
}

impl<'tree, T> ImmediateDescendants<'tree, T> {
    pub(super) fn try_new(tree: &'tree Tree<T>, id: &NodeId) -> Result<Self, Error> {
        Ok(Self {
            tree,
            inner: ImmediateDescendantIds::try_new(tree, id)?,
        })
    }
}

impl<'tree, T> Iterator for ImmediateDescendants<'tree, T> {
    type Item = &'tree T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .and_then(|ref child_id| self.tree.get_by_id(child_id))
    }
}

impl<'tree, T> DoubleEndedIterator for ImmediateDescendants<'tree, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .and_then(|ref child_id| self.tree.get_by_id(child_id))
    }
}

impl<'tree, T> FusedIterator for ImmediateDescendants<'tree, T> {}

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////  LeafIds
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// An iterator over the [`NodeId`]s that correspond to the elements at the leaves of the [`Tree`],
/// which are also descendants of a specific [`NodeId`].
#[derive(Debug, Clone)]
pub struct LeafIds<'tree, T> {
    tree: &'tree Tree<T>,
    stack: Vec<(NodeId, &'tree TreeNode<T>)>,
}

impl<'tree, T> LeafIds<'tree, T> {
    pub(super) fn try_new(tree: &'tree Tree<T>, id: &'_ NodeId) -> Result<Self, Error> {
        Ok(Self {
            tree,
            stack: vec![(
                *id,
                tree.nodes
                    .get(*id as usize)
                    .ok_or(Error::InvalidNodeId(*id))?,
            )],
        })
    }
}

impl<'tree, T> Iterator for LeafIds<'tree, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id, tn)) = self.stack.pop() {
            if let Some(children) = tn.children.as_ref() {
                // SAFETY: We safely `unwrap` because `child_id` is retrieved from the `TreeNode`,
                // which has been sanitized during insertions, and the `Tree` is immutable.
                self.stack.extend(
                    children.iter().map(|child_id| {
                        (*child_id, self.tree.nodes.get(*child_id as usize).unwrap())
                    }),
                );
            } else {
                return Some(id);
            }
        }
        None
    }
}

impl<'tree, T> FusedIterator for LeafIds<'tree, T> {}

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////  Leaves
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// An iterator over the elements stored at the leaves of the [`Tree`], which are also descendants
/// of a specific [`NodeId`].
#[derive(Debug, Clone)]
pub struct Leaves<'tree, T> {
    tree: &'tree Tree<T>,
    ids_iter: LeafIds<'tree, T>,
}

impl<'tree, T> Leaves<'tree, T> {
    pub(super) fn try_new(tree: &'tree Tree<T>, id: &'_ NodeId) -> Result<Self, Error> {
        Ok(Self {
            tree,
            ids_iter: LeafIds::try_new(tree, id)?,
        })
    }
}

impl<'tree, T> Iterator for Leaves<'tree, T> {
    type Item = &'tree T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref id) = self.ids_iter.next() {
            return self.tree.get_by_id(id);
        }
        None
    }
}

impl<'tree, T> FusedIterator for Leaves<'tree, T> {}

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////  AncestorIds
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// An iterator over the [`NodeId`]s that correspond to the ancestor (i.e., the parent) elements of
/// a specific element stored in the [`Tree`].
///
/// # Note
///
/// The underlying algorithm's space and time complexities both are `Θ(|V|)`.
#[derive(Debug, Clone)]
pub struct AncestorIds<'tree, T> {
    tree: &'tree Tree<T>,
    found: bool,
    parents: Vec<Option<NodeId>>,
    curr: Option<NodeId>,
}

impl<'tree, T> AncestorIds<'tree, T> {
    pub(super) fn new(tree: &'tree Tree<T>, id: &NodeId) -> Self {
        Self {
            tree,
            found: false,
            parents: vec![None; tree.nodes.len()],
            curr: Some(*id),
        }
    }
}

impl<'tree, T> Iterator for AncestorIds<'tree, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.found {
            //self.parents.resize(self.tree.nodes.len(), None);
            for (parent_id, tn) in self.tree.nodes.iter().enumerate() {
                if let Some(children) = tn.children.as_ref() {
                    for &child_id in children {
                        self.parents[child_id as usize] = Some(parent_id as NodeId);
                    }
                }
            }
            self.found = true;
        }
        //self.stack.pop().map(|(id, _)| id)

        if let Some(curr) = self.curr {
            self.curr = self.parents[curr as usize];
        }
        self.curr
    }
}

impl<'tree, T> FusedIterator for AncestorIds<'tree, T> {}

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////  Ancestors
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// An iterator over the ancestor (i.e., the parent) elements of a specific element stored in the
/// [`Tree`].
///
/// # Note
///
/// The underlying algorithm's space and time complexities both are `Θ(|V|)`.
#[derive(Debug, Clone)]
pub struct Ancestors<'tree, T> {
    tree: &'tree Tree<T>,
    inner: AncestorIds<'tree, T>,
}

impl<'tree, T> Ancestors<'tree, T> {
    pub(super) fn new(tree: &'tree Tree<T>, id: &NodeId) -> Self {
        Self {
            tree,
            inner: AncestorIds::new(tree, id),
        }
    }
}

impl<'tree, T> Iterator for Ancestors<'tree, T> {
    type Item = &'tree T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref id) = self.inner.next() {
            return self.tree.get_by_id(id);
        }
        None
    }
}
