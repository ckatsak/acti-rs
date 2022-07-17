//! This crate provides definitions and convenience entities and methods to easily serialize,
//! deserialize and work with the hierarchical hardware topology of a physical machine for the
//! purposes of the ActiK8s project.

mod error;
mod iter;
mod types;

pub use error::Error;
pub use iter::NodeIds;
pub use types::CacheAttributes;
pub use types::CacheLevel;
pub use types::Element;
pub use types::ProcessingElement;

use hwloc2::{topology::Filter, ObjectType};
use immutree::{InsertMode, NodeId, Tree};
use serde::{Deserialize, Serialize};

/// Although hardware topology detection always happens the same way, the produced [`Topology`] may
/// vary based on the selected [`DetectionMode`].
pub enum DetectionMode {
    /// `Full` detection includes all hardware topology nodes that may be examined for the purposes
    /// of the ActiK8s project.
    Full,
    /// `IsolationBoundariesOnly` detection excludes any intermediate nodes in the hardware
    /// topology hierarchy; i.e., nodes that are the only child of their parent are excluded from
    /// the [`Topology`].
    ///
    /// # Note
    ///
    /// The only exception to this node exclusion rule on Intel is [`NumaNode`] as a child of
    /// [`Package`].
    ///
    /// [`Package`]: crate::ProcessingElement::Package
    /// [`NumaNode`]: crate::ProcessingElement::NumaNode
    IsolationBoundariesOnly,
}

/// Acti Topology is a subset of the hardware topology detected through `libhwloc2-rs`, useful for
/// the purposes of the ActiK8s project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Topology {
    tree: Tree<Element>,
}

impl Topology {
    /// Detect the underlying hardware topology employing `libhwloc2-rs`, process it, and return a
    /// new immutable Acti-[`Topology`].
    ///
    /// # Errors
    ///
    /// An [`Error`] is returned when any operation in `libhwloc2-rs` or [`immutree`] fails.
    ///
    /// # Panics
    ///
    /// Only in cases of unexpected results (certainly bugs) from the underlying `libhwloc2-rs`.
    pub fn detect(mode: DetectionMode) -> Result<Self, Error> {
        let topo = hwloc2::Topology::builder()?
            .all_types_filter(Filter::KeepNone)?
            .type_filter(ObjectType::Machine, Filter::KeepAll)?
            //.type_filter(ObjectType::Group, Filter::KeepAll)?
            .type_filter(ObjectType::Package, Filter::KeepAll)?
            .type_filter(ObjectType::Die, Filter::KeepAll)?
            .type_filter(ObjectType::NumaNode, Filter::KeepAll)?
            .type_filter(ObjectType::L1Cache, Filter::KeepAll)?
            .type_filter(ObjectType::L2Cache, Filter::KeepAll)?
            .type_filter(ObjectType::L3Cache, Filter::KeepAll)?
            .type_filter(ObjectType::L4Cache, Filter::KeepAll)?
            .type_filter(ObjectType::L5Cache, Filter::KeepAll)?
            .type_filter(ObjectType::Core, Filter::KeepAll)?
            .type_filter(ObjectType::PU, Filter::KeepAll)?
            .build()?;

        let mut tree = Tree::new();
        let root_obj = topo.root_object().ok_or(Error::EmptyTopology)?;
        let root_id = tree.insert(Element::try_from(&root_obj)?, InsertMode::AsRoot)?;

        let add_descendants_fn = match mode {
            DetectionMode::Full => Self::add_all_descendants,
            DetectionMode::IsolationBoundariesOnly => Self::add_isol_bound_descendants,
        };
        add_descendants_fn(&mut tree, &root_id, &root_obj)?;

        Ok(Self { tree })
    }

