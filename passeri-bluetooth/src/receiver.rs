use std::collections::HashMap;
use std::sync::mpsc;
use std::thread::JoinHandle;

// use btleplug::api::CharPropFlags;
// use btleplug::api::{
//     bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter, WriteType,
// };
// use btleplug::platform::{Adapter, Manager, Peripheral};

use log::{info, trace};
use midir::MidiOutputConnection;
use passeri_core::midi::MidiFrame;
use passeri_core::net::receiver::{self, Request, Responder, Response};
use passeri_core::net::Result;

use crate::connection::BLEAddr;
use crate::ThreadReturn;
type PasseriReq = (Request, Responder);

pub struct Receiver {
    thread: JoinHandle<ThreadReturn<()>>,
    tx: mpsc::Sender<PasseriReq>,
}

impl passeri_core::net::Receiver for Receiver {
    type Addr = BLEAddr;
    type ThreadReturn = ThreadReturn<()>;

    fn new(midi_out: MidiOutputConnection, addr: Self::Addr) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<PasseriReq>();

        let mut socket = ReceiverThread::new(midi_out, rx, addr)?;

        let thread = std::thread::spawn(move || socket.run().unwrap_err());

        Ok(Receiver { thread, tx })
    }

    fn receive(self) -> Result<Self::ThreadReturn> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.tx.send((Request::Receive, response_sender))?;

        match response_receiver.recv()? {
            receiver::Response::StartReceiving => {
                info!("received ListenStream");
                Ok(self.thread.join().unwrap_or(ThreadReturn::JoinError))
            }
            receiver::Response::Err(err) => Err(err.into()),
        }
    }

    fn info(&self) -> String {
        match self.socket_addr {
            Some(addr) => format!("addr1: {}", addr),
            None => String::new(),
        }
    }
}

struct ReceiverThread {
    midi_tx: MidiOutputConnection,
    pub distant: BLEAddr,
    messenger_rx: mpsc::Receiver<PasseriReq>,
}

impl ReceiverThread {
    pub fn new(
        midi_tx: MidiOutputConnection,
        messenger_rx: mpsc::Receiver<PasseriReq>,
        addr: BLEAddr,
    ) -> Result<Self> {
        Ok(ReceiverThread {
            midi_tx,
            distant: addr,
            messenger_rx,
        })
    }

    pub fn run(&mut self) -> std::result::Result<(), ThreadReturn<Response>> {
        loop {
            let (req, responder) = self
                .messenger_rx
                .recv()
                .expect("unable to read from the messenger tunnel");
            match req {
                Request::Receive => self.receive(responder)?,
            }
        }
    }

    /// Starting to listen over UDP socket for
    fn receive(&mut self, responder: Responder) -> std::result::Result<(), ThreadReturn<Response>> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.async_recv(responder))
    }

    pub fn async_recv(&mut self, responder: Responder) {
        self.distant.peripheral.characteristics();

        // println!("Subscribing to characteristic {:?}", characteristic.uuid);
        self.distant.subscribe(&characteristic).await?;
        // Print the first 4 notifications received.
        let mut notification_stream = peripheral.notifications().await?.take(4);
        // Process while the BLE connection is not broken or stopped.
        while let Some(data) = notification_stream.next().await {
            println!(
                "Received data from {:?} [{:?}]: {:?}",
                local_name, data.uuid, data.value
            );
        }
    }
}

// for characteristic in peripheral.characteristics() {
//     println!("Checking characteristic {:?}", characteristic);
//     // Subscribe to notifications from the characteristic with the selected
//     // UUID.
//     // if characteristic.properties.contains(CharPropFlags::READ)
//     if characteristic.properties.contains(CharPropFlags::NOTIFY) {
//         println!("Subscribing to characteristic {:?}", characteristic.uuid);
//         peripheral.subscribe(&characteristic).await?;
//         // Print the first 4 notifications received.
//         let mut notification_stream = peripheral.notifications().await?.take(4);
//         // Process while the BLE connection is not broken or stopped.
//         while let Some(data) = notification_stream.next().await {
//             println!(
//                 "Received data from {:?} [{:?}]: {:?}",
//                 local_name, data.uuid, data.value
//             );
//         }
//     }
// }
