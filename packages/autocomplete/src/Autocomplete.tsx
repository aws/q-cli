import { SETTINGS } from "@aws/amazon-q-developer-cli-api-bindings-wrappers";
import React, {
  useState,
  useEffect,
  useCallback,
  useRef,
  useMemo,
  MutableRefObject,
} from "react";

import logger from "loglevel";
import { getCWDForFilesAndFolders } from "@aws/amazon-q-developer-cli-shared/utils";
import { getMaxHeight, getMaxWidth, POPOUT_WIDTH } from "./window";
import { useParseArgumentsEffect } from "./parser/hooks";
import "./index.css";
import AutoSizedList, { AutoSizedHandleRef } from "./components/AutoSizedList";

import {
  useFigKeypress,
  useFigAutocomplete,
  useFigSettings,
  useFigClearCache,
} from "./fig/hooks";

import { getCommonSuggestionPrefix } from "./suggestions/helpers";

import { AutocompleteStore, createAutocompleteStore } from "./state";
import {
  useAutocompleteKeypressCallback,
  Direction,
  setInterceptKeystrokesBackend,
} from "./hooks/keypress";
import {
  useShake,
  useDynamicResizeObserver,
  useSystemTheme,
} from "./hooks/helpers";

import Suggestion from "./components/Suggestion";
import Description, { DescriptionPosition } from "./components/Description";
import { setFontFamily, setFontSize, setTheme } from "./fig/themes";
import LoadingIcon from "./components/LoadingIcon";
import { CsWebsocket } from "@aws/amazon-q-developer-cli-ipc-client-websocket-mux";

import "./index.css";
import { InterceptRequestSchema } from "@aws/amazon-q-developer-cli-proto/figterm";
import { create } from "@bufbuild/protobuf";
import { AutocompleteContext } from "./state/context";
import { useAutocomplete } from "./state/useAutocomplete";
import {
  AutocompleteConnectionType,
  StyleType,
  Visibility,
} from "./state/types";
import DevModeWarning from "./components/DevModeWarning";
import AutocompleteWindow from "./components/AutocompleteWindow";

const getIconPath = (cwd: string): string => {
  const home = window?.fig?.constants?.home;
  const path = home && cwd.startsWith("~") ? home + cwd.slice(1) : cwd;
  return path.startsWith("//") ? path.slice(1) : path;
};

interface IpcClientProps {
  type: AutocompleteConnectionType | string;
  websocket: CsWebsocket;
}

type UnsubscribeFn = () => void;
type Listener<T> = (data: T) => UnsubscribeFn;
type MaybePromise<T> = T | Promise<T>;

export interface AutocompleteProps {
  ipcClient: IpcClientProps;
  style?: StyleType;
  enableMocks?: boolean;
  onDisconnect?: () => void;
  onKeypress?: (listener: Listener<string>) => void;

  /** The sessionId of the active tab */
  sessionId?: string;

  /** A callback to set the visibility of the autocomplete window */
  setVisibilityCallback?: (
    callback: (visible: boolean) => MaybePromise<void>,
  ) => void;

  /** A callback to poll the status of the multiplexer */
  setPollMultiplexerCallback?: (
    callback: (
      event: KeyboardEvent,
    ) => MaybePromise<{ multiplexerConnected: boolean }>,
  ) => void;
}

