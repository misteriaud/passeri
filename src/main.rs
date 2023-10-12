use passeri::messenger_thread::Messenger;
use passeri::rtp_midi::ControlPacket;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::exit;
use std::str::FromStr;

use passeri::{
    builder,
    messenger_thread::{self, tcp_messenger::TcpMessenger},
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Name with which you will be define for the choosen command
    // name: String,
    addr: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Listen for the incomming session invitation, automatically accept and start to forward RTP to midi port
    Send,

    /// send request to provided address, start to stream if accepted
    Receive {
        /// lists test values
        addr: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Send) => {
            let listening_addr = match cli.addr {
                Some(addr) => SocketAddr::from_str(&addr).expect("error while parsing addr"),
                None => SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
            };

            let (_midi_instance, net_instance) =
                builder::new_sender::<TcpMessenger>(0, listening_addr).unwrap_or_else(|err| {
                    println!("Error while trying to create binding: {}", err);
                    exit(1);
                });

            println!("{}", net_instance.info());

            match net_instance.req(messenger_thread::Request::OpenRoom) {
                Ok(resp) => match resp {
                    messenger_thread::Response::NewClient(addr) => {
                        println!("{} is now connected", addr);
                        match net_instance
                            .req(messenger_thread::Request::AcceptClient)
                            .unwrap()
                        {
                            messenger_thread::Response::HasHangUp => {
                                println!("finished: has hang up")
                            }
                            messenger_thread::Response::Err(err) => println!("err: {}", err),
                            _ => {}
                        }
                    }
                    _ => {}
                },
                Err(err) => println!("{}", err),
            }
        }

        Some(Commands::Receive { addr }) => {
            let addr = SocketAddr::from_str(&addr).expect("error while parsing addr");

            let net_instance = builder::new_receiver::<TcpMessenger>(0).unwrap_or_else(|err| {
                println!("Error while trying to create binding: {}", err);
                exit(1);
            });

            match net_instance.req(messenger_thread::Request::JoinRoom(addr)) {
                Ok(resp) => match resp {
                    messenger_thread::Response::Err(err) => println!("err: {}", err),
                    _ => {}
                },
                Err(err) => println!("{}", err),
            }
        }
        None => {}
    }
}
