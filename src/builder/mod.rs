use crate::{
    messenger_thread::{self, Result},
    midi_thread,
};
use midir::MidiInputConnection;

pub fn new_sender<T: messenger_thread::sender_trait::NetSender>(
    midi_port_index: usize,
    messenger_addr: T::Addr,
) -> Result<(MidiInputConnection<()>, T)> {
    let (conn, rx) = midi_thread::new_receiver(midi_port_index)?;
    let net = T::new(rx, messenger_addr)?;

    Ok((conn, net))
}

pub fn new_receiver<T: messenger_thread::receiver_trait::NetReceiver>(
    midi_port_index: usize,
    messenger_addr: T::Addr,
) -> Result<T> {
    let conn = midi_thread::new_sender(midi_port_index)?;
    let net = T::new(conn, messenger_addr)?;

    Ok(net)
}
