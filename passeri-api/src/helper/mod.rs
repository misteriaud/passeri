use crate::{
    midi,
    net::{self, Result},
};
use midir::MidiInputConnection;

pub fn new_sender<T: net::SenderThread>(
    midi_port_index: usize,
    messenger_addr: T::Addr,
) -> Result<net::Sender<T>> {
    let (conn, rx) = midi::new_receiver(midi_port_index)?;
    let net = net::Sender::<T>::new(conn, rx, messenger_addr)?;

    Ok(net)
}

pub fn new_receiver<T: net::ReceiverThread>(
    midi_port_index: usize,
    messenger_addr: T::Addr,
) -> Result<net::Receiver> {
    let conn = midi::new_sender(midi_port_index)?;
    let net = net::Receiver::new::<T>(conn, messenger_addr)?;

    Ok(net)
}
