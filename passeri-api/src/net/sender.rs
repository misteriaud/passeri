use crate::midi::MidiPayload;
pub use crate::net::Result;
use log::{error, info, trace};
use midir::MidiInputConnection;
use std::{
    fmt::{Debug, Display},
    sync::mpsc::{self},
    thread::JoinHandle,
};

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

use thiserror::Error;

/// Possible thread return values of the TCP Sender and Receiver
#[derive(Error, Debug)]
pub enum ThreadReturn<Addr> {
    #[error("unable to init Sender")]
    InitError,
    /// unable to get request from tunnel
    #[error("unable to get request from tunnel")]
    Recv(#[from] mpsc::RecvError),

    /// unable to send response to tunnel
    #[error("unable to send response to tunnel")]
    Send(#[from] oneshot::SendError<Response<Addr>>),

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
//	SenderThread definition
//
pub type PasseriReq<Addr> = (Request<Addr>, Responder<Addr>);

pub trait SenderThread {
    type Addr: 'static + Send + Debug;

    fn new(
        addr: Self::Addr,
        midi_rx: mpsc::Receiver<MidiPayload>,
        messenger_rx: mpsc::Receiver<PasseriReq<Self::Addr>>,
    ) -> Result<Self>
    where
        Self: Sized;

    fn run(&mut self) -> std::result::Result<(), ThreadReturn<Self::Addr>>;
    fn send(
        &mut self,
        distant: Self::Addr,
        responder: Responder<Self::Addr>,
    ) -> std::result::Result<(), ThreadReturn<Self::Addr>>;
    fn info(&self) -> String;
}

//
//	Sender<T> implementation
//

pub struct Sender<T: SenderThread> {
    _midi_thread: MidiInputConnection<()>,
    net_thread: JoinHandle<ThreadReturn<T::Addr>>,
    tx: mpsc::Sender<PasseriReq<T::Addr>>,
}

impl<T: SenderThread> Sender<T> {
    // type Addr = SocketAddr;
    // type ThreadReturn = ThreadReturn<Response>;

    pub fn new(
        _midi_thread: MidiInputConnection<()>,
        midi_rx: mpsc::Receiver<MidiPayload>,
        addr: T::Addr,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<PasseriReq<T::Addr>>();

        // let socket_addr = socket.get_addr();

        let net_thread = std::thread::spawn(|| {
            let Ok(mut socket) = T::new(addr, midi_rx, rx) else {
                return ThreadReturn::InitError;
            };

            info!("{}", socket.info());

            socket.run().unwrap_err()
        });

        Ok(Sender {
            _midi_thread,
            net_thread,
            tx,
        })
    }

    pub fn wait_for_client(&self) -> Result<T::Addr> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::OpenRoom, response_sender))?;

        match response_receiver.recv()? {
            Response::NewClient(addr) => Ok(addr),
            _ => Err("invalid response from tcp_thread".into()),
        }
    }
    pub fn send(self, client: T::Addr) -> Result<ThreadReturn<T::Addr>> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx
            .send((Request::AcceptClient(client), response_sender))?;

        match response_receiver.recv()? {
            Response::StartStream => {
                trace!("received StartStream");
                Ok(self.net_thread.join().unwrap_or(ThreadReturn::JoinError))
            }
            _ => Err("invalid response from tcp_thread".into()),
        }
    }

    // pub fn info(&self) -> String {
    //     match self.addr {
    //         Some(addr) => format!("addr: {:?}", addr),
    //         None => String::new(),
    //     }
    // }
}

// /// Minimum set of function that have to implement a Network Sender
// ///
// /// It is recommended to implement it as a background thread waiting for any incomming MIDI message from the `mpsc::Receiver`,
// /// then sending it over network to the connected Receiver client.
// pub trait Sender {
//     /// Type used by the chosen Network Layer to describe addresses (e.g.: `SocketAddr` for TCP)
//     type Addr;
//     /// Define the returning value of the background thread
//     type ThreadReturn;

//     /// create a new Sender instance
//     ///
//     /// # Arguments
//     /// * `rx` - the receiving end of the tunnel used by the MIDI in thread to forward incomming MIDI messages
//     /// * `addr` - the address on which the Network Layer have to bind to
//     fn new(rx: mpsc::Receiver<MidiPayload>, addr: Self::Addr) -> Result<Self>
//     where
//         Self: Sized;

//     /// waiting for any client trying to connect to the Sender
//     fn wait_for_client(&self) -> Result<Self::Addr>;

//     /// start to forward the local MIDI In message to the distant Receiver
//     fn send(self, client: Self::Addr) -> Result<Self::ThreadReturn>;

//     /// String describing the address used by the Network Layer
//     fn info(&self) -> String;
// }
