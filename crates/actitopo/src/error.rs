/// An error type returned by calls to the API exposed by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Indicates that this `hwloc2::Object` does not correspond to any element defined in
    //       TODO(ckatsak): ^^^^^^^^^^^^^^^^
    /// Acti-topology terms.
    #[error("No equivalent actitopo::Element for this hwloc2::Object")]
    NoEquivalentElement,

    /// Returned when the `hwloc2::Object` under examination does not appear to be associated
    /// with a `hwloc2::object::attributes::CacheAttributes`.
    //         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TODO(ckatsak)
    #[error("No cache attributes found in this hwloc2::Object")]
    NoCacheAttributes,

    /// Returned when the [`Topology`] is empty, while it shouldn't be.
    ///
    /// [`Topology`]: crate::Topology
    #[error("Topology appears empty, but it should not be")]
    EmptyTopology,

    /// Returned when an `hwloc2::Object`'s memory arity is found to be greater than 1, which is
    /// currently not supported by this crate.
    #[error("A topology object's memory arity equals {0}, which is > 1, thus unsupported")]
    MemoryArity(u32),

    /// Error emanating from the [`immutree`] crate.
    #[error("Tree Error: {source}")]
    ImmuTree {
        #[from]
        source: immutree::Error,
    },

    /// Error emanating from `libhwloc2-rs`.
    #[error("libhwloc2-rs Error: {source}")]
    Hwloc {
        #[from]
        source: hwloc2::Error,
    },
}
