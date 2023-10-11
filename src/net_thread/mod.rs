use std::sync::mpsc::Receiver;

use midir::MidiOutputConnection;

use crate::midi_thread::MidiPayload;


pub trait RTPMessenger {
	fn new_sender(rx: Receiver<MidiPayload>) -> Self;
	fn new_receiver(midi_out: MidiOutputConnection) -> Self;
	// fn sync();
	// fn send();
	// fn recv();
}

// pub trait NetSender {
// 	fn init_session();
// }

// pub trait NetReceiver {
// 	fn accept_session();
// }

pub mod ip_messenger;
