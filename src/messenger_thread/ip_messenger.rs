use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use crate::midi_thread::{MidiPayload};
use crate::{rtp_midi};

use super::*;

type RTPPayload = (Request<SocketAddr>, Responder);



pub struct IpMessenger {
	thread: JoinHandle<()>,
	tx: Sender<RTPPayload>,
	socket_addr: (SocketAddr, SocketAddr)
}

impl Messenger for IpMessenger {
	type Addr  = SocketAddr;

	fn new_sender(midi_rx: Receiver<MidiPayload>) -> Self {
		let (tx, rx) = channel::<RTPPayload>();
		let mut socket = IpMessengerSender::new(midi_rx, rx).unwrap();

		let addr1 = socket.rtcp_sock.local_addr().unwrap();
		let addr2 = socket.rtcp_sock.local_addr().unwrap();

		let thread = std::thread::spawn(move || {
			socket.run();
		});

		IpMessenger { thread, tx, socket_addr: (addr1, addr2) }
	}

	fn req(&self, req: Request<SocketAddr>) -> Result<Response, Box<dyn Error>> {
		let (response_sender, response_receiver) = oneshot::channel() ;
		self.tx.send((req, response_sender))?;

		Ok(response_receiver.recv()?)
	}

	fn info(&self) -> String {
		format!("addr1: {}, addr2: {}", self.socket_addr.0, self.socket_addr.1)
	}


	// fn new_receiver(midi_out: MidiOutputConnection) -> Self {

	// }
}

struct IpMessengerSender {
	rtcp_sock: UdpSocket,	// Real Time Control Protocol Socket
	rtcp_buf: [u8; 1024],
	rtp_sock: UdpSocket,	// Real Time Protocol Socket
	rtp_buf: [u8; 1024],
	incommingConn: RefCell<HashMap<SocketAddr, Vec<u8>>>,
	midi_rx: Receiver<MidiPayload>,
	messenger_rx: Receiver<RTPPayload>
}

impl IpMessengerSender {
	pub fn new(midi_rx: Receiver<MidiPayload>, messenger_rx: Receiver<RTPPayload>) -> Result<Self, std::io::Error> {
		Ok(IpMessengerSender {
				rtcp_sock: UdpSocket::bind("127.0.0.1:0")?,
				rtcp_buf: [0; 1024],
				rtp_sock: UdpSocket::bind("127.0.0.1:0")?,
				rtp_buf: [0; 1024],
				incommingConn: RefCell::new(HashMap::new()),
				midi_rx,
				messenger_rx
		})
	}

	pub fn run(&mut self) {
		for (req, responder) in self.messenger_rx.iter() {
			match req {
				Request::WaitForInvitation(addr) => responder.send(self.wait_for_invitation(addr)).unwrap(),
				// Request::AcceptInvitation(packet) => responder.send(self.accept_invite(packet)).unwrap(),
				_ => {}
			}
		}
	}

	pub fn get_addr(&self) -> Option<std::net::SocketAddr>
	{
		self.rtcp_sock.local_addr().ok()
	}

	/// Starting to listen over UDP socket for
	fn wait_for_invitation(&self, addr: SocketAddr) -> Response {
		// connect to the provided address
		self.rtcp_sock.connect(addr).expect("error");
		// if let Err(_err) = self.rtcp_sock.connect(addr) {
		// 	return Response::Err;
		// }

		println!("Connected to {:?}", addr);

		let mut buf: [u8; 1024] = [0; 1024];
		let mut vec = vec![];

		loop {
			if let Ok(len) = self.rtcp_sock.recv(&mut buf) {
				vec.append(&mut buf[..len].to_vec());

				println!("receive {:?} of size {}", buf, len);
				match rtp_midi::Frame::parse(&vec) {
					Ok(rtp_midi::Frame::IN(packet)) => return Response::InvitationReceived(packet),
					Err(rtp_midi::Error::Incomplete) => { println!("incomplete")},
					Err(rtp_midi::Error::Invalid) => {
						vec.clear();
						println!("invalid");
					},
					_ => { println!("incomplete")},
				}
			}
		}
	}

	// fn accept_invite(&self, packet: rtp_midi::ControlPacket) -> Response {

	// }

}
