use std::env;
use std::{net::SocketAddr, process::exit};

use passeri_core::net::Receiver;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("not enough arguments");
    }

    let addr = SocketAddr::from_str(&args[1]).expect("error while parsing addr");

    let net_instance =
        passeri_core::new_receiver::<passeri_tcp::Receiver>(1, addr).unwrap_or_else(|err| {
            println!("Error while trying to create binding: {}", err);
            exit(1);
        });
    match net_instance.receive() {
        Ok(thread_resp) => println!("the net thread ended: {:?}", thread_resp),
        Err(err) => println!("{}", err),
    }
}
