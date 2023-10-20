import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import SenderComp from "./Sender";
import { v4 as uuidv4 } from "uuid";

export type Sender = { id: uuidv4; addr: String };

function App() {
  const [addr, setAddr] = useState("");
  const [name, setName] = useState("");
  const [senders, setSenders] = useState<Array<Sender>>([]);
  const [receivers, setReceivers] = useState([]);

  //   async function greet() {
  //     // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  //     setGreetMsg();
  //   }

  async function new_sender() {
    if (addr == "") return;
    await invoke<Array<String>>("new_sender", { addr, midiPortName: name })
      .then((resp) => {
        console.log(resp);
        setSenders([
          ...senders,
          {
            id: uuidv4.parse(resp[0]),
            addr: resp[1],
          },
        ]);
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

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          new_sender();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setAddr(e.currentTarget.value)}
          placeholder="Enter an address"
        />
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <ul>{sender_list}</ul>
    </div>
  );
}

export default App;
