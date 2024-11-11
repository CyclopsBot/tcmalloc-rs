#![deny(clippy::pedantic)]

use clap::{Parser, Subcommand};
use commands::{
    ci::{CiAnnotateArgs, CiHealthcheckArgs},
    InitArgs,
};

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    #[command(
        name = "init",
        about = "Initializes new project based upon the templates in //templates"
    )]
    Init(InitArgs),
    #[command(name = "ci", about = "Useful commands for ci ")]
    Ci {
        #[command(subcommand)]
        command: CiCommands,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum CiCommands {
    #[command(
        name = "annotate",
        about = "Annotates the job on buildkite from the .buildkite/annotate.toml"
    )]
    Annotate(CiAnnotateArgs),
    #[command(name = "healthcheck", about = "Checks CI health.")]
    Healthcheck(CiHealthcheckArgs),
}

#[derive(Clone, Debug, Parser)]
#[command(
    bin_name = "cyc",
    name = "cyclops",
    about = "Central CLI for all cyclops needs"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
