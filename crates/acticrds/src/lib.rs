use std::collections::HashMap;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// ActiNodeSpec defines the desired state of an ActiNode.
#[derive(
    CustomResource, Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema, Validate,
)]
#[kube(
    group = "acti.cslab.ece.ntua.gr",
    version = "v1alpha1",
    kind = "ActiNode",
    namespaced,
    status = "ActiNodeStatus",
    derive = "PartialEq",
    derive = "Default",
    shortname = "an",
    shortname = "actin",
    shortname = "anode"
)]
#[serde(rename_all = "camelCase")]
pub struct ActiNodeSpec {
    /// Assignments include the Pods that are executed on the Node related to an ActiNode, along
    /// with the OS indices of the cores where each of them is pinned.
    pub assignments: HashMap<String, Vec<u32>>,
}

/// ActiNodeStatus describes the observed state of an ActiNode.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActiNodeStatus {
    /// Pinnings include the actual assignments of Pods to physical cores, as observed (and
    /// enforced) by ActiK8s' `internal` controller.
    pub pinnings: HashMap<String, Vec<u32>>,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use kube::CustomResourceExt;
    use validator::Validate;

    use super::ActiNode;

    #[test]
    fn print_an_crd_yaml() {
        let an = ActiNode::crd();
        eprintln!(
            "CRD:\n{}",
            serde_yaml::to_string(&an).expect("failed to YAML-serialize ActiNode CRD")
        );
    }

    #[test]
    fn print_default_an_yaml() {
        let an = ActiNode::default();
        eprintln!(
            "ActiNode (default):\n{}",
            serde_yaml::to_string(&an).expect("failed to YAML-serialize ActiNode")
        );
    }

    #[test]
    fn initialize_an() -> Result<()> {
        let mut an = ActiNode::new(
            "initialize-an",
            Default::default(),
            //ActiNodeSpec {
            //    assignments: HashMap::new(),
            //},
        );
        an.metadata.namespace = Some("koko_ns".to_owned());
        an.metadata
            .annotations
            //.insert([("yolo".to_owned(), "re".to_owned())].into_iter().collect());
            .get_or_insert_with(Default::default)
            .extend([("yolo".to_owned(), "re".to_owned())].into_iter());
        an.status = Some(Default::default());
        assert!(an.spec.validate().is_ok());

        eprintln!("new an:\n{an:#?}\n");
        eprintln!(
            "serialized an:\n{}\n",
            serde_yaml::to_string(&an).expect("failed to YAML-serialize ActiNode")
        );

        Ok(())
    }

    #[test]
    fn crd_yaml() -> Result<()> {
        let crd = serde_yaml::to_string(&ActiNode::crd())
            .expect("failed to YAML-serialize the CRD for ActiNode");
        eprintln!("{crd}");
        Ok(())
    }
}
