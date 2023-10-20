use std::{
    process::exit,
    sync::mpsc::{channel, Receiver},
};

use log::{info, trace};
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

mod midi_frame;
pub use midi_frame::MidiFrame;

const LOOKUP_PORT_NAME: &str = "passeri lookup";
const LISTEN_PORT_NAME: &str = "PASSERI_LISTENER";
const EMITTER_PORT_NAME: &str = "PASSERI_EMITTER";

/// Returns a vector of all MIDI input ports that [midir] can connect to
pub fn get_availables_midi_port() -> Result<Vec<(usize, String)>, String> {
    match MidiInput::new(LOOKUP_PORT_NAME) {
        Ok(lookup_port) => {
            let in_ports = lookup_port.ports();
            Ok(in_ports
                .into_iter()
                .map(|port| lookup_port.port_name(&port).unwrap())
                .enumerate()
                .collect::<Vec<(usize, String)>>())
        }
        Err(_) => Err("unable to lookup available ports".into()),
    }
}

pub type MidiPayload = (u64, MidiFrame);

/// Create a new [MidiOutputConnection] instance, which can be called to send MIDI message to the provided MIDI port
///
/// Under the hood, [midir] spawn a background listening thread which is waiting for any incomming call to the returned instance.
///
/// # Arguments
/// * `midi_port_index` - Index of a MIDI output port (you can get it from a [get_availables_midi_port] function call)
pub fn new_sender(midi_port_index: usize) -> Result<MidiOutputConnection, String> {
    let midi_out = MidiOutput::new(EMITTER_PORT_NAME).expect("unable to create the lookup port");
    info!("MIDI-OUT port is set up to: {}", EMITTER_PORT_NAME);

    if let Some(port) = midi_out.ports().get(midi_port_index) {
        info!("midi_thread is running for {}", EMITTER_PORT_NAME);
        if let Ok(conn) = midi_out.connect(port, "midir-read-input") {
            return Ok(conn);
        }
        return Err("unable to connect to the port".into());
    }
    Err("couldnt find the port".into())
}

/// Create a new [MidiInputConnection] instance, which will forward any received MIDI message to the returned [Receiver](Receiver) end tunnel
///
/// # Arguments
/// * `midi_port_index` - Index of a MIDI output port (you can get it from a [get_availables_midi_port] function call)
pub fn new_receiver(
    midi_port_index: usize,
) -> Result<(MidiInputConnection<()>, Receiver<MidiPayload>), String> {
    let mut midi_in = MidiInput::new(LISTEN_PORT_NAME).expect("unable to create the lookup port");
    midi_in.ignore(Ignore::None);
    info!("MIDI-IN port is set up to: {}", LISTEN_PORT_NAME);

    match midi_in.ports().get(midi_port_index) {
        Some(port) => {
            info!("midi_thread is running for {}", LISTEN_PORT_NAME);
            let (tx, rx) = channel::<MidiPayload>();

            match midi_in.connect(
                port,
                "midir-read-input",
                move |stamp: u64, msg: &[u8], _| {
                    trace!("msg: {}", stamp);
                    if let Err(_) = tx.send((stamp, msg.into())) {
                        exit(1);
                    }
                },
                (),
            ) {
                Ok(conn) => Ok((conn, rx)),
                Err(_) => Err("unable to connect to the port".into()),
            }
        }
        None => Err("couldnt find the port".into()),
    }
}
