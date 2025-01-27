import React, { useCallback, useRef } from "react";
import Autocomplete from "./Autocomplete";
import Emittery from "emittery";
import { AutocompleteConnectionType } from "./state/types";

class WebsocktShim {
  private emitter: Emittery = new Emittery();

  constructor(private websocket: WebSocket) {
    websocket.onmessage = async (event) => {
      if (event.data instanceof Blob) {
        const message = {
          ...event,
          data: await event.data.text(),
        };
        this.emitter.emit("autocompleteMessage", message);
      }
    };
  }

  public on(
    event: "autocompleteMessage",
    listener: (data: { data: string }) => void,
  ) {
    this.emitter.on(event, listener);
    return this;
  }

  public send(data: string): void {
    this.websocket.send(data);
  }

  public close() {
    this.websocket.close();
  }
}

export function Test() {
  const websocket = useRef<WebsocktShim>();
  if (!websocket.current) {
    const inner = new WebSocket("ws://localhost:8080");
    inner.onopen = () => console.log("ws opened");
    inner.onclose = () => console.log("ws closed");
    websocket.current = new WebsocktShim(inner);
  }

  // const [avisibilityCallback, asetVisibilityCallback] = useState<
  //   undefined | ((visible: boolean) => Promise<void> | void)
  // >(undefined);

  const setVisibilityCallback = useCallback((callback: unknown) => {
    console.log("Hey!", callback);
    // asetVisibilityCallback(callback);
  }, []);

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-row gap-4 p-2">
        <button
          className="p-2 border rounded bg-blue-300"
          onClick={() => {
            console.log("HIDE");
            // if (visibilityCallback) visibilityCallback(false);
          }}
        >
          Hide
        </button>
        <button
          className="p-2 border rounded bg-blue-300"
          onClick={() => {
            console.log("SHOW");
            // if (visibilityCallback) visibilityCallback(true);
          }}
        >
          Show
        </button>
        <input></input>
      </div>

      {websocket && (
        <Autocomplete
          ipcClient={{
            type: AutocompleteConnectionType.CS_WEBSOCKET,
            websocket: websocket.current,
          }}
          onDisconnect={() => {
            console.error("DISCONNECT!");
          }}
          setVisibilityCallback={setVisibilityCallback}
        />
      )}
    </div>
  );
}
