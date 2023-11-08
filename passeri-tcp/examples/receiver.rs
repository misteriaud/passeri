use std::env;
use std::{net::SocketAddr, process::exit};

use std::str::FromStr;

use log::{error, info};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        error!("Usage:\n\treceiver <address> <midi_out_port_name>");
        exit(1);
    }

    let addr = SocketAddr::from_str(&args[1]).expect("error while parsing address argument");

    let mut receiver = passeri_api::new_receiver::<passeri_tcp::Receiver>(1, args.remove(2), addr)
        .unwrap_or_else(|err| {
            error!(
                "Err: unable to initialize Receiver on address \"{}\" ({})",
                &args[1], err
            );
            exit(1);
        });

    receiver.receive().unwrap_or_else(|err| {
        error!("error trying to receive from Sender: {}", err);
        exit(1);
    });

    if let Ok(result) = receiver.join() {
        info!("Net thread return {:?}", result);
    }
}
