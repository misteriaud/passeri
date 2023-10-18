use crate::midi::MidiFrame;
use crate::net::receiver::{self, Receiver, Request, Responder, Response};
use log::{info, trace};
use midir::MidiOutputConnection;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread::JoinHandle;

type RTPPayload = (Request, Responder);

use crate::net::Result;
use std::io::Read;

use super::ThreadReturn;

pub struct TcpReceiver {
    thread: JoinHandle<ThreadReturn<Response>>,
    tx: mpsc::Sender<RTPPayload>,
    socket_addr: Option<SocketAddr>,
}

impl Receiver for TcpReceiver {
    type Addr = SocketAddr;
    type ThreadReturn = ThreadReturn<Response>;

    fn new(midi_out: MidiOutputConnection, addr: Self::Addr) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<RTPPayload>();

        let mut socket = ReceiverThread::new(midi_out, rx, addr)?;

        let thread = std::thread::spawn(move || socket.run().unwrap_err());

        Ok(TcpReceiver {
            thread,
            tx,
            socket_addr: None,
        })
    }

    fn receive(self) -> Result<Self::ThreadReturn> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::Receive, response_sender))?;

        match response_receiver.recv()? {
            receiver::Response::StartReceiving => {
                info!("received ListenStream");
                Ok(self.thread.join().unwrap_or(ThreadReturn::JoinError))
            }
            receiver::Response::Err(err) => Err(err.into()),
        }
    }

    fn info(&self) -> String {
        match self.socket_addr {
            Some(addr) => format!("addr1: {}", addr),
            None => String::new(),
        }
    }
}

struct ReceiverThread {
    midi_tx: MidiOutputConnection,
    distant: TcpStream,
    messenger_rx: mpsc::Receiver<RTPPayload>,
}

impl ReceiverThread {
    pub fn new(
        midi_tx: MidiOutputConnection,
        messenger_rx: mpsc::Receiver<RTPPayload>,
        addr: SocketAddr,
    ) -> Result<Self> {
        Ok(ReceiverThread {
            midi_tx: midi_tx,
            distant: TcpStream::connect(addr)?,
            messenger_rx,
        })
    }

    pub fn run(&mut self) -> std::result::Result<(), ThreadReturn<Response>> {
        loop {
            let (req, responder) = self
                .messenger_rx
                .recv()
                .expect("unable to read from the messenger tunnel");
            match req {
                Request::Receive => self.receive(responder)?,
            }
        }
    }
    /// Starting to listen over UDP socket for
    fn receive(&mut self, responder: Responder) -> std::result::Result<(), ThreadReturn<Response>> {
        let mut buf: [u8; 33] = [0; 33];
        responder.send(Response::StartReceiving)?;
        loop {
            let len = self
                .distant
                .read(&mut buf)
                .map_err(|err| ThreadReturn::Read(err))?;

            if len == 0 {
                return Err(ThreadReturn::ReceiveEnd);
            }
            self.midi_tx
                .send(MidiFrame::get_payload(&buf))
                .map_err(|err| ThreadReturn::MidiSendError(err))?;
            trace!("MIDI -> {} bytes", len);
        }
    }
}
