use std::net::{Ipv4Addr, SocketAddr};
use std::process::exit;
use std::str::FromStr;

use passeri::{
    builder,
    messenger_thread::{tcp, Receiver, Sender},
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Listen for the incomming session invitation, automatically accept and start to forward RTP to midi port
    Send {
        /// address used to stream your midi, if undefined it will be choose by the operating system
        addr: Option<String>,
    },

    /// send request to provided address, start to stream if accepted
    Receive {
        /// address you want to listen
        addr: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Send { addr } => {
            let listening_addr = match addr {
                Some(addr) => SocketAddr::from_str(&addr).expect("error while parsing addr"),
                None => SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
            };

            let (_midi_instance, net_instance) =
                builder::new_sender::<tcp::TcpSender>(0, listening_addr).unwrap_or_else(|err| {
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

        Commands::Receive { addr } => {
            let addr = SocketAddr::from_str(&addr).expect("error while parsing addr");

            let net_instance =
                builder::new_receiver::<tcp::TcpReceiver>(1, addr).unwrap_or_else(|err| {
                    println!("Error while trying to create binding: {}", err);
                    exit(1);
                });
            match net_instance.receive() {
                Ok(thread_resp) => println!("the net thread ended: {:?}", thread_resp),
                Err(err) => println!("{}", err),
            }
        }
    }
}
