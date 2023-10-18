use std::{
    net::{Ipv4Addr, SocketAddr},
    process::exit,
};

use passeri::net::Sender;
use passeri::tcp;

fn main() {
    let listening_addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

    let (_midi_instance, net_instance) = passeri::new_sender::<tcp::TcpSender>(0, listening_addr)
        .unwrap_or_else(|err| {
            println!("Error while trying to create binding: {}", err);
            exit(1);
        });

    println!("{}", net_instance.info());

    match net_instance.wait_for_client() {
        Ok(addr) => {
            println!("{} is now connected", addr);
            match net_instance.send(addr) {
                Ok(thread_resp) => println!("the net thread ended: {:?}", thread_resp),
                Err(err) => println!("err: {}", err),
            }
        }
        Err(err) => println!("{}", err),
    }
}
