use std::{error::Error, time::Duration};

use btleplug::api::CharPropFlags;
use btleplug::api::{
    bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};

struct Ret {}

pub struct Receiver {}

impl passeri_core::net::Receiver for Receiver {
    type Addr = Peripheral;
    type ThreadReturn = Ret;

    fn new(midi_out: midir::MidiOutputConnection, sender: Peripheral) -> Result<Self, ()> {
        Err(())
        // let thread
        // let manager = btleplug::platform::Manager::new().unwrap();
        // let central = manager
        //     .adapters()
        //     .unwrap()
        //     .into_iter()
        //     .next()
        //     .expect("No Bluetooth adapter available.");

        // central.connect().unwrap();
        // central.start_scan().unwrap();
    }

    fn receive(self) -> Result<Self::ThreadReturn, ()> {
        Err(())
    }

    fn info(&self) -> String {}
}

struct ReceiverThread {
    manager: Manager,
    peripheral: Peripheral,
}

use futures_util::StreamExt;
use tokio::time;
use uuid::Uuid;

// https://www.bluetooth.com/specifications/assigned-numbers/

impl ReceiverThread {
    pub async fn new(filter_name: String) -> Result<ReceiverThread, Box<dyn Error>> {
        let manager = Manager::new().await?;

        // get the first bluetooth adapter
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().nth(0).unwrap();

        // start scanning for devices
        central.start_scan(ScanFilter::default()).await?;

        // instead of waiting, you can use central.events() to get a stream which will
        // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
        time::sleep(Duration::from_secs(2)).await;

        let peripherals = central.peripherals().await?;

        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
            todo!();
            return Ok(());
        }

        // let peripheral = peripherals.into_iter().find_map(async |p| Self::find_peripheral(p, &filter_name, CharPropFlags::NOTIFY).await)
        for peripheral in peripherals.into_iter() {
            let properties = peripheral.properties().await?;
            let is_connected = peripheral.is_connected().await?;
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));
            // println!(
            //     "Peripheral {:?} is connected: {:?}",
            //     &local_name, is_connected
            // );
            // Check if it's the peripheral we want.
            if !local_name.contains(&filter_name) {
                continue;
            }
            println!("Found matching peripheral {:?}...", &local_name);
            // if !is_connected {
            //     // Connect if we aren't already connected.
            //     if let Err(err) = peripheral.connect().await {
            //         eprintln!("Error connecting to peripheral, skipping: {}", err);
            //         continue;
            //     }
            // }
            let is_connected = peripheral.is_connected().await?;
            println!(
                "Now connected ({:?}) to peripheral {:?}.",
                is_connected, &local_name
            );
            if !is_connected {
                continue;
            }
            println!("Discover peripheral {:?} services...", local_name);
            peripheral.discover_services().await?;

            if peripheral
                .characteristics()
                .into_iter()
                .find(|char| char.properties.contains(CharPropFlags::NOTIFY))
                .is_none()
            {
                continue;
            }

            return Ok(ReceiverThread {
                manager,
                peripheral,
            });
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
            // println!("Disconnecting from peripheral {:?}...", local_name);
            // peripheral.disconnect().await?;
        }
        todo!();
    }

    // async fn find_peripheral(
    //     peripheral: Peripheral,
    //     name: &str,
    //     flag: CharPropFlags,
    // ) -> Option<Peripheral> {
    //     let properties = peripheral.properties().await.ok()?;
    //     // let is_connected = peripheral.is_connected().await.ok()?;
    //     let local_name = properties
    //         .unwrap()
    //         .local_name
    //         .unwrap_or(String::from("(peripheral name unknown)"));
    //     // println!(
    //     //     "Peripheral {:?} is connected: {:?}",
    //     //     &local_name, is_connected
    //     // );
    //     // Check if it's the peripheral we want.
    //     if !local_name.contains(name) {
    //         return None;
    //     }
    //     println!("Found matching peripheral {:?}...", &local_name);
    //     // if !is_connected {
    //     //     // Connect if we aren't already connected.
    //     //     if let Err(err) = peripheral.connect().await {
    //     //         eprintln!("Error connecting to peripheral, skipping: {}", err);
    //     //         continue;
    //     //     }
    //     // }
    //     let is_connected = peripheral.is_connected().await.ok()?;
    //     println!(
    //         "Now connected ({:?}) to peripheral {:?}.",
    //         is_connected, &local_name
    //     );
    //     if !is_connected {
    //         return None;
    //     }
    //     println!("Discover peripheral {:?} services...", local_name);
    //     peripheral.discover_services().await.ok()?;

    //     peripheral
    //         .characteristics()
    //         .iter()
    //         .find(|char| char.properties.contains(flag))?;

    //     Some(peripheral)
    // }
}
