use std::{fmt::Debug, sync::mpsc, thread::JoinHandle};

pub use crate::net::Result;
use log::{info, trace};
use midir::MidiOutputConnection;

/// Set of requests send by the [Receiver instance](Receiver) to the [net_thread](Thread).
/// It have to be able to process all these requests to be compliant with this [Receiver instance](Receiver).
pub enum Request {
    /// start receiving from the distant sender
    Receive, // send invitation to specified address:port
}
#[derive(Debug)]

/// Set of responses that can return the [net_thread](Thread) to the [Receiver instance](Receiver) after receiving [Request].
pub enum Response {
    /// notify that [net_thread](Thread) start to receive from distant sender
    StartReceiving,
}

/// Oneshot tunnel letting the [net_thread](Thread) return [Response] to the [Receiver instance](Receiver)
pub type Responder = oneshot::Sender<Response>;

/// Packet send to the [net_thread](Thread) containing the [Request] and the [Responder]
pub type PasseriReq = (Request, Responder);

use thiserror::Error;

/// Possible [thread join()](std::thread::JoinHandle::join) return values of the [net_thread](Thread) implementation
#[derive(Error, Debug)]
pub enum ThreadReturn {
    /// unable to init Receiver
    #[error("unable to init Receiver")]
    InitError,
    /// unable to get request from tunnel
    #[error("unable to get request from tunnel")]
    Recv(#[from] mpsc::RecvError),

    /// unable to send response to tunnel
    #[error("unable to send response to tunnel")]
    Send(#[from] oneshot::SendError<Response>),

    /// unable to write to TcpStream
    #[error("unable to write to TcpStream")]
    Write(#[from] std::io::Error),

    /// unable to read from TcpStream
    #[error("unable to read from TcpStream")]
    Read(std::io::Error),

    /// unable to read from MIDI
    #[error("unable to read from MIDI")]
    MidiRecvError,

    /// unable to send to MIDI
    #[error("unable to send to MIDI")]
    MidiSendError(midir::SendError),

    /// unable to join the TCP Thread
    #[error("Join Error")]
    JoinError,

    /// Distant Sender disconnect
    #[error("Receive End")]
    ReceiveEnd,

    /// Distant Receiver disconnect
    #[error("Send End")]
    SendEnd,
}

//
//	ReceiverThread trait
//

/// Minimum set of function that have to implement a [net_thread](Thread)
///
/// It is recommended to implement it as a background thread waiting for any incomming data from the distant Sender,
/// then forwarding it to the MIDI out port by a `send()` call to the provided `MidiOutputConnection` instance.
pub trait Thread {
    /// Type used by the chosen Network Layer to describe addresses (e.g.: `SocketAddr` for TCP)
    type Addr: 'static + Send;

    /// create a new Receiver instance
    ///
    /// # Arguments
    /// * `addr` - the distant Sender address to which the newly created **ReceiverThread** have to listen for
    /// * `midi_tx` - the [MidiOutputConnection] instance used to forward the receiving call to the local MIDI out port
    /// * `messenger_rx` - [Receiver](mpsc::Receiver) from which the **ReceiverThread** will get [Request] from the main thread
    fn new(
        addr: Self::Addr,
        midi_tx: MidiOutputConnection,
        messenger_rx: mpsc::Receiver<PasseriReq>,
    ) -> std::result::Result<Self, String>
    where
        Self: Sized;

    /// implementation have to block reading on the `messenger_rx` [Receiver](mpsc::Receiver), processing each incomming [Request]
    fn run(&mut self) -> std::result::Result<(), ThreadReturn>;

    /// implementation have to start forwarding incomming [crate::midi::MidiFrame] from the distant sender to the local midi_thread using `midi_tx` [MidiOutputConnection].
    /// It have to notify the main thread that the receiving stream is starting by a [Response::StartReceiving] [Response] and then looping over this way:
    /// 	- blocking on reading the incomming [crate::midi::MidiFrame] from the distant sender
    /// 	- forwarding received message to the local midi_thread using `midi_tx` [MidiOutputConnection]
    fn receive(&mut self, responder: Responder) -> std::result::Result<(), ThreadReturn>;

    /// String describing the distant Sender address
    fn info(&self) -> String;
}

//
//	Receiver<T> implementation
//

/// [Receiver instance](Receiver) used to bridge an incomming network stream (implemented by [net_thread](Thread)) to an output MIDI port
pub struct Receiver {
    net_thread: Option<JoinHandle<ThreadReturn>>,
    tx: mpsc::Sender<PasseriReq>,
    addr: String,
}

impl Receiver {
    /// Create a new [Receiver instance](Receiver) (it is recommended to use the [new_receiver()][crate::new_receiver] function)
    pub fn new<T: Thread>(midi_tx: MidiOutputConnection, addr: T::Addr) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<PasseriReq>();
        let (init_tx, init_rx) = oneshot::channel::<std::result::Result<String, String>>();

        let net_thread = Some(std::thread::spawn(|| {
            let mut socket = match T::new(addr, midi_tx, rx) {
                Ok(res) => {
                    init_tx.send(Ok(res.info())).unwrap();
                    res
                }
                Err(err) => {
                    init_tx.send(Err(err)).unwrap();
                    return ThreadReturn::InitError;
                }
            };

            info!("receiver created on {}", socket.info());

            socket.run().unwrap_err()
        }));

        let addr = init_rx.recv()??;

        Ok(Receiver {
            net_thread,
            tx,
            addr,
        })
    }

    /// Start forwarding network stream from [net_thread](Thread) to output MIDI port
    pub fn receive(&self) -> Result<()> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::Receive, response_sender))?;

        match response_receiver.recv()? {
            Response::StartReceiving => {
                trace!("received ListenStream");
                Ok(())
            }
        }
    }

    pub fn join(&mut self) -> Result<ThreadReturn> {
        Ok(self
            .net_thread
            .take()
            .unwrap()
            .join()
            .unwrap_or(ThreadReturn::JoinError))
    }

    pub fn info(&self) -> String {
        self.addr.clone()
    }
}
