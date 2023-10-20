import { invoke } from "@tauri-apps/api";
import { Receiver } from "./App";
import { stringify } from "uuid";

export default function ReceiverComp({ receiver }: { receiver: Receiver }) {
  async function remove() {
    await invoke("remove_receiver", {
      uuid: stringify(receiver.id),
    })
      .then((resp) => {
        console.log(resp);
      })
      .catch((err) => {
        console.log(err);
      });
  }

  async function receive() {
    await invoke("receiver_receive", {
      uuid: stringify(receiver.id),
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
      <h1>{receiver.addr}</h1>
      <button onClick={() => remove()}>X</button>
      <button onClick={() => receive()}>receive</button>
    </div>
  );
}
