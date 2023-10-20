import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import SenderComp from "./Sender";
import { v4 as uuidv4, parse as uuidParse } from "uuid";
import ReceiverComp from "./Receiver";

enum BridgeType {
  Sender = 0,
  Receiver = 1,
}

export class Sender {
  id: uuidv4;
  addr: String;

  constructor(id: String, addr: String) {
    this.id = uuidParse(id);
    this.addr = addr;
  }
}

export class Receiver {
  id: uuidv4;
  addr: String;

  constructor(id: String, addr: String) {
    this.id = uuidParse(id);
    this.addr = addr;
  }
}

function App() {
  const [addr, setAddr] = useState("");
  const [name, setName] = useState("");
  const [senders, setSenders] = useState<Array<Sender>>([]);
  const [receivers, setReceivers] = useState<Array<Receiver>>([]);

  //   async function greet() {
  //     // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  //     setGreetMsg();
  //   }

  async function new_bridge(type: BridgeType) {
    if (addr == "") return;
    await invoke<Array<String>>("new_bridge", {
      bridgeType: type as number,
      addr,
      midiPortName: name,
    })
      .then((resp) => {
        if (type == BridgeType.Sender) {
          setSenders([...senders, new Sender(resp[0], resp[1])]);
        } else {
          setReceivers([...receivers, new Receiver(resp[0], resp[1])]);
        }
        console.log(resp);
        setAddr("");
        setName("");
      })
      .catch((err) => {
        console.log(err);
      });
  }

  const sender_list = senders.map((sender) => (
    <li key={sender.id}>
      <SenderComp sender={sender} />
    </li>
  ));

  const receiver_list = receivers.map((receiver) => (
    <li key={receiver.id}>
      <ReceiverComp receiver={receiver} />
    </li>
  ));

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <input
        value={addr}
        onChange={(e) => setAddr(e.currentTarget.value)}
        placeholder="Enter an address"
      />
      <input
        value={name}
        onChange={(e) => setName(e.currentTarget.value)}
        placeholder="Enter a name..."
      />
      <button onClick={() => new_bridge(BridgeType.Sender)}>Sender</button>
      <button onClick={() => new_bridge(BridgeType.Receiver)}>Receiver</button>
      <ul>{sender_list}</ul>
      <ul>{receiver_list}</ul>
    </div>
  );
}

export default App;
