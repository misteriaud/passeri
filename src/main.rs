use std::{sync::mpsc::channel, process::exit};

use passeri::{midi_thread::{self, MidiPayload}, messenger::ip_messenger::IpMessenger};


#[tokio::main]
async fn main() {

	let mut messenger = IpMessenger::new().await.unwrap_or_else(|err| {
		println!("Error while initializing messenger: {}", err);
		exit(1);
	});

	let response = messenger.listen().await;
	println!("frame: {:?}", response);

	// // let _conn = midi_thread::setup_midi_receiver(0, tx).unwrap_or_else(|err| {
	// // 	println!("Error while initializing listening midi port: {}", err);
	// // 	exit(1);
	// // });

	// println!("passeri is listening on port {}", port_name);
	// loop {
	// 	if let Ok(msg) = rx.try_recv() {
	// 			println!("msg: {:?}", msg);
	// 	}
	// }

}
