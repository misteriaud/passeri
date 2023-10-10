use crate::midi_thread::{MidiSender, MidiReceiver, MidiPayload};

pub trait messenger {
	fn init_session();
	fn accept_session();
	fn sync();
	// fn send();
	// fn recv();
}

enum MessengerState {
	Idle,
	Sender(MidiSender),
	Receiver(MidiReceiver),
}


pub mod frame;

pub mod ip_messenger;


// struct invitation {
// 	initiator_token: u32,
// 	ssrc: u32,
// 	name: String,
// }

// type Responder<T> = oneshot::Sender<T>;

