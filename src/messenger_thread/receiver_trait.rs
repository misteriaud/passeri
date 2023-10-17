pub use crate::messenger_thread::Result;
use midir::MidiOutputConnection;

pub enum Request {
    Receive, // send invitation to specified address:port
}
pub enum Response {
    StartReceiving,
    Err(String),
}
pub type Responder = oneshot::Sender<Response>;

pub trait NetReceiver {
    type Addr;
    fn new(midi_out: MidiOutputConnection, sender: Self::Addr) -> Result<Self>
    where
        Self: Sized;
    fn receive(&self) -> Result<()>;
    fn info(&self) -> String;
}
