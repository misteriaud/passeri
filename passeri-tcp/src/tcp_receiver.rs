use log::{debug, trace};
use midir::MidiOutputConnection;
use passeri_api::midi::MidiParser;
use passeri_api::net::receiver::{Request, Responder, Response, Thread, ThreadReturn};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;

type PasseriReq = (Request, Responder);

use std::io::Read;

/// Implementation of the [Receiver Thread Trait](Thread) over TCP network
pub struct Receiver {
    midi_tx: MidiOutputConnection,
    distant: TcpStream,
    messenger_rx: mpsc::Receiver<PasseriReq>,
}

impl Thread for Receiver {
    type Addr = SocketAddr;

    fn new(
        addr: SocketAddr,
        midi_tx: MidiOutputConnection,
        messenger_rx: mpsc::Receiver<PasseriReq>,
    ) -> Result<Self, String> {
        debug!("try to connect to {}", addr);
        let distant = match TcpStream::connect(addr) {
            Ok(result) => result,
            Err(err) => {
                return Err(format!("{}", err));
            }
        };

        Ok(Receiver {
            midi_tx,
            distant: distant,
            messenger_rx,
        })
    }

    fn run(&mut self) -> Result<(), ThreadReturn> {
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
    fn receive(&mut self, responder: Responder) -> Result<(), ThreadReturn> {
        let mut buf: [u8; 1024] = [0; 1024];
        let mut midi_parser = MidiParser::new();
        responder.send(Response::StartReceiving)?;
        loop {
            let len = self
                .distant
                .read(&mut buf)
                .map_err(|err| ThreadReturn::Read(err))?;

            if len == 0 {
                return Err(ThreadReturn::ReceiveEnd);
            }
            for msgs in midi_parser.parse(&buf[..len]) {
                self.midi_tx
                    .send(&msgs)
                    .map_err(|err| ThreadReturn::MidiSendError(err))?;
                trace!("MIDI -> {} bytes", len);
            }
            if let Some(msg) = midi_parser.flush() {
                self.midi_tx
                    .send(&msg)
                    .map_err(|err| ThreadReturn::MidiSendError(err))?;
                trace!("MIDI -> {} bytes", len);
            }
        }
    }

    fn info(&self) -> String {
        format!("{}", self.distant.local_addr().unwrap())
    }
}
