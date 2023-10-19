import { StoreContext } from "@/context/zustand";
import { useContext } from "react";
import { useStore } from "zustand";

export function useSetting(key: string) {
  const store = useContext(StoreContext);
  if (!store) throw new Error("Missing BearContext.Provider in the tree");
  return [
    useStore(store, (state) => state.settings[key]),
    useStore(
      store,
      (state) => (value: unknown) => state.setSetting(key, value)
    ),
  ] as const;
}
