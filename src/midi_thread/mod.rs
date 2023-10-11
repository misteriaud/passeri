use std::{sync::mpsc::{Sender, Receiver, channel}, process::exit};

use midir::{MidiInput, Ignore, MidiInputConnection, MidiOutputConnection, MidiOutput};

const LOOKUP_PORT_NAME: &str = "passeri lookup";
const LISTEN_PORT_NAME: &str = "passeri listener";


pub fn get_availables_midi_port() -> Result<Vec<(usize, String)>, String> {
	match MidiInput::new(LOOKUP_PORT_NAME) {
		Ok(lookup_port) => {
			let in_ports = lookup_port.ports();
			Ok(in_ports
			.into_iter()
			.map(|port| lookup_port.port_name(&port).unwrap())
			.enumerate()
			.collect::<Vec<(usize, String)>>())
		},
		Err(_) => Err("unable to lookup available ports".into())
	}
}

pub type MidiPayload = (u64, [u8;8]);


pub fn new_sender(midi_port_index: usize) -> Result<MidiOutputConnection, String> {
	let midi_out = MidiOutput::new(LISTEN_PORT_NAME).expect("unable to create the lookup port");
	println!("{} midi port is set up", LISTEN_PORT_NAME);

	if let Some(port) = midi_out.ports().get(midi_port_index) {
			println!("routine is setup for {} midi port", LISTEN_PORT_NAME);
			if let Ok(conn) = midi_out.connect(
				port,
				"midir-read-input"
			) {
				return Ok( conn )
			}
			return Err("unable to connect to the port".into())
		}
	Err("couldnt find the port".into())
}


pub fn new_receiver(midi_port_index: usize) -> Result<(MidiInputConnection<()>, Receiver<MidiPayload>), String> {
	let mut midi_in = MidiInput::new(LISTEN_PORT_NAME).expect("unable to create the lookup port");
	midi_in.ignore(Ignore::None);
	println!("{} midi port is set up", LISTEN_PORT_NAME);

	match midi_in.ports().get(midi_port_index) {
		Some(port) => {
			println!("routine is setup for {} midi port", LISTEN_PORT_NAME);
			let (tx, rx) = channel::<MidiPayload>();

			match midi_in.connect(
				port,
				"midir-read-input",
				move |stamp: u64, msg: &[u8], _| {
					println!("msg: {}", stamp);
					if let Err(_) = tx.send((stamp, msg[..1].try_into().unwrap())) {
						exit(1);
					}
				},
				()) {
					Ok(conn) => Ok((conn, rx)),
					Err(_) => Err("unable to connect to the port".into())
				}
		}
		None => Err("couldnt find the port".into())
	}
}





