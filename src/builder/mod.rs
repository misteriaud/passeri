use midir::MidiInputConnection;
use crate::{midi_thread, messenger_thread};

pub fn new_sender<T: messenger_thread::Messenger>(midi_port_index: usize, messenger_addr: T::Addr) -> Result<(Option<MidiInputConnection<()>>, T), String> {
	let (conn, rx) = midi_thread::new_receiver(midi_port_index)?;
	let net = T::new_sender(rx, messenger_addr);

	Ok((Some(conn), net))
}

