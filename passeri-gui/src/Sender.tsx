import { invoke } from "@tauri-apps/api";
import { Sender } from "./App";

import { stringify } from "uuid";

export default function SenderComp({ sender }: { sender: Sender }) {
  async function remove() {
    await invoke("remove_sender", {
      uuid: stringify(sender.id),
    })
      .then((resp) => {
        console.log(resp);
      })
      .catch((err) => {
        console.log(err);
      });
  }

  async function listen() {
    await invoke("sender_listen", {
      uuid: stringify(sender.id),
    })
      .then((resp) => {
        console.log(resp);
      })
      .catch((err) => {
        console.log(err);
      });
  }

  return (
    <div>
      <h1>{sender.addr}</h1>
      <button onClick={() => remove()}>X</button>
      <button onClick={() => listen()}>listen</button>
    </div>
  );
}
