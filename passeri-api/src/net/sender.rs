use crate::midi::MidiPayload;
pub use crate::net::Result;
use log::{error, info, trace};
use midir::MidiInputConnection;
use std::{
    fmt::Debug,
    sync::mpsc::{self},
    thread::JoinHandle,
};

/// Set of requests send by the [Sender instance](Sender) to the [net_thread](Thread).
/// It have to be able to process all these requests to be compliant with this [Sender instance](Sender).
pub enum Request<Addr> {
    /// start listening on the provided address for potential receiver client
    OpenRoom,
    /// start to stream [crate::midi::MidiFrame] to the given address (obtained by the `OpenRoom` request)
    AcceptClient(Addr),
}

/// Set of responses that can return the [net_thread](Thread) to the [Sender instance](Sender) after receiving [Request].
#[derive(Debug)]
pub enum Response<Addr> {
    /// received a new potential receiver client
    NewClient(Addr),
    /// notify that sender thread start to stream incomming MIDI to receiver client
    StartStream,
    /// response that have to be return in case of a [Request::AcceptClient] request before a [Request::OpenRoom] one
    ClientNotFound,
}

/// Oneshot tunnel letting the [net_thread](Thread) return [Response] to the [Sender instance](Sender)
pub type Responder<Addr> = oneshot::Sender<Response<Addr>>;

use thiserror::Error;

/// Possible [thread join()](std::thread::JoinHandle::join) return values of the [net_thread](Thread) implementation
#[derive(Error, Debug)]
pub enum ThreadReturn<Addr> {
    /// unable to init Sender
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

/// Packet send to the [net_thread](Thread) containing the [Request] and the [Responder]
pub type PasseriReq<Addr> = (Request<Addr>, Responder<Addr>);

/// Minimum set of function that have to implement a [net_thread](Thread)
///
/// It is recommended to implement it as a background thread waiting for any incomming MIDI message from the `midi_rx` [mpsc::Receiver],
/// then sending it over network to the connected receiver client.
pub trait Thread {
    /// Type used by the chosen Network Layer to describe addresses (e.g.: `SocketAddr` for TCP)
    type Addr: 'static + Send + Debug;

    /// create a new Sender instance
    ///
    /// # Arguments
    /// * `addr` -		the address on which the Network Layer have to bind to
    /// * `midi_rx` -	[Receiver](mpsc::Receiver) from which the **SenderThread** will get timestamp and
    /// 				[MidiFrame](crate::midi::MidiFrame) received by the midi thread
    /// * `messenger_rx` - [Receiver](mpsc::Receiver) from which the **SenderThread** will get [Request] from the main thread
    fn new(
        addr: Self::Addr,
        midi_rx: mpsc::Receiver<MidiPayload>,
        messenger_rx: mpsc::Receiver<PasseriReq<Self::Addr>>,
    ) -> std::result::Result<Self, String>
    where
        Self: Sized;

    /// implementation have to block reading on the `messenger_rx` [Receiver](mpsc::Receiver), processing each incomming [Request]
    fn run(&mut self) -> std::result::Result<(), ThreadReturn<Self::Addr>>;

    /// implementation have to start forwarding local MIDI message to connected receiver client.
    /// It have to notify the main thread that the stream is starting by a [Response::StartStream] [Response] and then looping over this way:
    /// 	- blocking on reading `midi_rx` [Receiver](mpsc::Receiver)
    /// 	- forwarding received message to the receiver client
    fn send(
        &mut self,
        distant: Self::Addr,
        responder: Responder<Self::Addr>,
    ) -> std::result::Result<(), ThreadReturn<Self::Addr>>;

    /// return a informationnal string on the address on which is bound the sender thread
    fn info(&self) -> String;
}

//
//	Sender<T> implementation
//

/// [Sender instance](Sender) used to bridge local MIDI messages to distant receiver over network (implemented by [net_thread](Thread))
pub struct Sender<T: Thread> {
    _midi_thread: MidiInputConnection<()>,
    net_thread: JoinHandle<ThreadReturn<T::Addr>>,
    tx: mpsc::Sender<PasseriReq<T::Addr>>,
}
impl<T: Thread> Sender<T> {
    /// Create a new [Sender instance](Sender) (it is recommended to use the [new_sender()][crate::new_sender] function)
    pub fn new(
        _midi_thread: MidiInputConnection<()>,
        midi_rx: mpsc::Receiver<MidiPayload>,
        addr: T::Addr,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<PasseriReq<T::Addr>>();
        let (init_tx, init_rx) = oneshot::channel::<std::result::Result<(), String>>();

        // let socket_addr = socket.get_addr();

        let net_thread = std::thread::spawn(move || {
            let mut socket = match T::new(addr, midi_rx, rx) {
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

        Ok(Sender {
            _midi_thread,
            net_thread,
            tx,
        })
    }

    /// listen for possible distant receiver client
    pub fn wait_for_client(&self) -> Result<T::Addr> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::OpenRoom, response_sender))?;

        match response_receiver.recv()? {
            Response::NewClient(addr) => Ok(addr),
            _ => Err("invalid response from tcp_thread".into()),
        }
    }

    /// Start forwarding local MIDI messages to distant receiver over network
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
