// Following specs: https://developer.apple.com/library/archive/documentation/Audio/Conceptual/MIDINetworkDriverProtocol/MIDI/MIDI.html

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use midir::MidiOutputConnection;

use crate::midi_thread::{MidiPayload};
use crate::rtp_midi;

use super::RTPMessenger;



enum IpMessengerInternal {
	Sender {
		rtcp_sock: UdpSocket,	// Real Time Control Protocol Socket
		rtcp_buf: [u8; 1024],
		rtp_sock: UdpSocket,	// Real Time Protocol Socket
		rtp_buf: [u8; 1024],
		incommingConn: HashMap<SocketAddr, Vec<u8>>,
		midi_rx: Receiver<MidiPayload>,
		messenger_rx: Receiver<rtp_midi::Frame>
	}
}

pub enum IpMessenger {
	Sender {
		thread: JoinHandle<()>,
		tx: Sender<rtp_midi::Frame>
	},
	Receiver {
		thread: JoinHandle<()>,
		rx: Receiver<rtp_midi::Frame>
	}
}

impl RTPMessenger for IpMessenger {
	fn new_sender(midi_rx: Receiver<MidiPayload>) -> Self {
		let (tx, rx) = channel::<rtp_midi::Frame>();
		let thread = std::thread::spawn(move || {
			let socket = IpMessengerInternal::new_sender(midi_rx, rx);



		});

		IpMessenger::Sender { thread, tx }
	}
	fn new_receiver(midi_out: MidiOutputConnection) -> Self {

	}
}

impl IpMessengerInternal {
	pub async fn new_sender(midi_rx: Receiver<MidiPayload>, messenger_rx: Receiver<rtp_midi::Frame>) -> Result<Self, std::io::Error> {
		Ok(IpMessengerInternal::Sender {
				rtcp_sock: UdpSocket::bind("127.0.0.1:0")?,
				rtcp_buf: [0; 1024],
				rtp_sock: UdpSocket::bind("127.0.0.1:0")?,
				rtp_buf: [0; 1024],
				incommingConn: HashMap::new(),
				midi_rx,
				messenger_rx
		})
	}

	pub fn get_addr(&self) -> Option<std::net::SocketAddr>
	{
		self.rtcp_sock.local_addr().ok()
	}

	/// Starting to listen over UDP socket for
	pub async fn listen(&mut self) -> rtp_midi::Frame {
		loop {

			if let Ok((len, addr)) = self.rtcp_sock.recv_from(&mut self.rtcp_buf) {
				let conn =  self.incommingConn.entry(addr).or_insert_with(|| Vec::new());
				conn.append(&mut self.rtcp_buf[..len].to_vec());

				println!("receive {:?} of size {}", conn, len);
				match rtp_midi::Frame::parse(&conn) {
					Ok(frame) => return frame,
					Err(rtp_midi::Error::Incomplete) => { println!("incomplete")},
					Err(rtp_midi::Error::Invalid) => {
						conn.clear();
						println!("invalid");
					}
				}
			}
		}
	}
}
