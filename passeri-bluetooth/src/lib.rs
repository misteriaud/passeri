#![warn(missing_docs)]
//! Implementation of the Sender and Receiver traits from `passeri-core`

use oneshot::SendError;
use std::sync::mpsc::RecvError;
use thiserror::Error;

mod receiver;
pub use receiver::Receiver;

mod connection;
// pub use connection::
// mod sender;
// pub use sender::Sender;

#[derive(Error, Debug)]
pub enum ThreadReturn<Response> {
    #[error("Bluetooth Error")]
    BleErr(btleplug::Error),

    #[error("No available matching Bluetooth pair")]
    NoMatchingClient,
}
