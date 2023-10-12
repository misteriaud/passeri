use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::error::Error;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::ops::DerefMut;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

use bytes::Bytes;
use midir::MidiOutputConnection;

use crate::midi_thread::MidiPayload;
use std::io::{Read, Write};

use super::{midi_frame, Messenger};

type Request = super::Request<<TcpMessenger as Messenger>::Addr>;
type Response = super::Response<<TcpMessenger as Messenger>::Addr>;
type Responder = super::Responder<<TcpMessenger as Messenger>::Addr>;
type RTPPayload = (Request, Responder);

pub enum TcpError {
    UnmutableTcpStream,
}
pub struct TcpMessenger {
    thread: JoinHandle<()>,
    tx: Sender<RTPPayload>,
    socket_addr: Option<SocketAddr>,
}

impl Messenger for TcpMessenger {
    type Addr = SocketAddr;

    fn new_sender(midi_rx: Receiver<MidiPayload>, addr: Self::Addr) -> Self {
        let (tx, rx) = channel::<RTPPayload>();

        let mut socket = TcpSender::new(addr, midi_rx, rx).unwrap();
        let socket_addr = socket.get_addr();

        let thread = std::thread::spawn(move || {
            socket.run();
        });

        TcpMessenger {
            thread,
            tx,
            socket_addr: Some(socket_addr),
        }
    }

    fn new_receiver(midi_out: MidiOutputConnection) -> Self {
        let (tx, rx) = channel::<RTPPayload>();

        let mut socket = TcpReceiver::new(midi_out, rx).unwrap();

        let thread = std::thread::spawn(move || {
            socket.run();
        });

        TcpMessenger {
            thread,
            tx,
            socket_addr: None,
        }
    }

    fn req(&self, req: Request) -> Result<Response, Box<dyn Error>> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((req, response_sender))?;

        Ok(response_receiver.recv()?)
    }

    fn info(&self) -> String {
        match self.socket_addr {
            Some(addr) => format!("addr1: {}", addr),
            None => String::new(),
        }
    }

    // fn new_receiver(midi_out: MidiOutputConnection) -> Self {

    // }
}

struct TcpSender {
    local: TcpListener,
    distant: RefCell<Option<TcpStream>>,
    midi_rx: Receiver<MidiPayload>,
    messenger_rx: Receiver<RTPPayload>,
}

impl TcpSender {
    pub fn new(
        addr: SocketAddr,
        midi_rx: Receiver<MidiPayload>,
        messenger_rx: Receiver<RTPPayload>,
    ) -> Result<Self, std::io::Error> {
        Ok(TcpSender {
            local: TcpListener::bind(addr)?,
            distant: RefCell::new(None),
            midi_rx,
            messenger_rx,
        })
    }

    pub fn run(&mut self) {
        for (req, responder) in self.messenger_rx.iter() {
            match req {
                Request::OpenRoom => responder.send(self.open_room()).unwrap(),
                Request::AcceptClient => responder.send(self.send()).unwrap(),
                // Request::AcceptInvitation(packet) => responder.send(self.accept_invite(packet)).unwrap(),
                _ => {}
            }
        }
    }

    pub fn get_addr(&self) -> std::net::SocketAddr {
        self.local.local_addr().unwrap()
    }

    /// Starting to listen over UDP socket for
    fn open_room(&self) -> Response {
        match self.local.accept() {
            Ok((distant, addr)) => {
                *self.distant.borrow_mut() = Some(distant);
                Response::NewClient(addr)
            }
            Err(e) => Response::Err(e.to_string()),
        }
    }

    fn send(&self) -> Response {
        match self.distant.try_borrow_mut() {
            Ok(mut client_ref) => {
                if let Some(client) = client_ref.deref_mut() {
                    while let Some(msg) = self.midi_rx.iter().next() {
                        println!("send {:?}", msg);
                        match client.write(&msg.1.serialize()) {
                            Err(err) => return Response::Err(err.to_string()),
                            _ => {}
                        }
                    }
                    return Response::HasHangUp;
                }
                return Response::Err("no client".into());
            }
            Err(err) => Response::Err(err.to_string()),
        }
    }
}

struct TcpReceiver {
    midi_tx: RefCell<MidiOutputConnection>,
    messenger_rx: Receiver<RTPPayload>,
}

impl TcpReceiver {
    pub fn new(
        midi_tx: MidiOutputConnection,
        messenger_rx: Receiver<RTPPayload>,
    ) -> Result<Self, std::io::Error> {
        Ok(TcpReceiver {
            midi_tx: RefCell::new(midi_tx),
            messenger_rx,
        })
    }

    pub fn run(&mut self) {
        for (req, responder) in self.messenger_rx.iter() {
            match req {
                Request::JoinRoom(addr) => responder.send(self.join_room(addr)).unwrap(),
                // Request::AcceptInvitation(packet) => responder.send(self.accept_invite(packet)).unwrap(),
                _ => {}
            }
        }
    }
    /// Starting to listen over UDP socket for
    fn join_room(&self, addr: SocketAddr) -> Response {
        if let Ok(mut stream) = TcpStream::connect(addr) {
            let mut buf: [u8; 33] = [0; 33];
            match self.midi_tx.try_borrow_mut() {
                Ok(mut midi) => loop {
                    match stream.read(&mut buf) {
                        Ok(len) => {
                            println!("receive {:?}", midi_frame::get_payload(&buf));
                            match midi.send(midi_frame::get_payload(&buf)) {
                                Ok(()) => println!("send {} bytes", len),
                                Err(err) => println!("err: {}", err),
                            }
                        }
                        Err(err) => return Response::Err(err.to_string()),
                    }
                },
                Err(err) => return Response::Err(err.to_string()),
            }
        }
        Response::Err(format!("error while connecting to {}", addr))
    }
}
