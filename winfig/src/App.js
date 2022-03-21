import React from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { appWindow, LogicalPosition } from "@tauri-apps/api/window";

class App extends React.Component {
  async componentDidMount() {
    await appWindow.setAlwaysOnTop(true);
    // listen for window info updates
    await listen("wininfo", (event) => {
      // only set position if caret exists
      if (event.payload.caret_pos.w !== 0) {
        appWindow.setPosition(
          new LogicalPosition(
            event.payload.caret_pos.x,
            event.payload.caret_pos.y + event.payload.caret_pos.h
          )
        );
      }

      this.setState({
        window_id: event.payload.window_id,
        process_id: event.payload.process_id,
        caret_pos: {
          x: event.payload.caret_pos.x,
          y: event.payload.caret_pos.y,
          w: event.payload.caret_pos.w,
          h: event.payload.caret_pos.h,
        },
        window_pos: {
          x: event.payload.window_pos.x,
          y: event.payload.window_pos.y,
          w: event.payload.window_pos.w,
          h: event.payload.window_pos.h,
        },
      });
    });

    // listen for messages from figterm
    await listen("figterm", (event) => {
      this.setState({
        figterm_ipc_msg: event.payload,
      });
    });

    // listen for session_id chagnes
    await listen("session_id", (event) => {
      this.setState({
        session_id: event.payload,
      });
    });
  }

  constructor() {
    super();
    // spawn window observer and socket listener
    invoke("window_stream");
    invoke("socket_listener");
    this.state = {
      window_id: 0,
      process_id: 0,
      window_pos: { x: 0, y: 0, w: 0, h: 0 },
      caret_pos: { x: 0, y: 0, w: 0, h: 0 },
      figterm_ipc_msg: "",
      session_id: "",
    };
  }

  render() {
    let {
      figterm_ipc_msg,
      window_id,
      process_id,
      window_pos,
      caret_pos,
      session_id,
    } = this.state;

    return (
      <div style={{ marginLeft: 30, marginRight: 30, fontSize: 25 }}>
        <div style={{ fontSize: 40, marginBottom: 10 }}>Fig</div>
        <div>{`Window Id - ${window_id}`}</div>
        <div>{`Process Id - ${process_id}`}</div>
        <div>{`Session Id - ${session_id}`}</div>
        <div>{`Window Position - x: ${window_pos.x} y: ${window_pos.y} w: ${window_pos.w} h :${window_pos.h}`}</div>
        <div>{`Caret Position - x: ${caret_pos.x} y: ${caret_pos.y} w: ${caret_pos.w} h :${caret_pos.h}`}</div>
        <div style={{ marginTop: 20 }}>{`Last figterm msg:`}</div>
        <div style={{ fontSize: 16 }}>{`${figterm_ipc_msg}`}</div>
        <button
          onClick={() => {
            invoke("insert_text", {
              sessionId: session_id,
              text: "Hello World!",
            });
          }}
        >
          Insert Hello World
        </button>
      </div>
    );
  }
}

export default App;
