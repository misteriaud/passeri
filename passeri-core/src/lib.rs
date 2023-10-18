#![warn(missing_docs)]
#![doc = include_str!("../../README.md")]
mod helper;
pub use helper::*;

/// provides interfaces between OS MIDI ports and **Passeri**, it is fully relying on *midir* crate
pub mod midi;
/// defines the necessary behaviour to implement midi over network messenger
pub mod net;

// /// implements `net::Sender` and `net::Receiver` over TCP
// pub mod tcp;
