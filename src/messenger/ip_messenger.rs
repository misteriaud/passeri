// Following specs: https://developer.apple.com/library/archive/documentation/Audio/Conceptual/MIDINetworkDriverProtocol/MIDI/MIDI.html

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
	control_buffer: BytesMut,
	comm_socket: UdpSocket,
	comm_buffer: BytesMut,
	state: super::MessengerState,
}

impl IpMessenger {

	pub async fn new() -> Result<Self, std::io::Error> {
		Ok(IpMessenger {
				control_socket: UdpSocket::bind("127.0.0.1:0").await?,
				control_buffer: BytesMut::with_capacity(4096),
				comm_socket: UdpSocket::bind("127.0.0.1:0").await?,
				comm_buffer: BytesMut::with_capacity(4096),
				state: MessengerState::Idle
		})
	}

	pub async fn listen(&mut self) ->  Frame {
		loop {
			// Attempt to parse a frame from the buffered data. If
			// enough data has been buffered, the frame is
			// returned.
			if let Ok((_len, _addr)) = self.control_socket.recv_from(&mut self.control_buffer).await {
				match Frame::parse(&self.control_buffer) {
					Ok(frame) => return frame,
					Err(Error::Incomplete) => {},
					Err(Error::Invalid) => self.control_buffer.clear()
				}
			}
		}
	}
}
