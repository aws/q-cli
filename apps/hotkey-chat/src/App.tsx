import { useEffect } from "react";
import Draggable from "./components/Draggable";
import Chat from "./components/Chat";
import { Screen } from "@amzn/fig-io-api-bindings";

const App = () => {
  useEffect(() => {
    window.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      Screen.openContextMenu({ x: e.clientX, y: e.clientY });
    });

    return window.removeEventListener("contextmenu", (e) => {
      e.preventDefault();
      Screen.openContextMenu({ x: e.clientX, y: e.clientY });
    });
  }, []);

  return (
    <Draggable>
      <Chat />
    </Draggable>
  );
};

export default App;
