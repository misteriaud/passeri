use std::env;
use std::{net::SocketAddr, process::exit};

use std::str::FromStr;

use log::{error, info};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("Usage:\n\treceiver <address>");
        exit(1);
    }

    let addr = SocketAddr::from_str(&args[1]).expect("error while parsing address argument");

    let receiver =
        passeri_api::new_receiver::<passeri_tcp::Receiver>(1, addr).unwrap_or_else(|err| {
            error!(
                "Err: unable to initialize Receiver on address \"{}\" ({})",
                &args[1], err
            );
            exit(1);
        });

    match receiver.receive() {
        Ok(thread_resp) => info!("the net thread ended: {:?}", thread_resp),
        Err(err) => error!("error trying to receive from Sender: {}", err),
    }
}
