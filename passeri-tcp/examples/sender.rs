use std::{
    net::{Ipv4Addr, SocketAddr},
    process::exit,
};

fn main() {
    env_logger::init();

    let listening_addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4242);

    let sender =
        passeri_api::new_sender::<passeri_tcp::Sender>(0, listening_addr).unwrap_or_else(|err| {
            println!("Error while trying to create binding: {}", err);
            exit(1);
        });

    // println!("{}", net_instance.info());

    match sender.wait_for_client() {
        Ok(addr) => {
            println!("{} is now connected", addr);
            match sender.send(addr) {
                Ok(thread_resp) => println!("the net thread ended: {:?}", thread_resp),
                Err(err) => println!("err: {}", err),
            }
        }
        Err(err) => println!("{}", err),
    }
}
