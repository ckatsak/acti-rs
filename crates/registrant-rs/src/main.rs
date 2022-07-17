mod registrant;

use std::{io, str::FromStr};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

use registrant::Registrant;

#[derive(Debug, Default, Parser, Clone, Copy)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Args {
    /// Passing 'full' or 'partial' determines the detection mode for the hardware topology. Any
    /// other value is interpreted as 'all'.
    #[clap(short = 'm', long = "mode", required = false, default_value = "all")]
    pub mode: Mode,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Mode {
    #[default]
    All,
    Full,
    Partial,
}

impl FromStr for Mode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "full" => Self::Full,
            "partial" => Self::Partial,
            _ => Self::All,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE)
        .try_init()
        .map_err(|e| anyhow!("Failed to initialize logger: {e}"))?;
    Registrant::new(Args::parse())
        .with_context(|| "could not initialize Registrant")?
        .run()
        .await
        .with_context(|| "failed registering with Kubernetes")
}
