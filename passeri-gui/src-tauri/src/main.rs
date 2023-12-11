// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::fmt::format;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Mutex;

struct State {
    sender: HashMap<Uuid, passeri_api::net::Sender<passeri_tcp::Sender>>,
    receiver: HashMap<Uuid, passeri_api::net::Receiver>,
    // receiver: <passeri_api::net::Receiver>,
}

impl State {
    pub fn new() -> Self {
        State {
            sender: HashMap::new(),
            receiver: HashMap::new(),
            // receiver: vec![],
        }
    }
}

use std::str::FromStr;

use uuid::Uuid;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn new_bridge(
    locked_state: tauri::State<Mutex<State>>,
    bridge_type: u8,
    addr: String,
    midi_port_name: String,
) -> Result<(String, String), String> {
    let addr = SocketAddr::from_str(&addr).map_err(|err| format!("{}", err))?;

    let id = Uuid::new_v4();

    match bridge_type {
        0 => {
            // Sender
            let sender = passeri_api::new_sender(0, &midi_port_name, addr)
                .map_err(|err| format!("{}", err))?;

            let addr = sender.info();

            let mut state = locked_state.lock().unwrap();

            let mut_state = state.deref_mut();
            mut_state.sender.insert(id, sender);

            Ok((id.to_string(), addr))
        }
        _ => {
            // Receiver
            let receiver =
                passeri_api::new_receiver::<passeri_tcp::Receiver>(0, &midi_port_name, addr)
                    .map_err(|err| format!("{}", err))?;

            let addr = receiver.info();

            let mut state = locked_state.lock().unwrap();

            let mut_state = state.deref_mut();
            mut_state.receiver.insert(id, receiver);

            Ok((id.to_string(), addr))
        }
    }
}

#[tauri::command]
fn sender_listen(locked_state: tauri::State<Mutex<State>>, uuid: String) -> Result<(), String> {
    let mut state = locked_state.lock().unwrap();

    let mut_state = state.deref_mut();

    let sender = mut_state
        .sender
        .get_mut(&Uuid::from_str(&uuid).map_err(|err| format!("{}", err))?)
        .ok_or(format!("not found"))?;

    let addr = sender
        .wait_for_client()
        .map_err(|err| format!("err: {}", err))?;

    match sender.send(addr) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("err: {}", err)),
    }
}

#[tauri::command]
fn receiver_receive(locked_state: tauri::State<Mutex<State>>, uuid: String) -> Result<(), String> {
    let mut state = locked_state.lock().unwrap();

    let mut_state = state.deref_mut();

    mut_state
        .receiver
        .get_mut(&Uuid::from_str(&uuid).map_err(|err| format!("{}", err))?)
        .ok_or(format!("not found"))?
        .receive()
        .map_err(|err| format!("{}", err))
}

#[tauri::command]
fn remove_sender(locked_state: tauri::State<Mutex<State>>, uuid: String) -> Result<(), String> {
    let mut state = locked_state.lock().unwrap();

    let mut_state = state.deref_mut();
    mut_state
        .sender
        .remove(&Uuid::from_str(&uuid).map_err(|err| format!("{}", err))?);
    Ok(())
}

#[tauri::command]
fn remove_receiver(locked_state: tauri::State<Mutex<State>>, uuid: String) -> Result<(), String> {
    let mut state = locked_state.lock().unwrap();

    let mut_state = state.deref_mut();
    mut_state
        .receiver
        .remove(&Uuid::from_str(&uuid).map_err(|err| format!("{}", err))?);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(State::new()))
        .invoke_handler(tauri::generate_handler![
            new_bridge,
            sender_listen,
            receiver_receive,
            remove_sender,
            remove_receiver
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
