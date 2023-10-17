use crate::messenger_thread::{
    receiver_trait::{self, NetReceiver, Request, Responder, Response},
    MidiFrame,
};
use midir::MidiOutputConnection;
use std::cell::RefCell;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread::JoinHandle;

type RTPPayload = (Request, Responder);

use crate::messenger_thread::Result;
use std::io::Read;

pub struct Receiver {
    thread: JoinHandle<()>,
    tx: mpsc::Sender<RTPPayload>,
    socket_addr: Option<SocketAddr>,
}

impl NetReceiver for Receiver {
    type Addr = SocketAddr;

    fn new(midi_out: MidiOutputConnection, addr: Self::Addr) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<RTPPayload>();

        let mut socket = ReceiverThread::new(midi_out, rx, addr)?;

        let thread = std::thread::spawn(move || {
            socket.run();
        });

        Ok(Receiver {
            thread,
            tx,
            socket_addr: None,
        })
    }

    fn receive(&self) -> Result<()> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::Receive, response_sender))?;

        match response_receiver.recv()? {
            receiver_trait::Response::StartReceiving => Ok(()),
            receiver_trait::Response::Err(err) => Err(err.into()),
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

    pub fn run(&mut self) {
        loop {
            let (req, responder) = self
                .messenger_rx
                .recv()
                .expect("unable to read from the messenger tunnel");
            match req {
                Request::Receive => responder.send(self.receive()).unwrap(),
            }
        }
    }
    /// Starting to listen over UDP socket for
    fn receive(&mut self) -> Response {
        let mut buf: [u8; 33] = [0; 33];
        loop {
            match self.distant.read(&mut buf) {
                Ok(len) => {
                    println!("receive {:?}", MidiFrame::get_payload(&buf));
                    match self.midi_tx.send(MidiFrame::get_payload(&buf)) {
                        Ok(()) => println!("send {} bytes", len),
                        Err(err) => println!("err: {}", err),
                    }
                }
                Err(err) => return Response::Err(err.to_string()),
            }
        }
    }
}
