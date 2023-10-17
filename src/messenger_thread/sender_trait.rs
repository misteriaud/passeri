pub use crate::messenger_thread::Result;
use crate::midi_thread::MidiPayload;
use std::sync::mpsc;

pub enum Request {
    OpenRoom,     // block on listening for invitation
    AcceptClient, // initiator_token / accept invitation and forward all midi received
}
pub enum Response<Addr> {
    NewClient(Addr),
    StartStream,
    Err(String),
}
pub type Responder<Addr> = oneshot::Sender<Response<Addr>>;

pub trait NetSender {
    type Addr;
    fn new(rx: mpsc::Receiver<MidiPayload>, addr: Self::Addr) -> Result<Self>
    where
        Self: Sized;
    fn wait_for_client(&self) -> Result<Self::Addr>;
    fn info(&self) -> String;
    fn send(self, client: Self::Addr) -> Result<()>;
}
