pub use crate::net::Result;
use midir::MidiOutputConnection;

pub enum Request {
    Receive, // send invitation to specified address:port
}
#[derive(Debug)]
pub enum Response {
    StartReceiving,
    Err(String),
}
pub type Responder = oneshot::Sender<Response>;

/// Minimum set of function that have to implement a Network Receiver
///
/// It is recommended to implement it as a background thread waiting for any incomming data from the distant Sender,
/// then forwarding it to the MIDI out port by a `send()` call to the provided `MidiOutputConnection` instance.
pub trait Receiver {
    /// Type used by the chosen Network Layer to describe addresses (e.g.: `SocketAddr` for TCP)
    type Addr;
    /// Define the returning value of the background thread
    type ThreadReturn;

    /// create a new Receiver instance
    ///
    /// # Arguments
    /// * `midi_out` - the MidiOut instance used to forward the receiving call to the local MIDI out port
    /// * `sender` - the distant Sender address to which the newly created Receiver have to listen for
    fn new(midi_out: MidiOutputConnection, sender: Self::Addr) -> Result<Self>
    where
        Self: Sized;

    /// start to forward the distant Sender messages to the local MIDI out port
    fn receive(self) -> Result<Self::ThreadReturn>;

    /// String describing the distant Sender address
    fn info(&self) -> String;
}
