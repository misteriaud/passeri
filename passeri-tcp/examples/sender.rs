use std::{env, net::SocketAddr, process::exit, str::FromStr};

use log::{error, info};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        error!("Usage:\n\treceiver <address> <midi_in_port_name>");
        exit(1);
    }

    let addr = SocketAddr::from_str(&args[1]).expect("error while parsing address argument");

    let mut sender = passeri_api::new_sender::<passeri_tcp::Sender>(0, args.remove(2), addr)
        .unwrap_or_else(|err| {
            error!(
                "Err: unable to initialize Sender on address \"{}\" ({})",
                &args[1], err
            );
            exit(1);
        });

    if let Ok(addr) = sender.wait_for_client() {
        info!("{} is now connected", addr);
        sender.send(addr).unwrap_or_else(|err| {
            error!("error trying to receive from Sender: {}", err);
            exit(1);
        });
        if let Ok(result) = sender.join() {
            info!("Net thread return {:?}", result);
        }
    }
}
