import { useContext } from "react";
import { useStore } from "zustand";
import { AutocompleteState } from "./types";
import { AutocompleteContext } from "./context";
import { useStoreWithEqualityFn } from "zustand/traditional";

const identity = <T>(arg: T): T => arg;

export function useAutocomplete(): AutocompleteState;
export function useAutocomplete<T>(
  selector: (state: AutocompleteState) => T,
): T;
export function useAutocomplete<T>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  selector: (state: AutocompleteState) => T = identity as any,
) {
  const store = useContext(AutocompleteContext);
  if (!store) {
    throw new Error("Missing AutocompleteContext.Provider in the tree");
  }
  return useStore(store, selector);
}

export function useAutocompleteWithEqualityFn<T>(
  selector: (state: AutocompleteState) => T,
  equalityFn: (a: T, b: T) => boolean,
): T {
  const store = useContext(AutocompleteContext);
  if (!store) {
    throw new Error("Missing AutocompleteContext.Provider in the tree");
  }
  return useStoreWithEqualityFn(store, selector, equalityFn);
}
