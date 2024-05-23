import { createContext } from "react";

export const GeneratingContext = createContext({
  generating: false,
  stopGenerating: () => {},
});
