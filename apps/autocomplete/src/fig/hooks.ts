import React, { useEffect } from "react";
import logger from "loglevel";
import {
  EditBufferNotifications,
  Keybindings,
  Settings,
  Shell,
  Types,
} from "@withfig/api-bindings";
import { AliasMap } from "@amzn/fig-io-shell-parser";
import { splitPath } from "@internal/shared/utils";
import {
  executeLoginShell,
  SettingsMap,
  updateSettings,
} from "@amzn/fig-io-api-bindings-wrappers";
import { captureError, initSentry } from "../sentry";
import { updateSelectSuggestionKeybindings } from "../actions";

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
  cliAliases: AliasMap;
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
  cliAliases: {},
  environmentVariables: {},
  shellContext: undefined,
};

export const useLoadAliasEffect = (
  setFigState: React.Dispatch<React.SetStateAction<FigState>>,
  currentProcess?: string,
) => {
  useEffect(() => {
    window.globalCWD = "";
    window.globalTerminalSessionId = "";
    window.globalSSHString = "";
  }, []);

  useEffect(() => {
    let isStale = false;
    (async () => {
      const shell = currentProcess ?? "bash";
      const basename = splitPath(shell)[1];
      if (shell && ["fish", "bash", "zsh"].includes(basename)) {
        const separator = shell.includes("fish") ? " " : "=";
        executeLoginShell({ executable: shell, command: "alias" })
          .then((aliasData) => {
            const aliases = aliasData
              .replace(/^alias\s/gm, "")
              .split("\n")
              .reduce((acc, alias) => {
                try {
                  const [key, ...value] = alias.split(separator);
                  acc[key] = value
                    .join(separator)
                    .replace(/^'/, "")
                    .replace(/'$/, "");
                } catch (err) {
                  logger.error(`Error parsing alias: ${alias}`, err);
                }
                return acc;
              }, {} as AliasMap);

            if (!isStale) {
              setFigState((state) => ({ ...state, aliases }));
            }
          })
          .catch((err) => {
            logger.error("Trouble loading aliases");
            captureError(err);
          });
      }
    })();
    return () => {
      isStale = true;
    };
  }, [currentProcess]);
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
  }, []);

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
) => {
  useFigSubscriptionEffect(
    () => Keybindings.pressed(keypressCallback),
    [keypressCallback],
  );
};

export const useFigAutocomplete = (
  setFigState: React.Dispatch<React.SetStateAction<FigState>>,
) => {
  useFigSubscriptionEffect(
    () =>
      EditBufferNotifications.subscribe((notification) => {
        const buffer = notification.buffer ?? "";
        const cursorLocation = notification.cursor ?? buffer.length;

        const cwd = notification.context?.currentWorkingDirectory ?? null;
        const shellContext = notification.context;
        setFigState((figState) => ({
          ...figState,
          buffer,
          cursorLocation,
          cwd,
          shellContext,
        }));
        return { unsubscribe: false };
      }),
    [],
  );

  useFigSubscriptionEffect(
    () =>
      Shell.processDidChange.subscribe((notification) => {
        const { newProcess } = notification;
        setFigState((figState) => ({
          ...figState,
          processUserIsIn: newProcess?.executable ?? null,
          cwd: newProcess?.directory ?? null,
        }));
        return { unsubscribe: false };
      }),
    [],
  );

  useEffect(() => {
    initSentry();
  }, []);
};
