use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use midir::MidiOutputConnection;

use crate::midi_thread::{MidiPayload};
use crate::{rtp_midi};

use super::Messenger;

type Request = super::Request<<IpMessenger as Messenger>::Addr>;
type Response = super::Response<<IpMessenger as Messenger>::Addr>;
type Responder = super::Responder<<IpMessenger as Messenger>::Addr>;
type RTPPayload = (Request, Responder);


pub struct IpMessenger {
	thread: JoinHandle<()>,
	tx: Sender<RTPPayload>,
	socket_addr: (SocketAddr, SocketAddr)
}

impl Messenger for IpMessenger {
	type Addr = SocketAddr;

	fn new_sender(midi_rx: Receiver<MidiPayload>, addr: Self::Addr) -> Self {
		let (tx, rx) = channel::<RTPPayload>();

		let addr1 = addr;
		let mut addr2 = addr;
		addr2.set_port(addr.port() + 1);

		let mut socket = IpMessengerThread::new((addr1, addr2), IpMessengerMidiInterface::Sender(midi_rx), rx).unwrap();

		let thread = std::thread::spawn(move || {
			socket.run();
		});

		IpMessenger { thread, tx, socket_addr: (addr1, addr2) }
	}

	fn req(&self, req: Request) -> Result<Response, Box<dyn Error>> {
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

enum IpMessengerMidiInterface {
	Sender(Receiver<MidiPayload>),
	Receiver(MidiOutputConnection)
}

struct IpMessengerThread {
	rtcp_sock: UdpSocket,	// Real Time Control Protocol Socket
	rtcp_buf: [u8; 1024],
	rtp_sock: UdpSocket,	// Real Time Protocol Socket
	rtp_buf: [u8; 1024],
	midi_intf: IpMessengerMidiInterface,
	messenger_rx: Receiver<RTPPayload>
}

impl IpMessengerThread {
	pub fn new(addr: (SocketAddr, SocketAddr), midi_intf: IpMessengerMidiInterface, messenger_rx: Receiver<RTPPayload>) -> Result<Self, std::io::Error> {
		Ok(IpMessengerThread {
				rtcp_sock: UdpSocket::bind(addr.0)?,
				rtcp_buf: [0; 1024],
				rtp_sock: UdpSocket::bind(addr.1)?,
				rtp_buf: [0; 1024],
				midi_intf,
				messenger_rx
		})
	}

	pub fn run(&mut self) {
		for (req, responder) in self.messenger_rx.iter() {
			match req {
				Request::WaitForInvitation => responder.send(self.wait_for_invitation()).unwrap(),
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
	fn wait_for_invitation(&self) -> Response {
		// println!("Connected to {:?}", addr);
		let mut buf: [u8; 1024] = [0; 1024];
		let mut incomming_conn: HashMap<SocketAddr, Vec<u8>> = HashMap::new();


		loop {
			if let Ok((len, addr)) = self.rtcp_sock.recv_from(&mut buf) {
				let conn =  incomming_conn.entry(addr).or_insert_with(|| Vec::new());
				conn.append(&mut buf[..len].to_vec());

				println!("receive {:?} of size {}", conn, len);
				match rtp_midi::Frame::parse(&conn) {
					Ok(rtp_midi::Frame::IN(packet)) => return Response::InvitationReceived((addr, packet)),
					Err(rtp_midi::Error::Incomplete) => { println!("incomplete")},
					Err(rtp_midi::Error::Invalid) => {
						incomming_conn.remove(&addr);
						// conn.clear();
						// println!("invalid");
					},
					_ => { println!("incomplete")},
				}
			}
		}
	}

	// fn accept_invite(&self, packet: rtp_midi::ControlPacket) -> Response {

	// }

}
