#![warn(missing_docs)]
//! Implementation of the Sender and Receiver traits from `passeri-api`

mod tcp_receiver;
pub use tcp_receiver::Receiver;
mod tcp_sender;
pub use tcp_sender::Sender;
