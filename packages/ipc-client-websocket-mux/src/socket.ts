export interface AutocompleteMessage {
  data: string;
}

export interface CsWebsocket {
  on(
    event: "autocompleteMessage",
    listener: (message: AutocompleteMessage) => void,
  ): this;
  send(message: string): void;
}

type WebsocketKind =
  | {
      type: "websocket";
      socket: WebSocket;
    }
  | {
      type: "cswebsocket";
      socket: CsWebsocket;
    };

export class Socket {
  websocketKind: WebsocketKind;
  active: boolean;

  private constructor(websocketKind: WebsocketKind) {
    this.websocketKind = websocketKind;
    this.active = true;
  }

  static real(websocket: WebSocket): Socket {
    return new Socket({ type: "websocket", socket: websocket });
  }

  static cs(cswebsocket: CsWebsocket): Socket {
    return new Socket({ type: "cswebsocket", socket: cswebsocket });
  }

  onMessage(listener: (message: string) => void) {
    if (this.active) {
      if (this.websocketKind.type === "cswebsocket") {
        this.websocketKind.socket.on("autocompleteMessage", ({ data }) => {
          listener(data);
        });
      } else {
        this.websocketKind.socket.addEventListener("message", (event) => {
          const chunk = new Uint8Array(event.data);
          const message = new TextDecoder().decode(chunk);
          listener(message);
        });
      }
    }
  }

  onClose(listener: () => void) {
    this.active = false;
    if (this.websocketKind.type === "cswebsocket") {
      // this.websocketKind.socket.on("close", listener)
    } else {
      this.websocketKind.socket.addEventListener("close", listener);
    }
  }

  send(message: string): void {
    if (this.active) {
      if (this.websocketKind.type === "cswebsocket") {
        this.websocketKind.socket.send(message);
      } else {
        this.websocketKind.socket.send(message);
      }
    }
  }

  close() {
    this.active = false;
    if (this.websocketKind.type === "cswebsocket") {
      // this.websocketKind.socket.close()
    } else {
      this.websocketKind.socket.close();
    }
  }
}
