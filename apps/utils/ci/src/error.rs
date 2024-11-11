#![deny(clippy::pedantic)]

use miette::Diagnostic;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug, Diagnostic)]
pub enum CyclopsAppError {
    #[diagnostic(code(invalid_root))]
    #[error("Invalid Workspace Root. Wrong directory?")]
    InvalidWorkspaceRoot,
}
