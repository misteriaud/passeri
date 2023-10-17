pub use crate::messenger_thread::Result;
use crate::midi_thread::MidiPayload;
use std::sync::mpsc;

pub enum Request<Addr> {
    OpenRoom,           // block on listening for invitation
    AcceptClient(Addr), // initiator_token / accept invitation and forward all midi received
}
#[derive(Debug)]
pub enum Response<Addr> {
    NewClient(Addr),
    StartStream,
    ClientNotFound,
}
pub type Responder<Addr> = oneshot::Sender<Response<Addr>>;

pub trait Sender {
    type Addr;
    type ThreadReturn;

    fn new(rx: mpsc::Receiver<MidiPayload>, addr: Self::Addr) -> Result<Self>
    where
        Self: Sized;
    fn wait_for_client(&self) -> Result<Self::Addr>;
    fn info(&self) -> String;
    fn send(self, client: Self::Addr) -> Result<Self::ThreadReturn>;
}
