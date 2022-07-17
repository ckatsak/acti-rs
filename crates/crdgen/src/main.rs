use std::io::{self, StdoutLock, Write};

use acticrds::ActiNode;
use anyhow::{Context, Result};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::CustomResourceExt;

fn print_crd_yaml(stdout: &mut StdoutLock, crd: &CustomResourceDefinition) -> Result<()> {
    let crd_yaml = serde_yaml::to_string(&crd).with_context(|| "failed to YAML-serialize CRD")?;
    stdout
        .write_all(crd_yaml.as_bytes())
        .with_context(|| "could not write to stdout")?;
    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = io::stdout().lock();

    print_crd_yaml(&mut stdout, &ActiNode::crd())
        .with_context(|| "failed to process the CRD for ActiNode")?;

    Ok(())
}
