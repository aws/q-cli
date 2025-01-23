import React, { useEffect } from "react";
import {
  EditBufferNotifications,
  Keybindings,
  Types,
  Event,
  Settings,
} from "@aws/amazon-q-developer-cli-api-bindings";
import { AliasMap } from "@aws/amazon-q-developer-cli-shell-parser";
import { IpcClient } from "@aws/amazon-q-developer-cli-ipc-client-core";
import { create } from "@bufbuild/protobuf";
import { KeybindingPressedNotificationSchema } from "@aws/amazon-q-developer-cli-proto/fig";
import {
  SettingsMap,
  updateSettings,
} from "@aws/amazon-q-developer-cli-api-bindings-wrappers";
import { updateSelectSuggestionKeybindings } from "../actions";
import { generatorCache } from "../generators/helpers";
import { clearSpecIndex } from "@aws/amazon-q-developer-cli-autocomplete-parser";

// TODO(sean) expose Subscription type from API binding library
type Unwrap<T> = T extends Promise<infer U> ? U : T;
type Subscription = Unwrap<
  NonNullable<ReturnType<(typeof EditBufferNotifications)["subscribe"]>>
>;

export type FigState = {
  buffer: string;
  cursorLocation: number;
  cwd: string | null;
  processUserIsIn: string | null;
  sshContextString: string | null;
  aliases: AliasMap;
  environmentVariables: Record<string, string>;
  shellContext?: Types.ShellContext | undefined;
};

export const initialFigState: FigState = {
  buffer: "",
  cursorLocation: 0,
  cwd: null,
  processUserIsIn: null,
  sshContextString: null,
  aliases: {},
  environmentVariables: {},
  shellContext: undefined,
};

export const useFigSubscriptionEffect = (
  getSubscription: () => Promise<Subscription> | undefined,
  deps?: React.DependencyList,
) => {
  useEffect(() => {
    let unsubscribe: () => void;
    let isStale = false;
    // if the component is unmounted before the subscription is awaited we
    // unsubscribe from the event
    getSubscription()?.then((result) => {
      unsubscribe = result.unsubscribe;
      if (isStale) unsubscribe();
    });
    return () => {
      if (unsubscribe) unsubscribe();
      isStale = true;
    };
  }, deps);
};

export const useFigSettings = (
  setSettings: React.Dispatch<React.SetStateAction<Record<string, unknown>>>,
) => {
  useEffect(() => {
    Settings.current().then((settings) => {
      setSettings(settings);
      updateSettings(settings as SettingsMap);
      updateSelectSuggestionKeybindings(settings as SettingsMap);
    });
  }, [setSettings]);
  useFigSubscriptionEffect(
    () =>
      Settings.didChange.subscribe((notification) => {
        const settings = JSON.parse(notification.jsonBlob ?? "{}");
        setSettings(settings);
        updateSettings(settings);
        updateSelectSuggestionKeybindings(settings as SettingsMap);
        return { unsubscribe: false };
      }),
    [],
  );
};

export const useFigKeypress = (
  keypressCallback: Parameters<typeof Keybindings.pressed>[0],
  ipcClient?: IpcClient,
) => {
  useEffect(() => {
    return ipcClient?.onInterceptedKey((keyHook) => {
      keypressCallback(
        create(KeybindingPressedNotificationSchema, {
          action: keyHook.action,
          context: keyHook.context,
        }),
      );
    });
  }, [ipcClient, keypressCallback]);
};

export const useFigAutocomplete = (
  setFigState: React.Dispatch<React.SetStateAction<FigState>>,
  ipcClient?: IpcClient,
) => {
  useEffect(() => {
    return ipcClient?.onEditBufferChange((notification) => {
      const buffer = notification.text ?? "";
      const cursorLocation = Number(notification.cursor);

      const cwd = notification.context?.currentWorkingDirectory ?? null;
      const shellContext = notification.context;
      setFigState((figState) => ({
        ...figState,
        buffer,
        cursorLocation,
        cwd,
        shellContext,
      }));
    });
  }, [ipcClient, setFigState]);
};

export const useFigClearCache = () => {
  useFigSubscriptionEffect(() =>
    Event.subscribe("clear-cache", () => {
      console.log("clearing cache");
      window.resetCaches?.();
      generatorCache.clear();
      clearSpecIndex();
      return { unsubscribe: false };
    }),
  );
};
