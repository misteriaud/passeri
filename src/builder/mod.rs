use crate::{messenger_thread, midi_thread};
use midir::MidiInputConnection;

pub fn new_sender<T: messenger_thread::Messenger>(
    midi_port_index: usize,
    messenger_addr: T::Addr,
) -> Result<(MidiInputConnection<()>, T), String> {
    let (conn, rx) = midi_thread::new_receiver(midi_port_index)?;
    let net = T::new_sender(rx, messenger_addr);

    Ok((conn, net))
}

pub fn new_receiver<T: messenger_thread::Messenger>(midi_port_index: usize) -> Result<T, String> {
    let conn = midi_thread::new_sender(midi_port_index)?;
    let net = T::new_receiver(conn);

    Ok(net)
}
