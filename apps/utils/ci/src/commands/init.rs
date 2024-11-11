use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct InitArgs {}

pub fn init(_args: &InitArgs) {
    tracing::info!("balls hahgadksl");
}
