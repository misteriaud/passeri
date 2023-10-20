use std::{env, net::SocketAddr, process::exit, str::FromStr};

use log::{error, info};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("Usage:\n\tsender <address>");
        exit(1);
    }

    let addr = SocketAddr::from_str(&args[1]).expect("error while parsing address argument");

    let sender = passeri_api::new_sender::<passeri_tcp::Sender>(0, addr).unwrap_or_else(|err| {
        error!(
            "Err: unable to initialize Sender on address \"{}\" ({})",
            &args[1], err
        );
        exit(1);
    });

    match sender.wait_for_client() {
        Ok(addr) => {
            info!("{} is now connected", addr);
            match sender.send(addr) {
                Ok(thread_resp) => info!("the net thread ended: {:?}", thread_resp),
                Err(err) => error!("err: {}", err),
            }
        }
        Err(err) => error!("error trying to wait_for_client: {}", err),
    }
}
