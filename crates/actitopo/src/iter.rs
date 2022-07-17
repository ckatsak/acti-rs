use std::iter::FusedIterator;

use immutree::NodeId;

use crate::{Element, Topology};

/// An iterator over [`NodeId`]s that correspond to [`Element`]s in the [`Topology`].
///
/// [`NodeId`]: immutree::NodeId
/// [`Element`]: crate::types::Element
/// [`Topology`]: crate::Topology
pub struct NodeIds<'topo, F>
where
    F: Fn(&Element) -> bool,
{
    topo: &'topo Topology,
    match_fn: F,
    curr: NodeId,
}

impl<'topo, F> NodeIds<'topo, F>
where
    F: Fn(&Element) -> bool,
{
    pub(crate) fn new(topology: &'topo Topology, match_fn: F) -> Self {
        Self {
            topo: topology,
            match_fn,
            curr: 0,
        }
    }
}

impl<'topo, F> Iterator for NodeIds<'topo, F>
where
    F: Fn(&Element) -> bool,
{
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.topo.tree.get_by_id(&self.curr) {
            self.curr += 1;
            if (self.match_fn)(e) {
                return Some(self.curr - 1);
            }
        }
        None
    }
}

impl<'topo, F: Fn(&Element) -> bool> FusedIterator for NodeIds<'topo, F> {}