    /// Recursively add all descendant objects into the given `Tree<Element>`.
    fn add_all_descendants<'topo, 'tree>(
        tree: &'tree mut Tree<Element>,
        parent_node_id: &'tree NodeId,
        parent_obj: &'topo hwloc2::Object,
    ) -> Result<(), Error> {
        // First, insert any memory child (i.e., only a single NUMA node in our case).
        let parent_mem_node_id = match parent_obj.memory_arity() {
            0 => None,
            1 => {
                let mem_child_obj = parent_obj
                    .memory_first_child()
                    .expect("memory_first_child() is None, despite memory_arity() == 1");
                match mem_child_obj.object_type() {
                    ObjectType::NumaNode => Some(tree.insert(
                        Element::try_from(&mem_child_obj)?,
                        InsertMode::Under(parent_node_id),
                    )?),
                    _ => unreachable!("Memory child's type is '{}'", mem_child_obj.object_type()),
                }
            }
            _ => {
                // NOTE(ckatsak): I am not sure if memory_arity can ever be > 1, but we currently
                // do not support it anyway, because I don't know how to handle it in the hierarchy
                return Err(Error::MemoryArity(parent_obj.memory_arity()));
            }
        };

        // Then, deal with "normal" descendants.
        for child_idx in 0..parent_obj.arity() {
            let child_obj = parent_obj.children()[child_idx as usize];

            match Element::try_from(&child_obj) {
                Ok(child_elem) => {
                    let child_node_id = tree.insert(
                        child_elem,
                        InsertMode::Under(&parent_mem_node_id.unwrap_or(*parent_node_id)),
                    )?;
                    Self::add_all_descendants(tree, &child_node_id, &child_obj)?;
                }
                Err(Error::NoEquivalentElement) => {
                    Self::add_all_descendants(
                        tree,
                        &parent_mem_node_id.unwrap_or(*parent_node_id),
                        &child_obj,
                    )?;
                }
                Err(err) => unreachable!("Element::try_from() returned {err:?}"),
            }
        }

        Ok(())
    }

    /// Recursively add into the given `Tree<Element>` only descendant objects at isolation
    /// boundaries.
    fn add_isol_bound_descendants<'topo, 'tree>(
        tree: &'tree mut Tree<Element>,
        parent_node_id: &'tree NodeId,
        parent_obj: &'topo hwloc2::Object,
    ) -> Result<(), Error> {
        // First, insert any memory child (i.e., only a single NUMA node in our case).
        let parent_mem_node_id = match parent_obj.memory_arity() {
            0 => None,
            1 => {
                let mem_child_obj = parent_obj
                    .memory_first_child()
                    .expect("memory_first_child() is None, despite memory_arity() == 1");
                match mem_child_obj.object_type() {
                    ObjectType::NumaNode => Some(tree.insert(
                        Element::try_from(&mem_child_obj)?,
                        InsertMode::Under(parent_node_id),
                    )?),
                    _ => unreachable!("Memory child's type is '{}'", mem_child_obj.object_type()),
                }
            }
            _ => {
                // NOTE(ckatsak): I am not sure if memory_arity can ever be > 1, but we currently
                // do not support it anyway, because I don't know how to handle it in the hierarchy
                return Err(Error::MemoryArity(parent_obj.memory_arity()));
            }
        };

        // Then, deal with "normal" descendants.
        for child_idx in 0..parent_obj.arity() {
            let child_obj = parent_obj.children()[child_idx as usize];

            match Element::try_from(&child_obj) {
                Ok(child_elem) => {
                    if parent_obj.arity() > 1 {
                        let child_node_id = tree.insert(
                            child_elem,
                            InsertMode::Under(&parent_mem_node_id.unwrap_or(*parent_node_id)),
                        )?;
                        Self::add_isol_bound_descendants(tree, &child_node_id, &child_obj)?;
                    } else {
                        Self::add_isol_bound_descendants(
                            tree,
                            &parent_mem_node_id.unwrap_or(*parent_node_id),
                            &child_obj,
                        )?;
                    }
                }
                Err(Error::NoEquivalentElement) => {
                    Self::add_isol_bound_descendants(
                        tree,
                        &parent_mem_node_id.unwrap_or(*parent_node_id),
                        &child_obj,
                    )?;
                }
                Err(err) => unreachable!("Element::try_from() returned {err:?}"),
            }
        }

        Ok(())
    }

    /// Returns an immutable reference to the inner `Tree<Element>` structure.
    #[inline]
    pub fn tree(&self) -> &Tree<Element> {
        &self.tree
    }

