#![warn(missing_docs)]
//! Implementation of the Sender and Receiver traits from `passeri-core`

use oneshot::SendError;
use std::sync::mpsc::RecvError;
use thiserror::Error;

mod receiver;
pub use receiver::Receiver;
// mod sender;
// pub use sender::Sender;
