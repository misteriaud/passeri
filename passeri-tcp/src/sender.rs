use passeri_api::net::{self, sender, Result};
use sender::{PasseriReq, Request, Responder, Response, ThreadReturn};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self};

use log::{error, trace};
use passeri_api::midi::MidiPayload;
use std::io::Write;

// use super::ThreadReturn;

// type Request = sender::Request<<Sender as passeri_api::net::Sender>::Addr>;
// type Response = sender::Response<<Sender as passeri_api::net::Sender>::Addr>;
// type Responder = sender::Responder<<Sender as passeri_api::net::Sender>::Addr>;
// type RTPPayload = (Request, Responder);

/// `passeri_api::net::Sender` trait implementation over TCP
type Addr = <Sender as net::SenderThread>::Addr;

pub struct Sender {
    local: TcpListener,
    distant: HashMap<Addr, TcpStream>,
    midi_rx: mpsc::Receiver<MidiPayload>,
    messenger_rx: mpsc::Receiver<PasseriReq<Addr>>,
}

impl net::SenderThread for Sender {
    type Addr = SocketAddr;

    fn new(
        addr: Self::Addr,
        midi_rx: mpsc::Receiver<MidiPayload>,
        messenger_rx: mpsc::Receiver<net::sender::PasseriReq<Self::Addr>>,
    ) -> Result<Self> {
        let local = match TcpListener::bind(addr) {
            Ok(result) => result,
            Err(err) => {
                error!("fail to bind on {}", addr);
                return Err(Box::new(err));
            }
        };

        Ok(Sender {
            local,
            distant: HashMap::new(),
            midi_rx,
            messenger_rx,
        })
    }

    fn run(&mut self) -> std::result::Result<(), ThreadReturn<Self::Addr>> {
        loop {
            let (req, responder) = self
                .messenger_rx
                .recv()
                .map_err(|err| ThreadReturn::Recv(err))?
                .into();
            match req {
                Request::OpenRoom => self.open_room(responder)?,
                Request::AcceptClient(addr) => self.send(addr, responder)?,
            }
        }
    }

    fn send(
        &mut self,
        distant: SocketAddr,
        responder: Responder<Self::Addr>,
    ) -> std::result::Result<(), ThreadReturn<Self::Addr>> {
        if let Some(mut stream) = self.distant.remove(&distant) {
            responder.send(Response::StartStream).unwrap();
            while let Some(msg) = self.midi_rx.iter().next() {
                trace!("send {:?}", msg);
                stream
                    .write(&msg.1.serialize())
                    .map_err(|err| ThreadReturn::Write(err))?;
            }
            Err(ThreadReturn::SendEnd)
        } else {
            Ok(responder.send(sender::Response::ClientNotFound)?)
        }
    }

    fn info(&self) -> String {
        format!(
            "Tcp Sender is listening on {}",
            self.local.local_addr().unwrap()
        )
    }
}

impl Sender {
    /// Starting to listen over UDP socket for
    fn open_room(
        &mut self,
        responder: Responder<Addr>,
    ) -> std::result::Result<(), ThreadReturn<Addr>> {
        let (distant, addr) = self.local.accept()?;
        self.distant.insert(addr, distant);
        responder
            .send(sender::Response::NewClient(addr))
            .map_err(|err| ThreadReturn::Send(err))
    }
}
