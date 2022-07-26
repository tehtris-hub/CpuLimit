use thiserror::Error;

use crate::limiter::Command;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The target process is dead")]
    DeadTarget,
    #[error("Couldn't spawn the limiting thread")]
    Spawn(#[from] std::io::Error),
    #[error("Couldn't send command to the limiting thread")]
    Send(#[from] std::sync::mpsc::SendError<Command>),
}

pub type Result<T> = core::result::Result<T, Error>;
