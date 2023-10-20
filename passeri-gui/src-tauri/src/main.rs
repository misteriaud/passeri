// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Mutex;

struct State {
    sender: HashMap<Uuid, passeri_api::net::Sender<passeri_tcp::Sender>>,
    // receiver: <passeri_api::net::Receiver>,
}

impl State {
    pub fn new() -> Self {
        State {
            sender: HashMap::new(),
            // receiver: vec![],
        }
    }
}

use std::str::FromStr;

use uuid::Uuid;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn new_sender(
    locked_state: tauri::State<Mutex<State>>,
    addr: String,
    midi_port_name: String,
) -> Result<(String, String), String> {
    let addr = SocketAddr::from_str(&addr).map_err(|err| format!("{}", err))?;

    let id = Uuid::new_v4();
    let sender =
        passeri_api::new_sender(0, midi_port_name, addr).map_err(|err| format!("{}", err))?;

    let addr = sender.info();

    let mut state = locked_state.lock().unwrap();

    let mut_state = state.deref_mut();
    mut_state.sender.insert(id, sender);
    Ok((id.to_string(), addr))
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(State::new()))
        .invoke_handler(tauri::generate_handler![new_sender])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