function AutocompleteInner({
  enableMocks,
  ipcClient: _ipcClientProps,
  setPollMultiplexerCallback,
  setVisibilityCallback,
  // onDisconnect,
}: AutocompleteProps) {
  const isWeb = useMemo(
    () => _ipcClientProps.type === AutocompleteConnectionType.CS_WEBSOCKET,
    [_ipcClientProps.type],
  );

  const {
    generatorStates,
    suggestions,
    selectedIndex,
    visibleState,
    setVisibleState,
    figState: { cwd, shellContext },
    parserResult: { searchTerm, currentArg },
    settings,
    setSettings,
    insertTextForItem,

    fuzzySearchEnabled,
    setUserFuzzySearchEnabled,
    setFigState,

    ipcClient,
    // setIpcClient,
  } = useAutocomplete();

  useMemo(() => {
    logger.enableAll();
  }, []);

  const suggestionsMock: Fig.Suggestion[] = [
    {
      name: "ec2",
      description: "The command line interface for Fig",
      insertValue: 'echo "123"',
    },
    {
      name: "account",
      description: "It's not what you think",
    },
    {
      name: "acm",
      description: "The largest and oldest of the group",
    },
    {
      name: "acm-pca",
      description: "With the circle",
    },
  ];

  const [systemTheme] = useSystemTheme();
  const [isShaking, shake] = useShake();
  const [isLoadingSuggestions, setIsLoadingSuggestions] = useState(false);
  const [windowState, setWindowState] = useState({
    isDescriptionSeparate: false,
    isAboveCursor: true,
    descriptionPosition: "unknown" as DescriptionPosition,
    previewPosition: "right" as DescriptionPosition,
  });
  const {
    [SETTINGS.THEME]: theme,
    [SETTINGS.FUZZY_SEARCH]: userFuzzySearch,
    [SETTINGS.FONT_FAMILY]: fontFamily,
    [SETTINGS.FONT_SIZE]: settingsFontSize,
    [SETTINGS.WIDTH]: settingsWidth,
    [SETTINGS.HEIGHT]: settingsHeight,
    [SETTINGS.ALWAYS_SHOW_DESCRIPTION]: alwaysShowDescription,
    [SETTINGS.DEV_MODE_NPM]: devMode,
  } = settings;

  const [size, setSize] = useState({
    fontSize: settingsFontSize as number | undefined,
    maxHeight: getMaxHeight(),
    suggestionWidth: getMaxWidth(),
    // 20 is the default height of a suggestion row. Font size is undefined unless set by user
    // in settings. If not set, row height should default to 20px.
    itemSize: 20,
  });

  useEffect(() => {
    setSize((oldSize) => ({
      ...oldSize,
      maxHeight: getMaxHeight(),
      suggestionWidth: getMaxWidth(),
    }));
  }, [settingsHeight, settingsWidth]);

  const isLoading = isLoadingSuggestions;

  useEffect(() => {
    // Default font-size is 12.8px (0.8em) and default row size is 20px = 12.8 * 1.5625
    // Row height should scale accordingly with font-size
    console.log("settingsFontSize", settingsFontSize);

    const fontSize =
      typeof settingsFontSize === "number" && settingsFontSize > 0
        ? settingsFontSize
        : 12.8;

    setSize((oldSize) => ({
      ...oldSize,
      fontSize,
      itemSize: fontSize * 1.5625,
    }));
  }, [settingsFontSize]);

  // Info passed down to suggestions to render icons and underline.
  const iconPath = useMemo(
    () => getIconPath(getCWDForFilesAndFolders(cwd, searchTerm)),
    [cwd, searchTerm],
  );

  const commonPrefix = useMemo(
    () =>
      getCommonSuggestionPrefix(
        selectedIndex,
        enableMocks ? suggestionsMock : suggestions,
      ),
    [enableMocks, selectedIndex, suggestions, suggestionsMock],
  );

  useEffect(() => {
    setWindowState((state) => ({
      isAboveCursor: alwaysShowDescription ? state.isAboveCursor : false,
      isDescriptionSeparate: alwaysShowDescription as boolean,
      descriptionPosition: "unknown",
      previewPosition: state.previewPosition,
    }));
  }, [alwaysShowDescription]);

  // Effect hooks into fig autocomplete events, parser, async generator results, and keypresses.
  const toggleDescriptionPopout = () => {
    setWindowState((state) => ({
      // if we are bringing the description back to the suggestion list, set isAboveCursor to false.
      isAboveCursor: state.isDescriptionSeparate ? false : state.isAboveCursor,
      isDescriptionSeparate: !state.isDescriptionSeparate,
      descriptionPosition: "unknown",
      previewPosition: state.previewPosition,
    }));
  };

  const changeSize = useCallback((direction: Direction): void => {
    // --font-size is read from the stylesheet as " 12.8px". We just want the number
    const currentFontSize = window
      .getComputedStyle(document.documentElement)
      .getPropertyValue("--font-size")
      .slice(0, -2)
      .trim();
    // Increase or decrease current size by 10%
    const change = (val: number) =>
      direction === Direction.INCREASE ? val * 1.1 : val / 1.1;

    setSize((oldSize) => ({
      fontSize: change(oldSize.fontSize ?? Number(currentFontSize)),
      itemSize: change(oldSize.itemSize),
      maxHeight: change(oldSize.maxHeight),
      suggestionWidth: change(oldSize.suggestionWidth),
    }));
  }, []);

  const keypressCallback = useAutocompleteKeypressCallback(
    toggleDescriptionPopout,
    shake,
    changeSize,
  );

  useEffect(() => {
    window.globalCWD = "";
    window.globalTerminalSessionId = "";
    window.globalSSHString = "";
  }, []);

  useFigAutocomplete(setFigState, ipcClient);
  useParseArgumentsEffect(setIsLoadingSuggestions, ipcClient);
  useFigSettings(setSettings);
  useFigKeypress(keypressCallback, ipcClient);
  useFigClearCache();

  useEffect(() => {
    if (setVisibilityCallback)
      setVisibilityCallback(async (visible) => {
        const v = visible
          ? Visibility.VISIBLE
          : Visibility.HIDDEN_UNTIL_KEYPRESS;
        setVisibleState(v);
      });
  }, [setVisibilityCallback, setVisibleState]);

  useEffect(() => {
    if (setPollMultiplexerCallback)
      setPollMultiplexerCallback(async (_event) => {
        const multiplexerConnected = ipcClient
          ? await ipcClient.isActive()
          : false;
        return {
          multiplexerConnected,
        };
      });
  }, [ipcClient, setPollMultiplexerCallback]);

  // useEffect(() => {
  //   let cancelled = false;
  //   const pingLoop = async () => {
  //     while (!cancelled) {
  //       if (ipcClient) {
  //         try {
  //           const timeoutPromise = new Promise((_, reject) => {
  //             setTimeout(() => reject(new Error("Ping timeout")), 10000);
  //           });
  //           await Promise.race([ipcClient.ping(), timeoutPromise]);
  //         } catch (_err) {
  //           if (!cancelled) {
  //             onDisconnect?.();
  //           }
  //         }
  //       }
  //       await new Promise((resolve) => setTimeout(resolve, 1000));
  //     }
  //   };
  //   pingLoop();
  //   return () => {
  //     cancelled = true;
  //   };
  // }, [ipcClient, onDisconnect]);

  // useEffect(() => {
  //   Settings.get(SETTINGS.DISABLE_HISTORY_LOADING)
  //     .catch(() => undefined)
  //     .then((res) => {
  //       if (!JSON.parse(res?.jsonBlob ?? "false")) {
  //         loadHistory({});
  //       }
  //     });
  // }, []);

  useEffect(() => {
    let isMostRecentEffect = true;
    if (generatorStates.some(({ loading }) => loading)) {
      setTimeout(() => {
        if (isMostRecentEffect) {
          setIsLoadingSuggestions(true);
        }
      }, 200);
    } else {
      setIsLoadingSuggestions(false);
    }
    return () => {
      isMostRecentEffect = false;
    };
  }, [generatorStates]);

  // Make sure fig dimensions align with our desired dimensions.
  const isHidden = visibleState !== Visibility.VISIBLE;
  const anySuggestions =
    (enableMocks ? suggestionsMock : suggestions).length > 0;
  const interceptKeystrokes = Boolean(
    !isHidden && (enableMocks ? suggestionsMock : suggestions).length > 0,
  );

  useEffect(() => {
    logger.info("Setting intercept keystrokes", {
      interceptKeystrokes,
    });
    setInterceptKeystrokesBackend(
      ipcClient,
      interceptKeystrokes,
      anySuggestions,
      shellContext?.sessionId,
    );
  }, [ipcClient, interceptKeystrokes, anySuggestions, shellContext?.sessionId]);
  useEffect(() => {
    setTheme(systemTheme, theme as string);
  }, [theme, systemTheme]);
  useEffect(() => {
    setFontSize(size.fontSize);
  }, [size.fontSize]);
  useEffect(() => {
    setUserFuzzySearchEnabled((userFuzzySearch ?? false) as boolean);
  }, [setUserFuzzySearchEnabled, userFuzzySearch]);
  useEffect(() => {
    setFontFamily(fontFamily as string);
  }, [fontFamily]);
  // Scroll when selectedIndex changes.
  const listRef =
    useRef<AutoSizedHandleRef>() as MutableRefObject<AutoSizedHandleRef>;

  const scrollToItemCallback = useCallback(() => {
    logger.info("Scrolling to", { selectedIndex });
    listRef.current?.scrollToItem(selectedIndex);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedIndex, windowState.descriptionPosition]);

  const hasSpecialArgDescription =
    (enableMocks ? suggestionsMock : suggestions).length === 0 &&
    Boolean(currentArg?.name || currentArg?.description);

  const onResize: (size: { height?: number; width?: number }) => void =
    useCallback(
      ({ height, width }) => {
        if (isWeb) return;
        const onLeft =
          !hasSpecialArgDescription &&
          windowState.descriptionPosition === "unknown";

        const frame = {
          height: height || 1,
          width: width || 1,
          anchorX: onLeft ? -POPOUT_WIDTH : 0,
          offsetFromBaseline: -3,
        };
        logger.debug("Setting window position", { frame });
      },
      // eslint-disable-next-line react-hooks/exhaustive-deps
      [
        windowState.descriptionPosition,
        hasSpecialArgDescription,
        isHidden,
        // eslint-disable-next-line react-hooks/exhaustive-deps
        (enableMocks ? suggestionsMock : suggestions)[selectedIndex]
          ?.previewComponent,
        isWeb,
      ],
    );

  useEffect(() => {
    onResize({});
  }, [onResize]);

  const { ref: autocompleteWindowRef } = useDynamicResizeObserver({ onResize });

  const hasSuggestions =
    (enableMocks ? suggestionsMock : suggestions).length > 0;

  useEffect(() => {
    if (shellContext?.sessionId) {
      ipcClient?.intercept(
        shellContext.sessionId,
        create(InterceptRequestSchema, {
          interceptCommand: {
            case: "setFigjsVisible",
            value: {
              visible: hasSuggestions,
            },
          },
        }),
      );
    }
  }, [hasSuggestions, ipcClient, shellContext?.sessionId]);

  if (isHidden) {
    return <div ref={autocompleteWindowRef} id="autocompleteWindow" />;
  }

  const descriptionPosition: DescriptionPosition =
    hasSuggestions && windowState.isDescriptionSeparate
      ? windowState.descriptionPosition
      : "unknown";

  const description = (
    <Description
      currentArg={currentArg}
      hasSuggestions={hasSuggestions}
      selectedItem={
        (enableMocks ? suggestionsMock : suggestions)[selectedIndex]
      }
      shouldShowHint={!isLoading && !alwaysShowDescription}
      position={descriptionPosition}
      height={size.itemSize}
    />
  );

  const hasBottomDescription =
    descriptionPosition === "unknown" && description !== null;
  const listClasses = [
    "rounded",
    isShaking && "shake",
    hasBottomDescription && "rounded-b-none",
  ];

  if (isLoading) {
    return (
      <AutocompleteWindow>
        <LoadingIcon />
      </AutocompleteWindow>
    );
  }

  return (
    <AutocompleteWindow>
      {windowState.isAboveCursor && (
        <DevModeWarning
          devMode={devMode}
          suggestionWidth={size.suggestionWidth}
        />
      )}
      <div className="q-autocomplete-container flex flex-row gap-1.5 p-1">
        {descriptionPosition === "left" && description}
        <div
          className={
            isWeb
              ? "q-autocomplete-container__sub-block"
              : "bg-main-bg relative flex flex-none flex-col items-stretch overflow-hidden rounded shadow-[0px_0px_3px_0px_rgb(85,_85,_85)]"
          }
          style={
            hasSuggestions
              ? {
                  width: size.suggestionWidth,
                  height: "100%",
                  alignSelf: windowState.isAboveCursor
                    ? "flex-end"
                    : "flex-start",
                  maxHeight: size.maxHeight,
                }
              : {}
          }
        >
          <AutoSizedList
            className={listClasses.filter((x) => !!x).join(" ")}
            onResize={scrollToItemCallback}
            ref={listRef}
            itemSize={size.itemSize}
            width="100%"
            itemCount={Math.round(
              (enableMocks ? suggestionsMock : suggestions).length,
            )}
          >
            {({ index, style }) => (
              <Suggestion
                style={style}
                suggestion={
                  (enableMocks ? suggestionsMock : suggestions)[index]
                }
                commonPrefix={commonPrefix || ""}
                onClick={(item) => insertTextForItem(item)}
                isActive={selectedIndex === index}
                searchTerm={searchTerm}
                fuzzySearchEnabled={fuzzySearchEnabled}
                iconPath={iconPath}
                iconSize={size.itemSize * 0.75}
                ipcClient={ipcClient}
                isWeb={isWeb}
              />
            )}
          </AutoSizedList>
          <div className="description-wrapper scrollbar-none flex flex-shrink-0 flex-row">
            {descriptionPosition === "unknown" && description}
            {isLoading && (
              <LoadingIcon
                className={
                  hasSuggestions
                    ? "left-[2px] [&>*]:top-[calc(50%-5px)]"
                    : "left-[2px] [&>*]:top-[calc(50%-3px)]"
                }
              />
            )}
          </div>
        </div>
        {descriptionPosition === "right" && description}
      </div>
      {!windowState.isAboveCursor && (
        <DevModeWarning
          devMode={devMode}
          suggestionWidth={size.suggestionWidth}
        />
      )}
    </AutocompleteWindow>
  );
}

function Autocomplete(props: AutocompleteProps) {
  const storeRef = useRef<AutocompleteStore>();
  if (!storeRef.current) {
    storeRef.current = createAutocompleteStore(props);
  }
  return (
    <AutocompleteContext.Provider value={storeRef.current}>
      <AutocompleteInner {...props} />
    </AutocompleteContext.Provider>
  );
}

export default Autocomplete;
