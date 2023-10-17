use crate::messenger_thread::sender_trait::{self, NetSender};
use crate::messenger_thread::Result;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self};
use std::thread::JoinHandle;

use crate::midi_thread::MidiPayload;
use std::io::Write;

use super::ThreadReturn;

type Request = sender_trait::Request;
type Response = sender_trait::Response<<Sender as NetSender>::Addr>;
type Responder = sender_trait::Responder<<Sender as NetSender>::Addr>;
type RTPPayload = (Request, Responder);

pub struct Sender {
    thread: JoinHandle<ThreadReturn<Response>>,
    tx: mpsc::Sender<RTPPayload>,
    socket_addr: Option<SocketAddr>,
}

impl NetSender for Sender {
    type Addr = SocketAddr;
    type ThreadReturn = ThreadReturn<Response>;

    fn new(midi_rx: mpsc::Receiver<MidiPayload>, addr: Self::Addr) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<RTPPayload>();

        let mut socket = SenderThread::new(addr, midi_rx, rx).unwrap();
        let socket_addr = socket.get_addr();

        let thread = std::thread::spawn(move || socket.run().unwrap_err());

        Ok(Sender {
            thread,
            tx,
            socket_addr: Some(socket_addr),
        })
    }

    fn wait_for_client(&self) -> Result<Self::Addr> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::OpenRoom, response_sender))?;

        match response_receiver.recv()? {
            sender_trait::Response::NewClient(addr) => Ok(addr),
            _ => Err("invalid response from tcp_thread".into()),
        }
    }
    fn send(self, client: Self::Addr) -> Result<Self::ThreadReturn> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::AcceptClient, response_sender))?;

        match response_receiver.recv()? {
            sender_trait::Response::StartStream => {
                println!("received StartStream");
                Ok(self.thread.join().unwrap_or(ThreadReturn::JoinError))
            }
            _ => Err("invalid response from tcp_thread".into()),
        }
    }

    fn info(&self) -> String {
        match self.socket_addr {
            Some(addr) => format!("addr1: {}", addr),
            None => String::new(),
        }
    }
}

//
//	SENDER THREAD
//

struct SenderThread {
    local: TcpListener,
    distant: Option<TcpStream>,
    midi_rx: mpsc::Receiver<MidiPayload>,
    messenger_rx: mpsc::Receiver<RTPPayload>,
}

impl SenderThread {
    pub fn new(
        addr: SocketAddr,
        midi_rx: mpsc::Receiver<MidiPayload>,
        messenger_rx: mpsc::Receiver<RTPPayload>,
    ) -> Result<Self> {
        Ok(SenderThread {
            local: TcpListener::bind(addr)?,
            distant: None,
            midi_rx,
            messenger_rx,
        })
    }

    pub fn run(&mut self) -> std::result::Result<(), ThreadReturn<Response>> {
        loop {
            let (req, responder) = self
                .messenger_rx
                .recv()
                .map_err(|err| ThreadReturn::Recv(err))?
                .into();
            match req {
                Request::OpenRoom => self.open_room(responder)?,
                Request::AcceptClient => self.send(responder)?,
            }
        }
    }

    pub fn get_addr(&self) -> std::net::SocketAddr {
        self.local.local_addr().unwrap()
    }

    /// Starting to listen over UDP socket for
    fn open_room(
        &mut self,
        responder: Responder,
    ) -> std::result::Result<(), ThreadReturn<Response>> {
        responder
            .send(match self.local.accept() {
                Ok((distant, addr)) => {
                    self.distant = Some(distant);
                    Response::NewClient(addr)
                }
                Err(e) => Response::Err(e.to_string()),
            })
            .map_err(|err| ThreadReturn::Send(err))
    }

    fn send(&mut self, responder: Responder) -> std::result::Result<(), ThreadReturn<Response>> {
        Ok(match self.distant.as_mut() {
            Some(client) => {
                responder.send(Response::StartStream).unwrap();
                while let Some(msg) = self.midi_rx.iter().next() {
                    // println!("send {:?}", msg);
                    client
                        .write(&msg.1.serialize())
                        .map_err(|err| ThreadReturn::Write(err))?;
                }
            }
            None => responder
                .send(Response::Err("no client".into()))
                .map_err(|err| ThreadReturn::Send(err))?,
        })
    }
}
