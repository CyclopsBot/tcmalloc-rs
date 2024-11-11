#![allow(clippy::missing_panics_doc)]
use clap::Args;
use serde::Deserialize;
use std::{io, io::Write};
use tracing::info;

#[derive(Deserialize)]
struct Annotation {
    style: String,
    string: String,
}

#[derive(Args, Clone, Debug)]
pub struct CiAnnotateArgs {}

pub fn annotate(_args: &CiAnnotateArgs) {
    info!("Attempting to annotate.");

    let file = std::fs::read_to_string(".buildkite/annotation.toml")
        .expect("Unable to fetch annotation file falling back to default");

    let annotation: Annotation = toml::from_str(&file).unwrap();

    let style = format!("--style={}", annotation.style);

    let annotate = std::process::Command::new("buildkite-agent")
        .args(["annotate", &annotation.string, &style])
        .output()
        .expect("unable to annotate");

    io::stdout().write_all(&annotate.stdout).unwrap();
}
