use crate::midi::MidiPayload;
pub use crate::net::Result;
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

/// Minimum set of function that have to implement a Network Sender
///
/// It is recommended to implement it as a background thread waiting for any incomming MIDI message from the `mpsc::Receiver`,
/// then sending it over network to the connected Receiver client.
pub trait Sender {
    /// Type used by the chosen Network Layer to describe addresses (e.g.: `SocketAddr` for TCP)
    type Addr;
    /// Define the returning value of the background thread
    type ThreadReturn;

    /// create a new Sender instance
    ///
    /// # Arguments
    /// * `rx` - the receiving end of the tunnel used by the MIDI in thread to forward incomming MIDI messages
    /// * `addr` - the address on which the Network Layer have to bind to
    fn new(rx: mpsc::Receiver<MidiPayload>, addr: Self::Addr) -> Result<Self>
    where
        Self: Sized;

    /// waiting for any client trying to connect to the Sender
    fn wait_for_client(&self) -> Result<Self::Addr>;

    /// start to forward the local MIDI In message to the distant Receiver
    fn send(self, client: Self::Addr) -> Result<Self::ThreadReturn>;

    /// String describing the address used by the Network Layer
    fn info(&self) -> String;
}
