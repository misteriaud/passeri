use std::process::exit;
use passeri::net_thread::Messenger;

use passeri::{net_thread::{ip_messenger::IpMessenger, self}, builder};

fn main() {
	let (_midi_instance, net_instance) = builder::new_sender::<IpMessenger>(0).unwrap_or_else(|err| {
		println!("Error while trying to create binding: {}", err);
		exit(1);
	});

	println!("{}", net_instance.info());

	match net_instance.req(net_thread::Request::WaitForInvitation) {
		Ok(resp) => println!("{:?}", resp),
		Err(err) => println!("{}", err)
	}

}
