use midir::MidiInputConnection;

use crate::{midi_thread, net_thread::RTPMessenger};

enum Binder<T: RTPMessenger> {
	Sender {
		midi: MidiInputConnection<()>,
		net: T
	},
	Receiver {
		net: T
	},
}

impl<T: RTPMessenger> Binder<T> {
	fn new_sender(midi_port_index: usize) -> Result<Self, String> {
		let (conn, rx) = midi_thread::new_receiver(midi_port_index)?;
		let net = T::new_sender(rx);

		Ok(Binder::Sender { midi: conn, net })
	}

	fn new_receiver(midi_port_index: usize) -> Result<Self, String> {
		let midi = midi_thread::new_sender(midi_port_index)?;
		let net = T::new_receiver(midi);

		Ok(Binder::Receiver { net })
	}

	fn init_session(&mut self) {
		// self.messenger.init_session()
	}
}
