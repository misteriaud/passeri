use passeri::messenger_thread::Messenger;
use passeri::rtp_midi::ControlPacket;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::exit;
use std::str::FromStr;

use passeri::{
    builder,
    messenger_thread::{self, ip_messenger::IpMessenger},
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Name with which you will be define for the choosen command
    name: String,
    addr: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Listen for the incomming session invitation, automatically accept and start to forward RTP to midi port
    Listen,

    /// send request to provided address, start to stream if accepted
    Send {
        /// lists test values
        addr: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let listening_addr = match cli.addr {
        Some(addr) => SocketAddr::from_str(&addr).expect("error while parsing addr"),
        None => SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4242),
    };

    let (_midi_instance, net_instance) = builder::new_sender::<IpMessenger>(0, listening_addr)
        .unwrap_or_else(|err| {
            println!("Error while trying to create binding: {}", err);
            exit(1);
        });

    println!("{}", net_instance.info());

    match &cli.command {
        Some(Commands::Listen) => {
            match net_instance.req(messenger_thread::Request::WaitForInvitation) {
                Ok(resp) => println!("{:?}", resp),
                Err(err) => println!("{}", err),
            }
        }

        Some(Commands::Send { addr }) => {
            let addr = SocketAddr::from_str(&addr).expect("error while parsing addr");

            match net_instance.req(messenger_thread::Request::InviteSomeone((
                addr,
                ControlPacket::new(cli.name),
            ))) {
                Ok(resp) => println!("{:?}", resp),
                Err(err) => println!("{}", err),
            }
        }
        None => {}
    }
}
