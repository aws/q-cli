import { StoreContext } from "@/context/zustand";
import { useContext } from "react";
import { useStore } from "zustand";

export function useAuth() {
  const store = useContext(StoreContext);
  if (!store) throw new Error("Missing StoreContext.Provider in the tree");
  return useStore(store, (state) => state.auth!);
}

export function useRefreshAuth() {
  const store = useContext(StoreContext);
  if (!store) throw new Error("Missing StoreContext.Provider in the tree");
  return useStore(store, (state) => state.refreshAuth!);
}
