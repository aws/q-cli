import { create } from "zustand";
import {
  Settings as ApiSettings,
  State as ApiState,
  Install,
} from "@withfig/api-bindings";

type KV = Record<string, unknown>;

export interface Data {
  settings: KV;
  state: KV;

  accessibilityIsInstalled: boolean | undefined;
  dotfilesIsInstalled: boolean | undefined;
  inputMethodIsInstalled: boolean | undefined;
}

export interface Actions {
  setSetting: (key: string, value: unknown) => Promise<void>;
  setState: (key: string, value: unknown) => Promise<void>;
  refreshAccessibilityIsInstalled: () => Promise<void>;
  refreshDotfilesIsInstalled: () => Promise<void>;
  refreshInputMethodIsInstalled: () => Promise<void>;
}

export type State = Data & Actions;

export type Store = ReturnType<typeof createStore>;

export const createStore = () => {
  const store = create<State>()((set) => ({
    settings: {},
    state: {},
    accessibilityIsInstalled: undefined,
    dotfilesIsInstalled: undefined,
    inputMethodIsInstalled: undefined,
    setSetting: async (key, value) => {
      set((s) => ({ settings: { ...s.settings, [key]: value } }));
      await ApiSettings.set(key, value);
    },
    setState: async (key, value) => {
      set((s) => ({ state: { ...s.state, [key]: value } }));
      await ApiState.set(key, value);
    },
    refreshAccessibilityIsInstalled: async () => {
      const accessibilityIsInstalled =
        await Install.isInstalled("accessibility");
      set(() => ({ accessibilityIsInstalled }));
    },
    refreshDotfilesIsInstalled: async () => {
      const shellIsInstalled = await Install.isInstalled("dotfiles");
      set(() => ({ dotfilesIsInstalled: shellIsInstalled }));
    },
    refreshInputMethodIsInstalled: async () => {
      const inputMethodIsInstalled = await Install.isInstalled("inputMethod");
      set(() => ({ inputMethodIsInstalled }));
    },
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

  Install.isInstalled("accessibility").then((isInstalled) => {
    store.setState({ accessibilityIsInstalled: isInstalled });
  });

  Install.isInstalled("dotfiles").then((isInstalled) => {
    store.setState({ dotfilesIsInstalled: isInstalled });
  });

  Install.isInstalled("inputMethod").then((isInstalled) => {
    store.setState({ inputMethodIsInstalled: isInstalled });
  });

  Install.installStatus.subscribe("accessibility", (isInstalled) => {
    store.setState({ accessibilityIsInstalled: isInstalled });
    return { unsubscribe: false };
  });

  return store;
};
