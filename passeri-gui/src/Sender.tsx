import { invoke } from "@tauri-apps/api";
import { Sender } from "./App";

import { stringify } from "uuid";
import { useState } from "react";

enum SenderState {
  Idle = "idle",
  Listening = "listening",
}

export default function SenderComp({ sender }: { sender: Sender }) {
  const [state, setState] = useState(SenderState.Idle);

  async function listen() {
    await invoke("sender_listen", {
      uuid: stringify(sender.id),
    })
      .then((resp) => {
        setState(SenderState.Listening);
        console.log(resp);
      })
      .catch((err) => {
        console.log(err);
      });
  }

  return (
    <div>
      <h1>{sender.addr}</h1>
      <button onClick={() => listen()}>listen</button>
      {state}
    </div>
  );
}
