use std::fmt;

use hwloc2::{object::Attributes, ObjectType};
use serde::{Deserialize, Serialize};

use crate::Error;

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////    Element
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Topology elements, as defined in terms of the Acti- node topology.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    /// The root element of the topology, representing the whole machine.
    Machine,

    /// A computation unit (e.g., physical core, etc).
    Processing(ProcessingElement),

    /// A data caching element (e.g., L3 cache, etc).
    Cache {
        /// The level of the cache.
        #[serde(rename = "lvl")]
        level: CacheLevel,

        /// The logical index of the cache, assigned by `libhwloc2-rs`.
        #[serde(rename = "li")]
        logical_index: u32,

        /// Attributes of the cache, detected by `libhwloc2-rs`.
        #[serde(rename = "attrs")]
        attributes: CacheAttributes,
    },
}

impl TryFrom<&hwloc2::Object<'_>> for Element {
    type Error = Error;

    fn try_from(obj: &hwloc2::Object) -> Result<Self, Self::Error> {
        match obj.object_type() {
            //
            // Root
            //
            ObjectType::Machine => Ok(Element::Machine),
            //
            // Processing elements
            //
            ObjectType::Package => Ok(Element::Processing(ProcessingElement::Package(
                obj.os_index(),
            ))),
            ObjectType::NumaNode => Ok(Element::Processing(ProcessingElement::NumaNode(
                obj.os_index(),
            ))),
            ObjectType::Core => Ok(Element::Processing(ProcessingElement::Core(obj.os_index()))),
            ObjectType::PU => Ok(Element::Processing(ProcessingElement::Thread(
                obj.os_index(),
            ))),
            //
            // Caches
            //
            ObjectType::L1Cache => Ok(Element::Cache {
                level: CacheLevel::L1,
                logical_index: obj.logical_index(),
                attributes: obj.attributes().try_into().unwrap_or_default(),
            }),
            ObjectType::L2Cache => Ok(Element::Cache {
                level: CacheLevel::L2,
                logical_index: obj.logical_index(),
                attributes: obj.attributes().try_into().unwrap_or_default(),
            }),
            ObjectType::L3Cache => Ok(Element::Cache {
                level: CacheLevel::L3,
                logical_index: obj.logical_index(),
                attributes: obj.attributes().try_into().unwrap_or_default(),
            }),
            ObjectType::L4Cache => Ok(Element::Cache {
                level: CacheLevel::L4,
                logical_index: obj.logical_index(),
                attributes: obj.attributes().try_into().unwrap_or_default(),
            }),
            ObjectType::L5Cache => Ok(Element::Cache {
                level: CacheLevel::L5,
                logical_index: obj.logical_index(),
                attributes: obj.attributes().try_into().unwrap_or_default(),
            }),
            //
            // No equivalent element in Acti-topology
            //
            _ => Err(Error::NoEquivalentElement),
        }
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Element::*;
        match self {
            Machine => write!(f, "Machine"),
            Processing(pe) => write!(f, "{pe}"),
            Cache {
                level,
                logical_index,
                attributes,
            } => write!(f, "{level} Cache L#{logical_index} ({attributes})"),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////    ProcessingElement
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Processing elements may be packages, NUMA nodes, physical cores or hardware threads (i.e.,
/// logical cores).
///
/// Each of them also carries its physical index, as assigned by the operating system and retrieved
/// by `libhwloc2-rs`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "kind", content = "id")]
pub enum ProcessingElement {
    /// Physical package (i.e., what goes into a physical socket).
    Package(u32),

    /// NUMA node (i.e., a set of processors around memory which all processors can directly access
    /// via the same physical link).
    NumaNode(u32),

    /// Physical core.
    Core(u32),

    /// Logical core (i.e., hardware thread, possibly sharing a physical core with other hardware
    /// threads).
    Thread(u32),
}

impl fmt::Display for ProcessingElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ProcessingElement::*;
        match self {
            Package(id) => write!(f, "Package P#{id}"),
            NumaNode(id) => write!(f, "NUMA node P#{id}"),
            Core(id) => write!(f, "Physical Core P#{id}"),
            Thread(id) => write!(f, "Hardware Thread P#{id}"),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////    CacheLevel
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// The cache level (e.g., L1, L2, etc).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum CacheLevel {
    /// L1 cache.
    L1,
    /// L2 cache.
    L2,
    /// L3 cache.
    L3,
    /// L4 cache.
    L4,
    /// L5 cache.
    L5,
}

impl fmt::Display for CacheLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CacheLevel::*;
        match self {
            L1 => write!(f, "L1"),
            L2 => write!(f, "L2"),
            L3 => write!(f, "L3"),
            L4 => write!(f, "L4"),
            L5 => write!(f, "L5"),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
////
////    CacheAttributes
////
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Attributes of a cache, as detected by `libhwloc2-rs`.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CacheAttributes {
    #[serde(rename = "size")]
    size: u64,
    #[serde(rename = "line")]
    linesize: u32,
    #[serde(rename = "ways")]
    associativity: i32,
}

impl CacheAttributes {
    /// Returns the total size of the cache, in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns the size of the cache line, in bytes.
    pub fn line(&self) -> u32 {
        self.linesize
    }

    /// Returns the associativity of the cache, in # ways.
    pub fn associativity(&self) -> i32 {
        self.associativity
    }
}

impl TryFrom<Option<Attributes<'_>>> for CacheAttributes {
    type Error = Error;

    fn try_from(attrs: Option<Attributes<'_>>) -> Result<Self, Self::Error> {
        match attrs {
            Some(Attributes::Cache(attrs)) => Ok(Self {
                size: attrs.size(),
                linesize: attrs.linesize(),
                associativity: attrs.associativity(),
            }),
            _ => Err(Error::NoCacheAttributes),
        }
    }
}

impl fmt::Display for CacheAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}B/{}B/{}-way",
            self.size, self.linesize, self.associativity
        )
    }
}
