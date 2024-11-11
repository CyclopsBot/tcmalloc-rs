#![deny(clippy::pedantic)]
#[allow(clippy::module_name_repetitions)]
pub mod init;
pub mod ci {
    pub use commands_ci::*;
}

pub use init::*;
