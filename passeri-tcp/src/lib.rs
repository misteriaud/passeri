#![warn(missing_docs)]
//! Implementation of the Sender and Receiver traits from `passeri-api`

use std::sync::mpsc::RecvError;

mod receiver;
pub use receiver::Receiver;
mod sender;
pub use sender::Sender;
// pub use sender::Sender;

// /// Possible thread return values of the TCP Sender and Receiver
// #[derive(Error, Debug)]
// pub enum ThreadReturn<Response> {
//     /// unable to get request from tunnel
//     #[error("unable to get request from tunnel")]
//     Recv(#[from] RecvError),

//     /// unable to send response to tunnel
//     #[error("unable to send response to tunnel")]
//     Send(#[from] SendError<Response>),

//     /// unable to write to TcpStream
//     #[error("unable to write to TcpStream")]
//     Write(#[from] std::io::Error),

//     /// unable to read from TcpStream
//     #[error("unable to read from TcpStream")]
//     Read(std::io::Error),

//     /// unable to read from MIDI
//     #[error("unable to read from MIDI")]
//     MidiRecvError,

//     /// unable to send to MIDI
//     #[error("unable to send to MIDI")]
//     MidiSendError(midir::SendError),

//     /// unable to join the TCP Thread
//     #[error("Join Error")]
//     JoinError,

//     /// Distant Sender disconnect
//     #[error("Receive End")]
//     ReceiveEnd,

//     /// Distant Receiver disconnect
//     #[error("Send End")]
//     SendEnd,
// }
