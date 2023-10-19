use log::{error, trace};
use midir::MidiOutputConnection;
use passeri_api::midi::MidiFrame;
use passeri_api::net::receiver::{self, Request, Responder, Response};
use receiver::ThreadReturn;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;

type PasseriReq = (Request, Responder);

use passeri_api::net::{ReceiverThread, Result};
use std::io::Read;

pub struct Receiver {
    midi_tx: MidiOutputConnection,
    distant: TcpStream,
    messenger_rx: mpsc::Receiver<PasseriReq>,
}

impl ReceiverThread for Receiver {
    type Addr = SocketAddr;

    fn new(
        midi_tx: MidiOutputConnection,
        messenger_rx: mpsc::Receiver<PasseriReq>,
        addr: SocketAddr,
    ) -> Result<Self> {
        let distant = match TcpStream::connect(addr) {
            Ok(result) => result,
            Err(err) => {
                error!("fail to connect to {}", addr);
                return Err(Box::new(err));
            }
        };

        Ok(Receiver {
            midi_tx,
            distant: distant,
            messenger_rx,
        })
    }

    fn run(&mut self) -> std::result::Result<(), ThreadReturn> {
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
    fn receive(&mut self, responder: Responder) -> std::result::Result<(), ThreadReturn> {
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

    fn info(&self) -> String {
        format!(
            "Tcp Receiver is connect to {}",
            self.distant.local_addr().unwrap()
        )
    }
}
