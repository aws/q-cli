import { WindowPosition } from "@amzn/fig-io-api-bindings";
import { ReactNode } from "react";

const Draggable = ({ children }: { children: ReactNode }) => {
  return (
    <div
      className="dark:text-white text-black w-full flex bg-white dark:bg-black opacity-95 rounded-lg max-h-[500px]"
      onMouseDown={(_) => WindowPosition.dragWindow()}
    >
      <div
        className="m-2.5 w-full max-h-[500px] relative overflow-auto"
        onMouseDown={(event) => event.stopPropagation()}
      >
        {children}
      </div>
    </div>
  );
};

export default Draggable;
