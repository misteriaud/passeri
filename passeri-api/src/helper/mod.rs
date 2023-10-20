use crate::{
    midi,
    net::{self, Result},
};

/// Helper function use to create a new [Sender](net::Sender) bridge
///
/// # Arguments
/// * `midi_port_index` - Index of a MIDI input port (you can get it from a [midi::get_availables_midi_port] function call)
/// * `binding_addr` - Address used by the given [net_thread][net::sender::Thread] implementation to listen on
pub fn new_sender<NetThread: net::sender::Thread>(
    midi_port_index: usize,
    binding_addr: NetThread::Addr,
) -> Result<net::Sender<NetThread>> {
    let (conn, rx) = midi::new_receiver(midi_port_index)?;
    let net = net::Sender::<NetThread>::new(conn, rx, binding_addr)?;

    Ok(net)
}

/// Helper function use to create a new [Receiver](net::Receiver) bridge
///
/// # Arguments
/// * `midi_port_index` - Index of a MIDI output port (you can get it from a [midi::get_availables_midi_port] function call)
/// * `sender_addr` - Address used by the given [net_thread][net::receiver::Thread]  implementation to connect to
pub fn new_receiver<NetThread: net::receiver::Thread>(
    midi_port_index: usize,
    sender_addr: NetThread::Addr,
) -> Result<net::Receiver> {
    let conn = midi::new_sender(midi_port_index)?;
    let net = net::Receiver::new::<NetThread>(conn, sender_addr)?;

    Ok(net)
}