    /// Returns an iterator over the [`NodeId`]s that correspond to [`Element`]s in the topology
    /// for which the provided `match_fn` returns `true`.
    ///
    /// [`NodeId`]: immutree::NodeId
    pub fn filter_elements<F: Fn(&Element) -> bool>(&self, match_fn: F) -> NodeIds<F> {
        NodeIds::new(self, match_fn)
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to a [`ProcessingElement`]s in the
    /// topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`Package`]: crate::ProcessingElement::Package
    pub fn processing_element_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        self.filter_elements(|e| matches!(e, Element::Processing(_)))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`Package`]s in the topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`Package`]: crate::ProcessingElement::Package
    pub fn package_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        self.filter_elements(|e| matches!(e, Element::Processing(ProcessingElement::Package(_))))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`NumaNode`]s in the topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`NumaNode`]: crate::ProcessingElement::NumaNode
    pub fn numa_node_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        self.filter_elements(|e| matches!(e, Element::Processing(ProcessingElement::NumaNode(_))))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`Core`]s in the topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`Core`]: crate::ProcessingElement::Core
    pub fn core_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        self.filter_elements(|e| matches!(e, Element::Processing(ProcessingElement::Core(_))))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`Thread`]s in the topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`Thread`]: crate::ProcessingElement::Thread
    pub fn thread_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        self.filter_elements(|e| matches!(e, Element::Processing(ProcessingElement::Thread(_))))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`Cache`]s in the topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`Cache`]: crate::Element::Cache
    pub fn cache_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        self.filter_elements(|e| matches!(e, Element::Cache { .. }))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`L1`] [`Cache`]s in the
    /// topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`L1`]: crate::CacheLevel::L1
    /// [`Cache`]: crate::Element::Cache
    pub fn l1_cache_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        use CacheLevel::L1;
        self.filter_elements(|e| matches!(e, Element::Cache { level: L1, .. }))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`L2`] [`Cache`]s in the
    /// topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`L2`]: crate::CacheLevel::L2
    /// [`Cache`]: crate::Element::Cache
    pub fn l2_cache_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        use CacheLevel::L2;
        self.filter_elements(|e| matches!(e, Element::Cache { level: L2, .. }))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`L3`] [`Cache`]s in the
    /// topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`L3`]: crate::CacheLevel::L3
    /// [`Cache`]: crate::Element::Cache
    pub fn l3_cache_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        use CacheLevel::L3;
        self.filter_elements(|e| matches!(e, Element::Cache { level: L3, .. }))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`L4`] [`Cache`]s in the
    /// topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`L4`]: crate::CacheLevel::L4
    /// [`Cache`]: crate::Element::Cache
    pub fn l4_cache_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        use CacheLevel::L4;
        self.filter_elements(|e| matches!(e, Element::Cache { level: L4, .. }))
    }

    /// Returns an iterator over all [`NodeId`]s that correspond to [`L5`] [`Cache`]s in the
    /// topology.
    ///
    /// [`NodeId`]: immutree::NodeId
    /// [`L5`]: crate::CacheLevel::L5
    /// [`Cache`]: crate::Element::Cache
    pub fn l5_cache_ids(&self) -> NodeIds<'_, impl Fn(&Element) -> bool> {
        use CacheLevel::L5;
        self.filter_elements(|e| matches!(e, Element::Cache { level: L5, .. }))
    }

    //pub fn packages_original(&self) -> Vec<NodeId> {
    //    (0..self.tree.len())
    //        .filter_map(|id| {
    //            self.tree.get_by_id(&(id as NodeId)).and_then(|&e| {
    //                matches!(&e, Element::Processing(ProcessingElement::Package(_)))
    //                    .then_some(id as NodeId)
    //            })
    //        })
    //        .collect()
    //}
    ///// Returns the [`NodeId`]s that correspond to [`Element`]s in the topology for which the
    ///// provided `match_fn` returns `true`.
    /////
    ///// The complexity of the function is `O(|V| * M)`, where `|V|` is the number of [`Element`]s
    ///// in the topology's [`Tree`], and `M` is the complexity of the provided `match_fn`.
    //pub fn filter_elems<F: Fn(&Element) -> bool>(&self, match_fn: F) -> Vec<NodeId> {
    //    (0..self.tree.len())
    //        .filter_map(|id| {
    //            self.tree
    //                .get_by_id(&(id as NodeId))
    //                .and_then(|e| match_fn(e).then_some(id as NodeId))
    //        })
    //        .collect()
    //}
    //pub fn numa_nodes(&self) -> Vec<NodeId> {
    //    self.filter_elems(|e| matches!(e, Element::Processing(ProcessingElement::NumaNode(_))))
    //}
    //pub fn l1_caches(&self) -> Vec<NodeId> {
    //    use CacheLevel::L1;
    //    self.filter_elems(|e| matches!(e, Element::Cache { level: L1, .. }))
    //}
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::{BufWriter, Write},
    };

    use anyhow::Result;
    use hwloc2::{topology::Filter, Object, ObjectType};

    use crate::{DetectionMode, Topology};

    //const TERMI5_TOPO_FILE: &str = "test-artifacts/topo__actitree.json";

    fn print_children_attrs(obj: Object, depth: usize) {
        let padding = " ".repeat(4 * depth);
        eprintln!(
            "\n\n{}{} ({}): #{}(L#{})\n{}└-attributes: {:?}",
            padding,
            obj,
            obj.object_type(),
            obj.os_index(),
            obj.logical_index(),
            padding,
            obj.attributes(),
        );

        if obj.memory_arity() > 0 {
            let mem_child = obj
                .memory_first_child()
                .expect("failed to retrieve first memory child");
            eprintln!(
                "{}└-{} ({}): #{}(L#{}) ({} children)\n{}  └-attributes: {:?}",
                padding,
                mem_child,
                mem_child.object_type(),
                mem_child.os_index(),
                mem_child.logical_index(),
                mem_child.arity(),
                padding,
                mem_child.attributes(),
            );
        }
        for i in 0..obj.arity() {
            print_children_attrs(obj.children()[i as usize], depth + 1)
        }
    }

    fn get_topo() -> Result<hwloc2::Topology> {
        Ok(hwloc2::Topology::builder()?
            .all_types_filter(Filter::KeepNone)?
            .type_filter(ObjectType::Machine, Filter::KeepAll)?
            .type_filter(ObjectType::Package, Filter::KeepAll)?
            .type_filter(ObjectType::NumaNode, Filter::KeepAll)?
            .type_filter(ObjectType::L1Cache, Filter::KeepAll)?
            .type_filter(ObjectType::L2Cache, Filter::KeepAll)?
            .type_filter(ObjectType::L3Cache, Filter::KeepAll)?
            .type_filter(ObjectType::L4Cache, Filter::KeepAll)?
            .type_filter(ObjectType::L5Cache, Filter::KeepAll)?
            .type_filter(ObjectType::Core, Filter::KeepAll)?
            .type_filter(ObjectType::PU, Filter::KeepAll)?
            .build()?)
    }

    #[test]
    fn t1() -> Result<()> {
        let topo = get_topo()?;
        eprintln!("\n\n{topo:#?}\n\n");

        print_children_attrs(
            topo.root_object()
                .expect("failed to retrieve topology's root object"),
            0,
        );
        Ok(())
    }

    #[test]
    fn t2() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;
        eprintln!("{topo:#?}");
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&topo)
                .expect("failed to pretty-stringify actitopo::Topology")
        );
        Ok(())
    }

    #[test]
    fn t4_de() -> Result<()> {
        const T4_JSON_FILE: &str = "test-artifacts/t4_de.json";
        const T4_TXT_FILE: &str = "test-artifacts/t4_de.txt";

        let topo = Topology::detect(DetectionMode::IsolationBoundariesOnly)?;
        //let _root = topo.tree().root().unwrap();

        //
        // Serialize the tree into a JSON string and dump it into a file.
        //
        let serialized = serde_json::to_string(topo.tree())?;
        eprintln!("Serialized size: {} B", serialized.len());
        fs::write(T4_JSON_FILE, &serialized)?;
        assert_eq!(serialized.len(), fs::metadata(T4_JSON_FILE)?.len() as usize);

        //
        // Deserialize the actitopo::Topology from the JSON file...
        //
        let treestr = fs::read_to_string(T4_JSON_FILE)?;
        let detopo: Topology = serde_json::from_str(&treestr)?;
        //
        // ...and print each node's information into a text file.
        //
        {
            let mut bw = BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(T4_TXT_FILE)?,
            );
            let dt = detopo.tree();
            for id in 0..dt.len() as u32 {
                assert_eq!(dt.ancestor_ids(&id).next(), dt.parent_id(&id));
                assert_eq!(dt.ancestors(&id).next(), dt.parent(&id));

                writeln!(bw, "- Node {id}: {:?}", dt.get_by_id(&id))?;
                writeln!(
                    bw,
                    "\t\tParent: ({:?}) {:?}",
                    dt.parent_id(&id),
                    dt.parent(&id)
                )?;
                writeln!(
                    bw,
                    "\t\tDescendant IDs: {:?}",
                    dt.immediate_descendant_ids(&id)?.collect::<Vec<_>>()
                )?;
                writeln!(
                    bw,
                    "\t\tDescendants: {:?}",
                    dt.immediate_descendants(&id)?.collect::<Vec<_>>()
                )?;
                writeln!(
                    bw,
                    "\t\tAncestor IDs: {:?}",
                    dt.ancestor_ids(&id).collect::<Vec<_>>()
                )?;
                writeln!(
                    bw,
                    "\t\tAncestors: {:?}",
                    dt.ancestors(&id).collect::<Vec<_>>()
                )?;
                writeln!(
                    bw,
                    "\t\tLeaf IDs: {:?}",
                    dt.leaf_descendant_ids(&id)?.collect::<Vec<_>>()
                )?;
                writeln!(
                    bw,
                    "\t\tLeaves: {:?}\n\n",
                    dt.leaf_descendants(&id)?.collect::<Vec<_>>()
                )?;
            }
        }

        //fs::remove_file(T4_JSON_FILE)?;
        //fs::remove_file(T4_TXT_FILE)?;

        Ok(())
    }

    #[test]
    fn test_filter_package_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.package_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::package_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_filter_numa_nodes_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.numa_node_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::numa_node_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_filter_core_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.core_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::core_ids returned invalid id")
            );
        }

        Ok(())
    }

    //#[test]
    //fn test_filter_packages() -> Result<()> {
    //    let topo = Topology::detect(DetectionMode::Full)?;
    //
    //    for id in topo.packages() {
    //        eprintln!(
    //            "- NodeID {id}: {}",
    //            topo.tree()
    //                .get_by_id(&id)
    //                .expect("Topology::packages returned invalid id")
    //        );
    //    }
    //
    //    Ok(())
    //}
    //
    //#[test]
    //fn test_filter_numa_nodes() -> Result<()> {
    //    let topo = Topology::detect(DetectionMode::Full)?;
    //
    //    for id in topo.numa_nodes() {
    //        eprintln!(
    //            "- NodeID {id}: {}",
    //            topo.tree()
    //                .get_by_id(&id)
    //                .expect("Topology::numa_nodes returned invalid id")
    //        );
    //    }
    //
    //    Ok(())
    //}
    //
    //#[test]
    //fn test_filter_cores() -> Result<()> {
    //    let topo = Topology::detect(DetectionMode::Full)?;
    //
    //    for id in topo.cores() {
    //        eprintln!(
    //            "- NodeID {id}: {}",
    //            topo.tree()
    //                .get_by_id(&id)
    //                .expect("Topology::cores returned invalid id")
    //        );
    //    }
    //
    //    Ok(())
    //}
    //#[test]
    //fn test_filter_threads() -> Result<()> {
    //    let topo = Topology::detect(DetectionMode::Full)?;
    //
    //    for id in topo.threads() {
    //        eprintln!(
    //            "- NodeID {id}: {}",
    //            topo.tree()
    //                .get_by_id(&id)
    //                .expect("Topology::threads returned invalid id")
    //        );
    //    }
    //
    //    Ok(())
    //}
    //
    //#[test]
    //fn test_filter_caches() -> Result<()> {
    //    let topo = Topology::detect(DetectionMode::Full)?;
    //
    //    for id in topo.caches() {
    //        eprintln!(
    //            "- NodeID {id}: {}",
    //            topo.tree()
    //                .get_by_id(&id)
    //                .expect("Topology::caches returned invalid id")
    //        );
    //    }
    //
    //    Ok(())
    //}

    #[test]
    fn test_filter_thread_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.thread_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::thread_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_filter_cache_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.cache_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::cache_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_filter_l1_cache_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.l1_cache_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::l1_cache_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_filter_l2_cache_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.l2_cache_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::l2_cache_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_filter_l3_cache_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.l3_cache_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::l3_cache_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_filter_l4_cache_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.l4_cache_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::l4_cache_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_filter_l5_cache_ids() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for id in topo.l5_cache_ids() {
            eprintln!(
                "- NodeID {id}: {}",
                topo.tree()
                    .get_by_id(&id)
                    .expect("Topology::l5_cache_ids returned invalid id")
            );
        }

        Ok(())
    }

    #[test]
    fn test_both_topo_and_tree() -> Result<()> {
        let topo = Topology::detect(DetectionMode::Full)?;

        for pkg_id in topo.package_ids() {
            eprintln!(
                "\t- Parent: {}",
                topo.tree()
                    .get_by_id(&pkg_id)
                    .expect("Topology::package_ids returned invalid NodeId")
            );
            for child_id in topo.tree().immediate_descendant_ids(&pkg_id)? {
                eprintln!(
                    "\t\t- Child: {}",
                    topo.tree()
                        .get_by_id(&child_id)
                        .expect("Tree::immediate_descendant_ids returned invalid NodeId")
                );
                for gc_id in topo.tree().immediate_descendant_ids(&child_id)? {
                    eprintln!(
                        "\t\t\t- Grandchild: {}",
                        topo.tree()
                            .get_by_id(&gc_id)
                            .expect("Tree::immediate_descendant_ids returned invalid NodeId")
                    );
                    for ggc in topo.tree().immediate_descendants(&gc_id)? {
                        eprintln!("\t\t\t- Greatgrandchild: {ggc}");
                    }
                }
            }
        }

        Ok(())
    }
}
