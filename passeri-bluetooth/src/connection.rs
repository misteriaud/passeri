use std::{collections::HashMap, time::Duration};

// use futures_util::StreamExt;
use crate::ThreadReturn;
use btleplug::{
    api::{CharPropFlags, ScanFilter},
    platform::{Adapter, Manager, Peripheral},
};
use log::{error, info, warn};
use tokio::time;

pub struct BLEAddr {
    manager: Manager,
    adapter: Adapter,
    pub peripheral: Peripheral,
}

impl BLEAddr {
    pub fn new(conn: BLEConn, peripheral: Peripheral) -> Self {
        BLEAddr {
            manager: conn.manager,
            adapter: conn.adapter,
            peripheral,
        }
    }

    pub async fn get_first_match(
        filter_name: &str,
        prop_flag: CharPropFlags,
    ) -> Result<Self, btleplug::Error> {
        let conn = BLEConn::new().await?;
        Ok(BLEAddr {
            peripheral: conn
                .get_matching_periph(filter_name, prop_flag, Duration::from_secs(2))
                .await?
                .into_iter()
                .nth(0)
                .ok_or(btleplug::Error::DeviceNotFound)?
                .1,
            manager: conn.manager,
            adapter: conn.adapter,
        })
    }

    pub fn get_characteristics(&self) {
        self.peripheral.characteristics()
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
// println!("Disconnecting from peripheral {:?}...", local_name);
// peripheral.disconnect().await?;

pub struct BLEConn {
    manager: Manager,
    adapter: Adapter,
}

// https://www.bluetooth.com/specifications/assigned-numbers/

impl BLEConn {
    pub async fn new() -> Result<Self, btleplug::Error> {
        // create the manager
        let manager = Manager::new().await?;

        // get the first bluetooth adapter
        let adapters = manager.adapters().await?;

        Ok(BLEConn {
            manager,
            adapter: adapters
                .into_iter()
                .nth(0)
                .ok_or(btleplug::Error::DeviceNotFound)?,
        })
    }

    pub async fn get_matching_periph(
        &self,
        filter_name: &str,
        prop_flag: CharPropFlags,
        scan_time: Duration,
    ) -> Result<HashMap<String, Peripheral>, btleplug::Error> {
        // start scanning for devices
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|err| ThreadReturn::BleErr(err))?;

        // instead of waiting, you can use central.events() to get a stream which will
        // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
        time::sleep(scan_time).await;

        let peripherals = self
            .adapter
            .peripherals()
            .await
            .map_err(|err| ThreadReturn::BleErr(err))?;

        if peripherals.is_empty() {
            error!("->>> BLE peripheral devices were not found, sorry. Exiting...");
            return Err(ThreadReturn::NoMatchingClient);
            // return Ok(());
        }

        let map = HashMap::<String, Peripheral>::new();

        // let peripheral = peripherals.into_iter().find_map(async |p| Self::find_peripheral(p, &filter_name, CharPropFlags::NOTIFY).await)
        for peripheral in peripherals.into_iter() {
            let Ok(Some(properties)) = peripheral.properties().await else {
                warn!("cannot access properties.");
                continue;
            };
            let Ok(is_connected) = peripheral.is_connected().await else {
                warn!("is not connected.");
                continue;
            };
            let Some(local_name) = properties.local_name else {
                warn!("name unknown.");
                continue;
            };

            if !local_name.contains(&filter_name) {
                continue;
            }
            info!("Found matching peripheral {:?}...", &local_name);

            let Ok(is_connected) = peripheral.is_connected().await else {
                warn!("unable to get connection status");
                continue;
            };
            if !is_connected {
                continue;
            }

            println!("Discover peripheral {:?} services...", local_name);
            if peripheral.discover_services().await.is_err() {
                warn!("unable to discover services");
                continue;
            }

            if peripheral
                .characteristics()
                .into_iter()
                .find(|char| char.properties.contains(prop_flag))
                .is_none()
            {
                warn!("doesn't have the provided prop_flags");
                continue;
            }

            map.insert(local_name, peripheral)
        }
        Ok(map)
    }
}
