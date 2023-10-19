import { StoreContext } from "@/context/zustand";
import { useContext } from "react";
import { useStore } from "zustand";

export function useState(key: string) {
  const store = useContext(StoreContext);
  if (!store) throw new Error("Missing BearContext.Provider in the tree");
  return [
    useStore(store, (state) => state.state[key]),
    useStore(
      store,
      (state) => (value: unknown) => state.setState(key, value)
    ),
  ] as const;
}
