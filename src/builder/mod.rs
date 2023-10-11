use midir::MidiInputConnection;
use crate::{midi_thread, net_thread};

pub fn new_sender<T: net_thread::Messenger>(midi_port_index: usize) -> Result<(Option<MidiInputConnection<()>>, T), String> {
	let (conn, rx) = midi_thread::new_receiver(midi_port_index)?;
	let net = T::new_sender(rx);

	Ok((Some(conn), net))
}

