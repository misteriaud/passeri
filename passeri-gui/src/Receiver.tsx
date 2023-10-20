import { invoke } from "@tauri-apps/api";
import { Receiver } from "./App";
import { stringify } from "uuid";
import { useState } from "react";

enum ReceiverState {
  Idle = "idle",
  Receiving = "receiving",
}

export default function ReceiverComp({ receiver }: { receiver: Receiver }) {
  const [state, setState] = useState(ReceiverState.Idle);

  async function receive() {
    await invoke("receiver_receive", {
      uuid: stringify(receiver.id),
    })
      .then((resp) => {
        setState(ReceiverState.Receiving);
        console.log(resp);
      })
      .catch((err) => {
        console.log(err);
      });
  }

  return (
    <div>
      <h1>{receiver.addr}</h1>
      <button onClick={() => receive()}>receive</button>
      {state}
    </div>
  );
}
