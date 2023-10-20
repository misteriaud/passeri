import { Sender } from "./App";

export default function SenderComp({ sender }: { sender: Sender }) {
  return (
    <div>
      <h1>{sender.addr}</h1>
    </div>
  );
}
