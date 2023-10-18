use crate::{
    midi,
    net::{self, Result},
};
use midir::MidiInputConnection;

pub fn new_sender<T: net::Sender>(
    midi_port_index: usize,
    messenger_addr: T::Addr,
) -> Result<(MidiInputConnection<()>, T)> {
    let (conn, rx) = midi::new_receiver(midi_port_index)?;
    let net = T::new(rx, messenger_addr)?;

    Ok((conn, net))
}

pub fn new_receiver<T: net::Receiver>(
    midi_port_index: usize,
    messenger_addr: T::Addr,
) -> Result<T> {
    let conn = midi::new_sender(midi_port_index)?;
    let net = T::new(conn, messenger_addr)?;

    Ok(net)
}
