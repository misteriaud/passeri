use std::net::{SocketAddr, Ipv4Addr};
use std::process::exit;
use std::time::Duration;
use passeri::messenger_thread::Messenger;

use passeri::{messenger_thread::{ip_messenger::IpMessenger, self}, builder};

fn main() {
	let (_midi_instance, net_instance) = builder::new_sender::<IpMessenger>(0).unwrap_or_else(|err| {
		println!("Error while trying to create binding: {}", err);
		exit(1);
	});

	println!("{}", net_instance.info());

	// std::thread::sleep(Duration::from_secs(10));
	let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4242);

	match net_instance.req(messenger_thread::Request::WaitForInvitation(addr)) {
		Ok(resp) => println!("{:?}", resp),
		Err(err) => println!("{}", err)
	}

}
