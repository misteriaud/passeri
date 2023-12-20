#![warn(missing_docs)]
//! Implementation of the Sender and Receiver traits from `passeri-api`

mod tcp_receiver;
pub use tcp_receiver::Receiver;
mod tcp_sender;

pub use tcp_sender::Sender;

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::time::Duration;
    use std::{thread, vec};

    use log::debug;
    use std::fs::File;
    use std::io::prelude::*;
    use std::sync::mpsc::RecvTimeoutError;

    #[test]
    fn it_works() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

        //read sysex file
        let mut f = File::open("file.syx").unwrap();
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        let (tx, rx) = channel();

        let src: Arc<Vec<u8>> = Arc::new(buffer);
        let src_cpy = Arc::clone(&src);

        let sender = thread::spawn(move || {
            // create fake Midi Source
            let mut midi_src = passeri_api::midi::new_sender(0, "PASSERI_FAKE_SENDER").unwrap();
            debug!("fake midi source created");

            // create passeri_sender
            let addr = SocketAddr::from_str("0.0.0.0:0000").unwrap();
            let mut sender =
                passeri_api::new_sender::<crate::Sender>(0, "PASSERI_SENDER", addr).unwrap();
            debug!("passeri_sender created");

            // send sender address to passeri_receiver
            let (init_tx, init_rx) = oneshot::channel::<()>();
            tx.send((sender.info(), init_tx))
                .expect("Unable to send on channel");

            let _ = init_rx.recv();

            // wait for passeri_receiver to connect
            let client = sender.wait_for_client().unwrap();

            // start forwarding midi message
            sender.send(client).unwrap();
            debug!("passeri_sender connected to passeri_receiver");

            // send mocking value to fake midi source
            debug!("passeri_sender start to send src vec");
            for msg in src_cpy.as_ref().chunks(32) {
                midi_src.send(msg);
                std::thread::sleep(std::time::Duration::from_micros(1));
            }
            debug!("passeri_sender finished to send src vec");
        });

        let receiver = thread::spawn(move || {
            let (sender_addr, responder) = rx.recv().expect("Unable to receive from channel");
            let mut receiver =
                passeri_api::new_receiver::<crate::Receiver>(1, "PASSERI_RECV", sender_addr)
                    .unwrap();
            debug!("passeri_receiver created");

            let mut res: Vec<u8> = vec![];
            let (fake_midi_recv_conn, fake_midi_recv) =
                passeri_api::midi::new_receiver(0, "PASSERI_FAKE_RECV").unwrap();
            debug!("fake midi receiver created");

            receiver.receive().unwrap();
            responder.send(());

            debug!("passeri_receiver start to receive from passeri_sender");
            loop {
                match fake_midi_recv.recv_timeout(Duration::from_secs(1)) {
                    Ok((_, mut msg)) => {
                        debug!("message received");
                        res.append(&mut msg)
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        debug!("check if passeri_receiver still alive");
                        if receiver.is_finished() {
                            break;
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            debug!("passeri_receiver ended");
            res
        });

        sender.join().expect("The sender thread has panicked");
        let dest: Vec<u8> = receiver.join().expect("The receiver thread has panicked");
        assert_eq!(*src, *dest);
    }
}
