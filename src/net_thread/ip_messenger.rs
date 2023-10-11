// Following specs: https://developer.apple.com/library/archive/documentation/Audio/Conceptual/MIDINetworkDriverProtocol/MIDI/MIDI.html

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use midir::MidiOutputConnection;
use crate::midi_thread::{MidiPayload};
use crate::{rtp_midi};

use super::*;

type RTPPayload = (Request, Responder);

pub struct IpMessenger {
	thread: JoinHandle<()>,
	tx: Sender<RTPPayload>,
	socket_addr: (SocketAddr, SocketAddr)
}

impl Messenger for IpMessenger {
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
				Request::WaitForInvitation => responder.send(self.listen()).unwrap(),
				_ => {}
			}
		}
	}

	pub fn get_addr(&self) -> Option<std::net::SocketAddr>
	{
		self.rtcp_sock.local_addr().ok()
	}

	/// Starting to listen over UDP socket for
	pub fn listen(&self) -> Response {
		let mut buf: [u8; 1024] = [0; 1024];

		loop {

			if let Ok((len, addr)) = self.rtcp_sock.recv_from(&mut buf) {
				let mut entities = self.incommingConn.borrow_mut();
				let conn =  entities.entry(addr).or_insert_with(|| Vec::new());
				conn.append(&mut buf[..len].to_vec());

				println!("receive {:?} of size {}", conn, len);
				match rtp_midi::Frame::parse(&conn) {
					Ok(rtp_midi::Frame::IN { initiator_token, ssrc, name }) => return Response::InvitationReceived { initiator_token, ssrc, name },
					Err(rtp_midi::Error::Incomplete) => { println!("incomplete")},
					Err(rtp_midi::Error::Invalid) => {
						conn.clear();
						println!("invalid");
					},
					_ => { println!("incomplete")},
				}
			}
		}
	}
}
