use std::env;
use std::{net::SocketAddr, process::exit};

use passeri::net::Receiver;
use passeri::tcp;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("not enough arguments");
    }

    let addr = SocketAddr::from_str(&args[1]).expect("error while parsing addr");

    let net_instance = passeri::new_receiver::<tcp::TcpReceiver>(1, addr).unwrap_or_else(|err| {
        println!("Error while trying to create binding: {}", err);
        exit(1);
    });
    match net_instance.receive() {
        Ok(thread_resp) => println!("the net thread ended: {:?}", thread_resp),
        Err(err) => println!("{}", err),
    }
}
