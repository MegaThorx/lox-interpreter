import LoxEditor from "./LoxEditor.tsx";
import {useState} from "react";
import * as wasm from "lox-wasm";
import examples from "../examples.json";

enum MessageType {
  Print,
  Error
}

type Message = {
  id: string,
  type: MessageType,
  text?: string,
  occurredAt: Date,
};

function LoxPlayground() {
  const [messages, setMessages] = useState<Message[]>([])
  const [code, setCode] = useState(examples[0].code)

  const runCode = () => {
    let emittedMessages: Message[] = [];
    try {
      wasm.run(code, (message: string) =>
        emittedMessages.push({id: crypto.randomUUID(), type: MessageType.Print, text: message, occurredAt: new Date()})
      );
    } catch (exception: unknown) {
      console.error(exception);
      if (exception instanceof Error) {
        emittedMessages.push({id: crypto.randomUUID(), type: MessageType.Error, text: exception.message, occurredAt: new Date()});
      } else if (typeof exception === "string") {
        emittedMessages.push({id: crypto.randomUUID(), type: MessageType.Error, text: exception, occurredAt: new Date()});
      } else {
        console.log(typeof exception);
      }
    }

    setMessages([...messages, ...emittedMessages])
  }

  return <div className="flex flex-col w-full p-4 h-screen">
    <div className="flex pb-4">
      <div className="flex flex-row gap-4">
        <button className="btn btn-primary" onClick={() => runCode()}>Run</button>
        <select defaultValue={examples[0].name} className="select" onChange={(event) => setCode(examples.find((example) => example.name === event.target.value)?.code ?? "")}>
          {examples.map((example) => {
            return <option key={example.name} value={example.name}>{example.name}</option>;
          })}
        </select>
      </div>
    </div>
    <div className="grid grid-cols-2 gap-4 w-full h-full">
      <div className="flex flex-col w-full h-full">
        <h2 className="mb-4 text-2xl">Code</h2>
        <LoxEditor value={code} onChange={(newCode) => setCode(newCode ?? "")} />
      </div>
      <div className="flex flex-col w-full h-full">
        <h2 className="mb-4 text-2xl">Execution</h2>
        <div className="h-full bg-gray-900 rounded-lg p-4 overflow-y-scroll relative">
          <div className="absolute pb-4">
            {messages.map((message) => {
              return (<span key={message.id} className={"block" + (message.type === MessageType.Error ? " text-red-500" : "")}>{message.text}</span>);
            })}
          </div>
        </div>
      </div>
    </div>
  </div>;
}

export default LoxPlayground;