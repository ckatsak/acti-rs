use std::{
    collections::{btree_map, BTreeMap},
    env,
};

use anyhow::{Context, Result};
use kube::{Api, Client};
use tracing::{info, instrument, trace, Level};
use validator::Validate;

use acticrds::ActiNode;
use actitopo::{DetectionMode, Topology};

use crate::{Args, Mode};

//
// Values for Kubernetes' "recommended labels"
//
const APP_K8S_IO_NAME: &str = "acti-system";
//const APP_K8S_IO_INSTANCE: &str = env!("ACTI_NODE_NAME");
const APP_K8S_IO_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_K8S_IO_COMPONENT: &str = "actinodes";
const APP_K8S_IO_PART_OF: &str = "actik8s";

//
// ActiK8s annotations' keys
//
const ACTI_FULL_TOPO_ANNOTATION_KEY: &str = "acti.cslab.ece.ntua.gr/full-topology";
const ACTI_PART_TOPO_ANNOTATION_KEY: &str = "acti.cslab.ece.ntua.gr/partial-topology";

//
// Environment variables expected to be set at runtime by CRI
//
const ACTI_K8S_NODE_NAME_ENV: &str = "ACTI_NODE_NAME";
const ACTI_K8S_NAMESPACE_ENV: &str = "ACTI_NAMESPACE";

#[derive(Debug, Clone)]
pub struct Registrant {
    mode: Mode,
    node_name: String,
    namespace: String,
}

impl Registrant {
    #[instrument(level = Level::DEBUG)]
    pub fn new(args: Args) -> Result<Self> {
        Ok(Self {
            mode: args.mode,
            node_name: env::var(ACTI_K8S_NODE_NAME_ENV).with_context(|| {
                format!("environment variable {ACTI_K8S_NODE_NAME_ENV:?} not found",)
            })?,
            namespace: env::var(ACTI_K8S_NAMESPACE_ENV)
                .with_context(|| format!("environment variable {ACTI_K8S_NAMESPACE_ENV:?}",))?,
        })
    }

    /// Detects and returns the full and partial (respectively) hardware topology of the physical
    /// node where we are running on.
    #[instrument(level = Level::DEBUG, skip(self))]
    fn detect_topology(&self) -> Result<(Option<Topology>, Option<Topology>)> {
        let full = || {
            Topology::detect(DetectionMode::Full)
                .with_context(|| "failed to detect the full underlying hardware topology")
        };
        let partial = || {
            Topology::detect(DetectionMode::IsolationBoundariesOnly)
                .with_context(|| "failed to detect the partial underlying hardware topology")
        };
        Ok(match self.mode {
            Mode::Full => (Some(full()?), None),
            Mode::Partial => (None, Some(partial()?)),
            Mode::All => (Some(full()?), Some(partial()?)),
        })
    }

    /// Allocates, properly initializes and returns a (local, in-memory) `ActiNode`.
    #[instrument(level = Level::DEBUG, skip(self, acti_annotations))]
    fn init_actinode(&self, acti_annotations: ActiAnnotations) -> Result<ActiNode> {
        let mut an = ActiNode::new(self.node_name.as_str(), Default::default());
        an.metadata.namespace = Some(self.namespace.clone());
        an.metadata
            .labels
            .get_or_insert_with(Default::default)
            .extend(ActiLabels::new(self.node_name.as_str()).into_iter());
        an.metadata
            .annotations
            .get_or_insert_with(Default::default)
            .extend(acti_annotations.into_iter());
        an.status = Some(Default::default());

        an.spec
            .validate()
            .with_context(|| "failed to validate local ActiNode struct (BUG)")?;
        Ok(an)
    }

    /// Register the provided `ActiNode` with the Kubernetes API server.
    #[instrument(level = Level::DEBUG, skip(self, actinode))]
    async fn register_node(&self, actinode: ActiNode) -> Result<()> {
        // Initialize a new Kubernetes client
        let klient = Client::try_default()
            .await
            .with_context(|| "failed to initialize kubernetes client")?;
        // Initialize a new Kubernetes client for our ActiNode API Object
        let actinodes = Api::namespaced(klient, &self.namespace);

        // Contact API server to create the upstream ActiNode Object
        let upstream_an = actinodes
            .create(&Default::default(), &actinode)
            .await
            .with_context(|| "failed to create new ActiNode K8s API Object")?;

        // Log success
        let ns = upstream_an.metadata.namespace.as_ref();
        let name = upstream_an.metadata.name.as_ref();
        info!(
            "Created new ActiNode API Object '{}/{}'",
            ns.expect("upstream ActiNode Object's namespace is None"),
            name.expect("upstream ActiNode Object's name is None")
        );
        trace!("Upstream ActiNode K8s API Object: {upstream_an:#?}");

        Ok(())
    }

    /// `Registrant`'s entry point.
    #[instrument(level = Level::DEBUG)]
    pub async fn run(self) -> Result<()> {
        let actinode = self
            .detect_topology()
            .with_context(|| "failed to detect hardware topology")?
            .try_into()
            .with_context(|| "could not convert Topology objects into ActiAnnotations")
            .and_then(|acti_annotations| self.init_actinode(acti_annotations))
            .with_context(|| "failed to initialize local ActiNode struct")?;
        self.register_node(actinode)
            .await
            .with_context(|| "failed registering new ActiNode with Kubernetes")
    }
}

#[derive(Debug, Default, Clone)]
struct ActiAnnotations(BTreeMap<String, String>);

impl TryFrom<(Option<Topology>, Option<Topology>)> for ActiAnnotations {
    type Error = anyhow::Error;

    fn try_from(
        (full_topo, partial_topo): (Option<Topology>, Option<Topology>),
    ) -> Result<Self, Self::Error> {
        let mut ret = BTreeMap::new();
        if let Some(full) = full_topo {
            let full = serde_json::to_string(&full)
                .with_context(|| "could not serialize Topology (full)")?;
            let _ = ret.insert(ACTI_FULL_TOPO_ANNOTATION_KEY.to_owned(), full);
        }
        if let Some(partial) = partial_topo {
            let partial = serde_json::to_string(&partial)
                .with_context(|| "could not serialize Topology (partial)")?;
            let _ = ret.insert(ACTI_PART_TOPO_ANNOTATION_KEY.to_owned(), partial);
        }
        Ok(Self(ret))
    }
}

impl IntoIterator for ActiAnnotations {
    type Item = (String, String);
    type IntoIter = btree_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

struct ActiLabels(BTreeMap<String, String>);

impl ActiLabels {
    fn new(instance: &str) -> Self {
        Self(BTreeMap::from_iter(
            [
                (
                    "app.kubernetes.io/name".to_owned(),
                    APP_K8S_IO_NAME.to_owned(),
                ),
                ("app.kubernetes.io/instance".to_owned(), instance.to_owned()),
                (
                    "app.kubernetes.io/version".to_owned(),
                    APP_K8S_IO_VERSION.to_owned(),
                ),
                (
                    "app.kubernetes.io/component".to_owned(),
                    APP_K8S_IO_COMPONENT.to_owned(),
                ),
                (
                    "app.kubernetes.io/part-of".to_owned(),
                    APP_K8S_IO_PART_OF.to_owned(),
                ),
            ]
            .into_iter(),
        ))
    }
}

impl IntoIterator for ActiLabels {
    type Item = (String, String);
    type IntoIter = btree_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
