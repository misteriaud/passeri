// Following specs: https://developer.apple.com/library/archive/documentation/Audio/Conceptual/MIDINetworkDriverProtocol/MIDI/MIDI.html

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use crate::midi_thread::{MidiSender, MidiReceiver, MidiPayload};
use bytes::BytesMut;
use bytes::Buf;
use std::io::Cursor;
use tokio::net::{UdpSocket, ToSocketAddrs};
use super::frame::Error;
use super::{frame::Frame, MessengerState};

pub struct IpMessenger {
	control_socket: UdpSocket,
	control_buffer: [u8; 1024],
	comm_socket: UdpSocket,
	comm_buffer: [u8; 1024],
	incommingConn: HashMap<SocketAddr, Vec<u8>>,
	state: super::MessengerState,
}

impl IpMessenger {

	pub async fn new() -> Result<Self, std::io::Error> {
		Ok(IpMessenger {
				control_socket: UdpSocket::bind("127.0.0.1:0").await?,
				control_buffer: [0; 1024],
				comm_socket: UdpSocket::bind("127.0.0.1:0").await?,
				comm_buffer: [0; 1024],
				incommingConn: HashMap::new(),
				state: MessengerState::Idle
		})
	}

	pub async fn listen(&mut self) ->  Frame {
		loop {
			// Attempt to parse a frame from the buffered data. If
			// enough data has been buffered, the frame is
			// returned.

			println!("listening on {:?}", self.control_socket.local_addr().unwrap());
			if let Ok((len, addr)) = self.control_socket.recv_from(&mut self.control_buffer).await {
				let conn =  self.incommingConn.entry(addr).or_insert_with(|| Vec::new());
				conn.append(&mut self.control_buffer[..len].to_vec());

				println!("receive {:?} of size {}", conn, len);
				match Frame::parse(&conn) {
					Ok(frame) => return frame,
					Err(Error::Incomplete) => { println!("incomplete")},
					Err(Error::Invalid) => {
						conn.clear();
						println!("invalid");
					}
				}
			}
		}
	}
}
