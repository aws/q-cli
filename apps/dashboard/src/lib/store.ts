import { create } from "zustand";
import {
  Settings as ApiSettings,
  State as ApiState,
  Auth
} from "@withfig/api-bindings";

type KV = Record<string, unknown>;

export interface Data {
  settings: KV;
  state: KV;
}

export interface Actions {
  setSetting: (key: string, value: unknown) => Promise<void>;
  setState: (key: string, value: unknown) => Promise<void>;
  isAuthed: boolean;
}

export type State = Data & Actions;

export type Store = ReturnType<typeof createStore>;

export const createStore = () => {
  const store = create<State>()((set) => ({
    settings: {},
    state: {},
    setSetting: async (key, value) => {
      set((s) => ({ settings: { ...s.settings, [key]: value } }));
      await ApiSettings.set(key, value);
    },
    setState: async (key, value) => {
      set((s) => ({ state: { ...s.state, [key]: value } }));
      await ApiState.set(key, value);
    },
    isAuthed: false,
  }));

  ApiSettings.current().then((settings) => {
    store.setState({ settings });
  });

  ApiState.current().then((state) => {
    store.setState({ state });
  });

  ApiSettings.didChange.subscribe((notification) => {
    const json = JSON.parse(notification.jsonBlob ?? "{}");
    store.setState({
      settings: json,
    });
    return { unsubscribe: false };
  });

  ApiState.didChange.subscribe((notification) => {
    const json = JSON.parse(notification.jsonBlob ?? "{}");
    store.setState({
      state: json,
    });
    return { unsubscribe: false };
  });

  

  return store;
};
