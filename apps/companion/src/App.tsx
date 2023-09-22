import { useEffect, useState } from "react";
import { EditBufferNotifications, WindowPosition } from "@withfig/api-bindings";

function App() {
  const [_sessionId, setSessionId] = useState("");
  useEffect(() => {
    WindowPosition.setFrame({
      width: 300,
      height: 300,
      anchorX: 0,
      offsetFromBaseline: 10,
    });
  }, []);

  useEffect(() => {
    EditBufferNotifications.subscribe((data) => {
      setSessionId(data.sessionId ?? "");
      return { unsubscribe: false };
    });
  });

  return (
    <div className="h-full w-full p-4 text-gray-100">
      <div className="bg-gray-800 w-full h-full rounded-xl">
        <div className="flex flex-col gap-2 p-2">
          <div className="bg-gray-700 w-full rounded p-2">
            What is the best way to run an npm project?
          </div>
          <div className="bg-gray-700 w-full rounded p-2">
            Run <code className="bg-zinc-800 p-1 rounded">npm run dev</code>
          </div>
          <div className="">Chat</div>
        </div>
      </div>
    </div>
  );
}

export default App;
