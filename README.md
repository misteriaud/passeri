Passeri is a [MIDI](https://en.wikipedia.org/wiki/MIDI) **Sender**/**Receiver** over Network (*TCP*, *Bluetooth*, and other to come).

It is build on a parallel blocking architecture composed of 3 threads:
- `main_thread`
- `midi_thread`
- `network_thread`

This architecture have been chosen over concurrent one for its efficiency.

**Passeri** is shipped with a couple of helper functions to make it easier to use.

# Roadmap
- MIDI thread implementation
	- [x] MIDI sender
	- [x] MIDI receiver
- Network thread implementation
	- [x] modular binding using trait
	- [x] support of TCP
	- [ ] support of Bluetooth
- [ ] GUI using Tauri

# Exemple
Let's build simples sender and receiver over TCP.


## Sender
```rust
use std::net::{Ipv4Addr, SocketAddr};
use std::process::exit;
use std::str::FromStr

use passeri::{
	builder,
	messenger_thread::{tcp, Sender},
};

fn main() {
	let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),

	let (_midi_instance, net_instance) =
	builder::new_sender::<tcp::TcpSender>(0, addr).unwrap_or_else(|err| {
		println!("Error while building the sender: {}", err);
		exit(1);
	});

	match net_instance.wait_for_client() {
		Ok(addr) => {
			println!("{} is now connected", addr);
			match net_instance.send(addr) {
				Ok(thread_resp) => println!("the net thread ended: {:?}", thread_resp),
				Err(err) => println!("internal err: {}", err),
			}
		}
		Err(err) => println!("{}", err),
	}
}
```

## Receiver
```rust
use std::net::{Ipv4Addr, SocketAddr};
use std::process::exit;
use std::str::FromStr

use passeri::{
	builder,
	messenger_thread::{tcp, Receiver},
};

fn main() {
	let sender_addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),

	let (_midi_instance, net_instance) =
	builder::new_sender::<tcp::TcpSender>(0, sender_addr).unwrap_or_else(|err| {
		println!("Error while building the receiver: {}", err);
		exit(1);
	});

	match net_instance.wait_for_client() {
		Ok(addr) => {
			println!("{} is now connected", addr);
			match net_instance.send(addr) {
				Ok(thread_resp) => println!("the net thread ended: {:?}", thread_resp),
				Err(err) => println!("internal err: {}", err),
			}
		}
		Err(err) => println!("{}", err),
	}
}
```
