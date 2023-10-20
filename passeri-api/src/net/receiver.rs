use std::{
    fmt::{Debug, Display},
    sync::mpsc,
    thread::JoinHandle,
};

pub use crate::net::Result;
use log::info;
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

type PasseriReq = (Request, Responder);

use thiserror::Error;

/// Possible thread return values of the TCP Sender and Receiver
#[derive(Error, Debug)]
pub enum ThreadReturn {
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

pub trait ReceiverThread {
    type Addr: 'static + Send;

    fn new(
        addr: Self::Addr,
        midi_tx: MidiOutputConnection,
        messenger_rx: mpsc::Receiver<PasseriReq>,
    ) -> std::result::Result<Self, String>
    where
        Self: Sized;
    fn run(&mut self) -> std::result::Result<(), ThreadReturn>;
    fn receive(&mut self, responder: Responder) -> std::result::Result<(), ThreadReturn>;
    fn info(&self) -> String;
}

//
//	Receiver<T> implementation
//

pub struct Receiver {
    net_thread: JoinHandle<ThreadReturn>,
    tx: mpsc::Sender<PasseriReq>,
    // socket_addr: Option<SocketAddr>,
}

impl Receiver {
    pub fn new<T: ReceiverThread>(midi_tx: MidiOutputConnection, addr: T::Addr) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<PasseriReq>();
        let (init_tx, init_rx) = oneshot::channel::<std::result::Result<(), String>>();

        let net_thread = std::thread::spawn(|| {
            let mut socket = match T::new(addr, midi_tx, rx) {
                Ok(res) => {
                    init_tx.send(Ok(())).unwrap();
                    res
                }
                Err(err) => {
                    init_tx.send(Err(err)).unwrap();
                    return ThreadReturn::InitError;
                }
            };

            info!("{}", socket.info());

            socket.run().unwrap_err()
        });

        init_rx.recv()??;

        Ok(Receiver { net_thread, tx })
    }

    pub fn receive(self) -> Result<ThreadReturn> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::Receive, response_sender))?;

        match response_receiver.recv()? {
            Response::StartReceiving => {
                info!("received ListenStream");
                Ok(self.net_thread.join().unwrap_or(ThreadReturn::JoinError))
            }
            Response::Err(err) => Err(err.into()),
        }
    }

    // fn info(&self) -> String {
    //     match self.socket_addr {
    //         Some(addr) => format!("addr1: {}", addr),
    //         None => String::new(),
    //     }
    // }
}

// /// Minimum set of function that have to implement a Network Receiver
// ///
// /// It is recommended to implement it as a background thread waiting for any incomming data from the distant Sender,
// /// then forwarding it to the MIDI out port by a `send()` call to the provided `MidiOutputConnection` instance.
// pub trait Receiver {
//     /// Type used by the chosen Network Layer to describe addresses (e.g.: `SocketAddr` for TCP)
//     type Addr;
//     /// Define the returning value of the background thread
//     type ThreadReturn;

//     /// create a new Receiver instance
//     ///
//     /// # Arguments
//     /// * `midi_out` - the MidiOut instance used to forward the receiving call to the local MIDI out port
//     /// * `sender` - the distant Sender address to which the newly created Receiver have to listen for
//     fn new(midi_out: MidiOutputConnection, sender: Self::Addr) -> Result<Self>
//     where
//         Self: Sized;

//     /// start to forward the distant Sender messages to the local MIDI out port
//     fn receive(self) -> Result<Self::ThreadReturn>;

//     /// String describing the distant Sender address
//     fn info(&self) -> String;
// }
