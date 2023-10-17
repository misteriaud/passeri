pub mod receiver;
use std::sync::mpsc::RecvError;

use oneshot::SendError;
pub use receiver::TcpReceiver;
pub mod sender;
pub use sender::TcpSender;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadReturn<Response> {
    #[error("unable to get request from tunnel")]
    Recv(#[from] RecvError),
    #[error("unable to send response to tunnel")]
    Send(#[from] SendError<Response>),
    #[error("unable to write to TcpStream")]
    Write(#[from] std::io::Error),
    #[error("unable to read from TcpStream")]
    Read(std::io::Error),
    #[error("unable to read from MIDI")]
    MidiRecvError,
    #[error("unable to send to MIDI")]
    MidiSendError(midir::SendError),
    #[error("Join Error")]
    JoinError,
    #[error("Receive End")]
    ReceiveEnd,
    #[error("Send End")]
    SendEnd,
}
