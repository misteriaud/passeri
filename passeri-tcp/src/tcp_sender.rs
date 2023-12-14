use passeri_api::net::sender::{PasseriReq, Request, Responder, Response, Thread, ThreadReturn};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self};

use log::{debug, trace};
use passeri_api::midi::MidiPayload;
use std::io::{self, Read, Write};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

const CONNECTION_CHECK_ITV: Duration = Duration::from_secs(10);

/// `passeri_api::net::Sender` trait implementation over TCP
type Addr = <Sender as Thread>::Addr;

/// Implementation of the [Sender Thread Trait](Thread) over TCP network
pub struct Sender {
    local: TcpListener,
    distant: HashMap<Addr, TcpStream>,
    midi_rx: mpsc::Receiver<MidiPayload>,
    messenger_rx: mpsc::Receiver<PasseriReq<Addr>>,
}

impl Thread for Sender {
    type Addr = SocketAddr;

    fn new(
        addr: Self::Addr,
        midi_rx: mpsc::Receiver<MidiPayload>,
        messenger_rx: mpsc::Receiver<PasseriReq<Self::Addr>>,
    ) -> Result<Self, String> {
        let local = match TcpListener::bind(addr) {
            Ok(result) => result,
            Err(err) => {
                // error!("fail to bind on {}", addr);
                return Err(format!("{}", err));
            }
        };

        Ok(Sender {
            local,
            distant: HashMap::new(),
            midi_rx,
            messenger_rx,
        })
    }

    fn run(&mut self) -> Result<(), ThreadReturn<Self::Addr>> {
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
    ) -> Result<(), ThreadReturn<Self::Addr>> {
        if let Some(mut stream) = self.distant.remove(&distant) {
            responder.send(Response::StartStream).unwrap();
            let mut peek_buf = [0];

            loop {
                match self.midi_rx.recv_timeout(CONNECTION_CHECK_ITV) {
                    Ok(msg) => {
                        trace!("send {:?}", msg);
                        stream
                            .write(&msg.1)
                            .map_err(|err| ThreadReturn::Write(err))?;
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => {
                        if stream.read(&mut peek_buf).is_ok_and(|x| x == 0) {
                            debug!("received leaved");
                            return Err(ThreadReturn::RecvLeave);
                        }
                    }
                }
            }
            Err(ThreadReturn::SendEnd)
        } else {
            Ok(responder.send(Response::ClientNotFound)?)
        }
    }

    fn info(&self) -> Self::Addr {
        self.local.local_addr().unwrap()
    }
}

impl Sender {
    /// Starting to listen over UDP socket for
    fn open_room(&mut self, responder: Responder<Addr>) -> Result<(), ThreadReturn<Addr>> {
        let (distant, addr) = self.local.accept()?;
        distant
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");
        self.distant.insert(addr, distant);
        responder
            .send(Response::NewClient(addr))
            .map_err(|err| ThreadReturn::Send(err))
    }
}
