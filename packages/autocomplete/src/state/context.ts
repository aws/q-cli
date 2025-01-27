import { createContext } from "react";
import { AutocompleteStore } from ".";

export const AutocompleteContext = createContext<AutocompleteStore | null>(
  null,
);
