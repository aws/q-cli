/* eslint-disable */
import Long from "long";
import _m0 from "protobufjs/minimal";

export const protobufPackage = "local";

/** == Commands == */
export enum IntegrationAction {
  INSTALL = 0,
  VERIFY_INSTALL = 1,
  UNINSTALL = 2,
  UNRECOGNIZED = -1,
}

export function integrationActionFromJSON(object: any): IntegrationAction {
  switch (object) {
    case 0:
    case "INSTALL":
      return IntegrationAction.INSTALL;
    case 1:
    case "VERIFY_INSTALL":
      return IntegrationAction.VERIFY_INSTALL;
    case 2:
    case "UNINSTALL":
      return IntegrationAction.UNINSTALL;
    case -1:
    case "UNRECOGNIZED":
    default:
      return IntegrationAction.UNRECOGNIZED;
  }
}

export function integrationActionToJSON(object: IntegrationAction): string {
  switch (object) {
    case IntegrationAction.INSTALL:
      return "INSTALL";
    case IntegrationAction.VERIFY_INSTALL:
      return "VERIFY_INSTALL";
    case IntegrationAction.UNINSTALL:
      return "UNINSTALL";
    default:
      return "UNKNOWN";
  }
}

export enum UiElement {
  MENU_BAR = 0,
  SETTINGS = 1,
  UNRECOGNIZED = -1,
}

export function uiElementFromJSON(object: any): UiElement {
  switch (object) {
    case 0:
    case "MENU_BAR":
      return UiElement.MENU_BAR;
    case 1:
    case "SETTINGS":
      return UiElement.SETTINGS;
    case -1:
    case "UNRECOGNIZED":
    default:
      return UiElement.UNRECOGNIZED;
  }
}

export function uiElementToJSON(object: UiElement): string {
  switch (object) {
    case UiElement.MENU_BAR:
      return "MENU_BAR";
    case UiElement.SETTINGS:
      return "SETTINGS";
    default:
      return "UNKNOWN";
  }
}

export interface LocalMessage {
  type?: { $case: "command"; command: Command } | { $case: "hook"; hook: Hook };
}

export interface Command {
  id?: number | undefined;
  /** opt-out of response from host app */
  noResponse?: boolean | undefined;
  command?:
    | {
        $case: "terminalIntegration";
        terminalIntegration: TerminalIntegrationCommand;
      }
    | {
        $case: "listTerminalIntegrations";
        listTerminalIntegrations: ListTerminalIntegrationsCommand;
      }
    | { $case: "logout"; logout: LogoutCommand }
    | { $case: "restart"; restart: RestartCommand }
    | { $case: "quit"; quit: QuitCommand }
    | { $case: "update"; update: UpdateCommand }
    | { $case: "diagnostics"; diagnostics: DiagnosticsCommand }
    | { $case: "reportWindow"; reportWindow: ReportWindowCommand }
    | {
        $case: "restartSettingsListener";
        restartSettingsListener: RestartSettingsListenerCommand;
      }
    | { $case: "runInstallScript"; runInstallScript: RunInstallScriptCommand }
    | { $case: "build"; build: BuildCommand }
    | { $case: "openUiElement"; openUiElement: OpenUiElementCommand }
    | { $case: "resetCache"; resetCache: ResetCacheCommand }
    | { $case: "debugMode"; debugMode: DebugModeCommand }
    | {
        $case: "promptAccessibility";
        promptAccessibility: PromptAccessibilityCommand;
      };
}

export interface Hook {
  hook?:
    | { $case: "editBuffer"; editBuffer: EditBufferHook }
    | { $case: "init"; init: InitHook }
    | { $case: "prompt"; prompt: PromptHook }
    | { $case: "preExec"; preExec: PreExecHook }
    | { $case: "postExec"; postExec: PostExecHook }
    | {
        $case: "keyboardFocusChanged";
        keyboardFocusChanged: KeyboardFocusChangedHook;
      }
    | { $case: "tmuxPaneChanged"; tmuxPaneChanged: TmuxPaneChangedHook }
    | {
        $case: "openedSshConnection";
        openedSshConnection: OpenedSSHConnectionHook;
      }
    | { $case: "callback"; callback: CallbackHook }
    | { $case: "integrationReady"; integrationReady: IntegrationReadyHook }
    | { $case: "hide"; hide: HideHook }
    | { $case: "event"; event: EventHook };
}

export interface TerminalIntegrationCommand {
  identifier: string;
  action: IntegrationAction;
}

export interface ListTerminalIntegrationsCommand {}

export interface LogoutCommand {}

export interface RestartCommand {}

export interface QuitCommand {}

export interface UpdateCommand {
  force: boolean;
}

export interface DiagnosticsCommand {}

export interface ReportWindowCommand {
  report: string;
  path: string;
  figEnvVar: string;
  terminal: string;
}

export interface RestartSettingsListenerCommand {}

export interface RunInstallScriptCommand {}

export interface BuildCommand {
  branch?: string | undefined;
}

export interface OpenUiElementCommand {
  element: UiElement;
}

export interface ResetCacheCommand {}

export interface DebugModeCommand {
  /** Set debug mode to true or false */
  setDebugMode?: boolean | undefined;
  /** Toggle debug mode */
  toggleDebugMode?: boolean | undefined;
}

export interface PromptAccessibilityCommand {}

/** == Hooks == */
export interface ShellContext {
  pid?: number | undefined;
  /** /dev/ttys## of terminal session */
  ttys?: string | undefined;
  /** the name of the process */
  processName?: string | undefined;
  /** the directory where the user ran the command */
  currentWorkingDirectory?: string | undefined;
  /** the value of $TERM_SESSION_ID */
  sessionId?: string | undefined;
  integrationVersion?: number | undefined;
  terminal?: string | undefined;
  hostname?: string | undefined;
  remoteContext?: ShellContext | undefined;
}

export interface EditBufferHook {
  context: ShellContext | undefined;
  text: string;
  cursor: number;
  histno: number;
}

export interface InitHook {
  context: ShellContext | undefined;
  calledDirect: boolean;
  bundle: string;
  env: { [key: string]: string };
}

export interface InitHook_EnvEntry {
  key: string;
  value: string;
}

export interface PromptHook {
  context: ShellContext | undefined;
}

export interface PreExecHook {
  context: ShellContext | undefined;
  /** the full command that was run in the shell */
  command?: string | undefined;
}

export interface PostExecHook {
  context: ShellContext | undefined;
  /** the full command that was run in the shell */
  command: string;
  /** the exit code of the command */
  exitCode: number;
}

export interface KeyboardFocusChangedHook {
  appIdentifier: string;
  /** a unique identifier associated with the pane or tab that is currently focused */
  focusedSessionId: string;
}

export interface TmuxPaneChangedHook {
  paneIdentifier: number;
}

export interface OpenedSSHConnectionHook {
  context: ShellContext | undefined;
  controlPath: string;
}

export interface CallbackHook {
  handlerId: string;
  filepath: string;
  exitCode: string;
}

export interface IntegrationReadyHook {
  identifier: string;
}

export interface HideHook {}

export interface EventHook {
  eventName: string;
}

/** == Responses == */
export interface ErrorResponse {
  exitCode?: number | undefined;
  message?: string | undefined;
}

export interface SuccessResponse {
  message?: string | undefined;
}

export interface TerminalIntegration {
  bundleIdentifier: string;
  name: string;
  status?: string | undefined;
}

export interface TerminalIntegrationsListResponse {
  integrations: TerminalIntegration[];
}

export interface DiagnosticsResponse {
  distribution: string;
  beta: boolean;
  debugAutocomplete: boolean;
  developerModeEnabled: boolean;
  currentLayoutName: string;
  isRunningOnReadOnlyVolume: boolean;
  pathToBundle: string;
  accessibility: string;
  keypath: string;
  docker: string;
  symlinked: string;
  onlytab: string;
  installscript: string;
  psudoterminalPath: string;
  securekeyboard: string;
  securekeyboardPath: string;
  currentProcess: string;
  currentWindowIdentifier: string;
  autocomplete: boolean;
}

export interface CommandResponse {
  id?: number | undefined;
  response?:
    | { $case: "error"; error: ErrorResponse }
    | { $case: "success"; success: SuccessResponse }
    | {
        $case: "integrationList";
        integrationList: TerminalIntegrationsListResponse;
      }
    | { $case: "diagnostics"; diagnostics: DiagnosticsResponse };
}

const baseLocalMessage: object = {};

export const LocalMessage = {
  encode(
    message: LocalMessage,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.type?.$case === "command") {
      Command.encode(message.type.command, writer.uint32(18).fork()).ldelim();
    }
    if (message.type?.$case === "hook") {
      Hook.encode(message.type.hook, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): LocalMessage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseLocalMessage } as LocalMessage;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 2:
          message.type = {
            $case: "command",
            command: Command.decode(reader, reader.uint32()),
          };
          break;
        case 3:
          message.type = {
            $case: "hook",
            hook: Hook.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): LocalMessage {
    const message = { ...baseLocalMessage } as LocalMessage;
    if (object.command !== undefined && object.command !== null) {
      message.type = {
        $case: "command",
        command: Command.fromJSON(object.command),
      };
    }
    if (object.hook !== undefined && object.hook !== null) {
      message.type = { $case: "hook", hook: Hook.fromJSON(object.hook) };
    }
    return message;
  },

  toJSON(message: LocalMessage): unknown {
    const obj: any = {};
    message.type?.$case === "command" &&
      (obj.command = message.type?.command
        ? Command.toJSON(message.type?.command)
        : undefined);
    message.type?.$case === "hook" &&
      (obj.hook = message.type?.hook
        ? Hook.toJSON(message.type?.hook)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<LocalMessage>): LocalMessage {
    const message = { ...baseLocalMessage } as LocalMessage;
    if (
      object.type?.$case === "command" &&
      object.type?.command !== undefined &&
      object.type?.command !== null
    ) {
      message.type = {
        $case: "command",
        command: Command.fromPartial(object.type.command),
      };
    }
    if (
      object.type?.$case === "hook" &&
      object.type?.hook !== undefined &&
      object.type?.hook !== null
    ) {
      message.type = {
        $case: "hook",
        hook: Hook.fromPartial(object.type.hook),
      };
    }
    return message;
  },
};

const baseCommand: object = {};

export const Command = {
  encode(
    message: Command,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== undefined) {
      writer.uint32(8).int64(message.id);
    }
    if (message.noResponse !== undefined) {
      writer.uint32(16).bool(message.noResponse);
    }
    if (message.command?.$case === "terminalIntegration") {
      TerminalIntegrationCommand.encode(
        message.command.terminalIntegration,
        writer.uint32(802).fork()
      ).ldelim();
    }
    if (message.command?.$case === "listTerminalIntegrations") {
      ListTerminalIntegrationsCommand.encode(
        message.command.listTerminalIntegrations,
        writer.uint32(810).fork()
      ).ldelim();
    }
    if (message.command?.$case === "logout") {
      LogoutCommand.encode(
        message.command.logout,
        writer.uint32(818).fork()
      ).ldelim();
    }
    if (message.command?.$case === "restart") {
      RestartCommand.encode(
        message.command.restart,
        writer.uint32(826).fork()
      ).ldelim();
    }
    if (message.command?.$case === "quit") {
      QuitCommand.encode(
        message.command.quit,
        writer.uint32(834).fork()
      ).ldelim();
    }
    if (message.command?.$case === "update") {
      UpdateCommand.encode(
        message.command.update,
        writer.uint32(842).fork()
      ).ldelim();
    }
    if (message.command?.$case === "diagnostics") {
      DiagnosticsCommand.encode(
        message.command.diagnostics,
        writer.uint32(850).fork()
      ).ldelim();
    }
    if (message.command?.$case === "reportWindow") {
      ReportWindowCommand.encode(
        message.command.reportWindow,
        writer.uint32(858).fork()
      ).ldelim();
    }
    if (message.command?.$case === "restartSettingsListener") {
      RestartSettingsListenerCommand.encode(
        message.command.restartSettingsListener,
        writer.uint32(866).fork()
      ).ldelim();
    }
    if (message.command?.$case === "runInstallScript") {
      RunInstallScriptCommand.encode(
        message.command.runInstallScript,
        writer.uint32(874).fork()
      ).ldelim();
    }
    if (message.command?.$case === "build") {
      BuildCommand.encode(
        message.command.build,
        writer.uint32(882).fork()
      ).ldelim();
    }
    if (message.command?.$case === "openUiElement") {
      OpenUiElementCommand.encode(
        message.command.openUiElement,
        writer.uint32(890).fork()
      ).ldelim();
    }
    if (message.command?.$case === "resetCache") {
      ResetCacheCommand.encode(
        message.command.resetCache,
        writer.uint32(898).fork()
      ).ldelim();
    }
    if (message.command?.$case === "debugMode") {
      DebugModeCommand.encode(
        message.command.debugMode,
        writer.uint32(906).fork()
      ).ldelim();
    }
    if (message.command?.$case === "promptAccessibility") {
      PromptAccessibilityCommand.encode(
        message.command.promptAccessibility,
        writer.uint32(914).fork()
      ).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Command {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCommand } as Command;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = longToNumber(reader.int64() as Long);
          break;
        case 2:
          message.noResponse = reader.bool();
          break;
        case 100:
          message.command = {
            $case: "terminalIntegration",
            terminalIntegration: TerminalIntegrationCommand.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 101:
          message.command = {
            $case: "listTerminalIntegrations",
            listTerminalIntegrations: ListTerminalIntegrationsCommand.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 102:
          message.command = {
            $case: "logout",
            logout: LogoutCommand.decode(reader, reader.uint32()),
          };
          break;
        case 103:
          message.command = {
            $case: "restart",
            restart: RestartCommand.decode(reader, reader.uint32()),
          };
          break;
        case 104:
          message.command = {
            $case: "quit",
            quit: QuitCommand.decode(reader, reader.uint32()),
          };
          break;
        case 105:
          message.command = {
            $case: "update",
            update: UpdateCommand.decode(reader, reader.uint32()),
          };
          break;
        case 106:
          message.command = {
            $case: "diagnostics",
            diagnostics: DiagnosticsCommand.decode(reader, reader.uint32()),
          };
          break;
        case 107:
          message.command = {
            $case: "reportWindow",
            reportWindow: ReportWindowCommand.decode(reader, reader.uint32()),
          };
          break;
        case 108:
          message.command = {
            $case: "restartSettingsListener",
            restartSettingsListener: RestartSettingsListenerCommand.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 109:
          message.command = {
            $case: "runInstallScript",
            runInstallScript: RunInstallScriptCommand.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 110:
          message.command = {
            $case: "build",
            build: BuildCommand.decode(reader, reader.uint32()),
          };
          break;
        case 111:
          message.command = {
            $case: "openUiElement",
            openUiElement: OpenUiElementCommand.decode(reader, reader.uint32()),
          };
          break;
        case 112:
          message.command = {
            $case: "resetCache",
            resetCache: ResetCacheCommand.decode(reader, reader.uint32()),
          };
          break;
        case 113:
          message.command = {
            $case: "debugMode",
            debugMode: DebugModeCommand.decode(reader, reader.uint32()),
          };
          break;
        case 114:
          message.command = {
            $case: "promptAccessibility",
            promptAccessibility: PromptAccessibilityCommand.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Command {
    const message = { ...baseCommand } as Command;
    if (object.id !== undefined && object.id !== null) {
      message.id = Number(object.id);
    } else {
      message.id = undefined;
    }
    if (object.noResponse !== undefined && object.noResponse !== null) {
      message.noResponse = Boolean(object.noResponse);
    } else {
      message.noResponse = undefined;
    }
    if (
      object.terminalIntegration !== undefined &&
      object.terminalIntegration !== null
    ) {
      message.command = {
        $case: "terminalIntegration",
        terminalIntegration: TerminalIntegrationCommand.fromJSON(
          object.terminalIntegration
        ),
      };
    }
    if (
      object.listTerminalIntegrations !== undefined &&
      object.listTerminalIntegrations !== null
    ) {
      message.command = {
        $case: "listTerminalIntegrations",
        listTerminalIntegrations: ListTerminalIntegrationsCommand.fromJSON(
          object.listTerminalIntegrations
        ),
      };
    }
    if (object.logout !== undefined && object.logout !== null) {
      message.command = {
        $case: "logout",
        logout: LogoutCommand.fromJSON(object.logout),
      };
    }
    if (object.restart !== undefined && object.restart !== null) {
      message.command = {
        $case: "restart",
        restart: RestartCommand.fromJSON(object.restart),
      };
    }
    if (object.quit !== undefined && object.quit !== null) {
      message.command = {
        $case: "quit",
        quit: QuitCommand.fromJSON(object.quit),
      };
    }
    if (object.update !== undefined && object.update !== null) {
      message.command = {
        $case: "update",
        update: UpdateCommand.fromJSON(object.update),
      };
    }
    if (object.diagnostics !== undefined && object.diagnostics !== null) {
      message.command = {
        $case: "diagnostics",
        diagnostics: DiagnosticsCommand.fromJSON(object.diagnostics),
      };
    }
    if (object.reportWindow !== undefined && object.reportWindow !== null) {
      message.command = {
        $case: "reportWindow",
        reportWindow: ReportWindowCommand.fromJSON(object.reportWindow),
      };
    }
    if (
      object.restartSettingsListener !== undefined &&
      object.restartSettingsListener !== null
    ) {
      message.command = {
        $case: "restartSettingsListener",
        restartSettingsListener: RestartSettingsListenerCommand.fromJSON(
          object.restartSettingsListener
        ),
      };
    }
    if (
      object.runInstallScript !== undefined &&
      object.runInstallScript !== null
    ) {
      message.command = {
        $case: "runInstallScript",
        runInstallScript: RunInstallScriptCommand.fromJSON(
          object.runInstallScript
        ),
      };
    }
    if (object.build !== undefined && object.build !== null) {
      message.command = {
        $case: "build",
        build: BuildCommand.fromJSON(object.build),
      };
    }
    if (object.openUiElement !== undefined && object.openUiElement !== null) {
      message.command = {
        $case: "openUiElement",
        openUiElement: OpenUiElementCommand.fromJSON(object.openUiElement),
      };
    }
    if (object.resetCache !== undefined && object.resetCache !== null) {
      message.command = {
        $case: "resetCache",
        resetCache: ResetCacheCommand.fromJSON(object.resetCache),
      };
    }
    if (object.debugMode !== undefined && object.debugMode !== null) {
      message.command = {
        $case: "debugMode",
        debugMode: DebugModeCommand.fromJSON(object.debugMode),
      };
    }
    if (
      object.promptAccessibility !== undefined &&
      object.promptAccessibility !== null
    ) {
      message.command = {
        $case: "promptAccessibility",
        promptAccessibility: PromptAccessibilityCommand.fromJSON(
          object.promptAccessibility
        ),
      };
    }
    return message;
  },

  toJSON(message: Command): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.noResponse !== undefined && (obj.noResponse = message.noResponse);
    message.command?.$case === "terminalIntegration" &&
      (obj.terminalIntegration = message.command?.terminalIntegration
        ? TerminalIntegrationCommand.toJSON(
            message.command?.terminalIntegration
          )
        : undefined);
    message.command?.$case === "listTerminalIntegrations" &&
      (obj.listTerminalIntegrations = message.command?.listTerminalIntegrations
        ? ListTerminalIntegrationsCommand.toJSON(
            message.command?.listTerminalIntegrations
          )
        : undefined);
    message.command?.$case === "logout" &&
      (obj.logout = message.command?.logout
        ? LogoutCommand.toJSON(message.command?.logout)
        : undefined);
    message.command?.$case === "restart" &&
      (obj.restart = message.command?.restart
        ? RestartCommand.toJSON(message.command?.restart)
        : undefined);
    message.command?.$case === "quit" &&
      (obj.quit = message.command?.quit
        ? QuitCommand.toJSON(message.command?.quit)
        : undefined);
    message.command?.$case === "update" &&
      (obj.update = message.command?.update
        ? UpdateCommand.toJSON(message.command?.update)
        : undefined);
    message.command?.$case === "diagnostics" &&
      (obj.diagnostics = message.command?.diagnostics
        ? DiagnosticsCommand.toJSON(message.command?.diagnostics)
        : undefined);
    message.command?.$case === "reportWindow" &&
      (obj.reportWindow = message.command?.reportWindow
        ? ReportWindowCommand.toJSON(message.command?.reportWindow)
        : undefined);
    message.command?.$case === "restartSettingsListener" &&
      (obj.restartSettingsListener = message.command?.restartSettingsListener
        ? RestartSettingsListenerCommand.toJSON(
            message.command?.restartSettingsListener
          )
        : undefined);
    message.command?.$case === "runInstallScript" &&
      (obj.runInstallScript = message.command?.runInstallScript
        ? RunInstallScriptCommand.toJSON(message.command?.runInstallScript)
        : undefined);
    message.command?.$case === "build" &&
      (obj.build = message.command?.build
        ? BuildCommand.toJSON(message.command?.build)
        : undefined);
    message.command?.$case === "openUiElement" &&
      (obj.openUiElement = message.command?.openUiElement
        ? OpenUiElementCommand.toJSON(message.command?.openUiElement)
        : undefined);
    message.command?.$case === "resetCache" &&
      (obj.resetCache = message.command?.resetCache
        ? ResetCacheCommand.toJSON(message.command?.resetCache)
        : undefined);
    message.command?.$case === "debugMode" &&
      (obj.debugMode = message.command?.debugMode
        ? DebugModeCommand.toJSON(message.command?.debugMode)
        : undefined);
    message.command?.$case === "promptAccessibility" &&
      (obj.promptAccessibility = message.command?.promptAccessibility
        ? PromptAccessibilityCommand.toJSON(
            message.command?.promptAccessibility
          )
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<Command>): Command {
    const message = { ...baseCommand } as Command;
    message.id = object.id ?? undefined;
    message.noResponse = object.noResponse ?? undefined;
    if (
      object.command?.$case === "terminalIntegration" &&
      object.command?.terminalIntegration !== undefined &&
      object.command?.terminalIntegration !== null
    ) {
      message.command = {
        $case: "terminalIntegration",
        terminalIntegration: TerminalIntegrationCommand.fromPartial(
          object.command.terminalIntegration
        ),
      };
    }
    if (
      object.command?.$case === "listTerminalIntegrations" &&
      object.command?.listTerminalIntegrations !== undefined &&
      object.command?.listTerminalIntegrations !== null
    ) {
      message.command = {
        $case: "listTerminalIntegrations",
        listTerminalIntegrations: ListTerminalIntegrationsCommand.fromPartial(
          object.command.listTerminalIntegrations
        ),
      };
    }
    if (
      object.command?.$case === "logout" &&
      object.command?.logout !== undefined &&
      object.command?.logout !== null
    ) {
      message.command = {
        $case: "logout",
        logout: LogoutCommand.fromPartial(object.command.logout),
      };
    }
    if (
      object.command?.$case === "restart" &&
      object.command?.restart !== undefined &&
      object.command?.restart !== null
    ) {
      message.command = {
        $case: "restart",
        restart: RestartCommand.fromPartial(object.command.restart),
      };
    }
    if (
      object.command?.$case === "quit" &&
      object.command?.quit !== undefined &&
      object.command?.quit !== null
    ) {
      message.command = {
        $case: "quit",
        quit: QuitCommand.fromPartial(object.command.quit),
      };
    }
    if (
      object.command?.$case === "update" &&
      object.command?.update !== undefined &&
      object.command?.update !== null
    ) {
      message.command = {
        $case: "update",
        update: UpdateCommand.fromPartial(object.command.update),
      };
    }
    if (
      object.command?.$case === "diagnostics" &&
      object.command?.diagnostics !== undefined &&
      object.command?.diagnostics !== null
    ) {
      message.command = {
        $case: "diagnostics",
        diagnostics: DiagnosticsCommand.fromPartial(object.command.diagnostics),
      };
    }
    if (
      object.command?.$case === "reportWindow" &&
      object.command?.reportWindow !== undefined &&
      object.command?.reportWindow !== null
    ) {
      message.command = {
        $case: "reportWindow",
        reportWindow: ReportWindowCommand.fromPartial(
          object.command.reportWindow
        ),
      };
    }
    if (
      object.command?.$case === "restartSettingsListener" &&
      object.command?.restartSettingsListener !== undefined &&
      object.command?.restartSettingsListener !== null
    ) {
      message.command = {
        $case: "restartSettingsListener",
        restartSettingsListener: RestartSettingsListenerCommand.fromPartial(
          object.command.restartSettingsListener
        ),
      };
    }
    if (
      object.command?.$case === "runInstallScript" &&
      object.command?.runInstallScript !== undefined &&
      object.command?.runInstallScript !== null
    ) {
      message.command = {
        $case: "runInstallScript",
        runInstallScript: RunInstallScriptCommand.fromPartial(
          object.command.runInstallScript
        ),
      };
    }
    if (
      object.command?.$case === "build" &&
      object.command?.build !== undefined &&
      object.command?.build !== null
    ) {
      message.command = {
        $case: "build",
        build: BuildCommand.fromPartial(object.command.build),
      };
    }
    if (
      object.command?.$case === "openUiElement" &&
      object.command?.openUiElement !== undefined &&
      object.command?.openUiElement !== null
    ) {
      message.command = {
        $case: "openUiElement",
        openUiElement: OpenUiElementCommand.fromPartial(
          object.command.openUiElement
        ),
      };
    }
    if (
      object.command?.$case === "resetCache" &&
      object.command?.resetCache !== undefined &&
      object.command?.resetCache !== null
    ) {
      message.command = {
        $case: "resetCache",
        resetCache: ResetCacheCommand.fromPartial(object.command.resetCache),
      };
    }
    if (
      object.command?.$case === "debugMode" &&
      object.command?.debugMode !== undefined &&
      object.command?.debugMode !== null
    ) {
      message.command = {
        $case: "debugMode",
        debugMode: DebugModeCommand.fromPartial(object.command.debugMode),
      };
    }
    if (
      object.command?.$case === "promptAccessibility" &&
      object.command?.promptAccessibility !== undefined &&
      object.command?.promptAccessibility !== null
    ) {
      message.command = {
        $case: "promptAccessibility",
        promptAccessibility: PromptAccessibilityCommand.fromPartial(
          object.command.promptAccessibility
        ),
      };
    }
    return message;
  },
};

const baseHook: object = {};

export const Hook = {
  encode(message: Hook, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.hook?.$case === "editBuffer") {
      EditBufferHook.encode(
        message.hook.editBuffer,
        writer.uint32(802).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "init") {
      InitHook.encode(message.hook.init, writer.uint32(810).fork()).ldelim();
    }
    if (message.hook?.$case === "prompt") {
      PromptHook.encode(
        message.hook.prompt,
        writer.uint32(818).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "preExec") {
      PreExecHook.encode(
        message.hook.preExec,
        writer.uint32(826).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "postExec") {
      PostExecHook.encode(
        message.hook.postExec,
        writer.uint32(834).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "keyboardFocusChanged") {
      KeyboardFocusChangedHook.encode(
        message.hook.keyboardFocusChanged,
        writer.uint32(842).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "tmuxPaneChanged") {
      TmuxPaneChangedHook.encode(
        message.hook.tmuxPaneChanged,
        writer.uint32(850).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "openedSshConnection") {
      OpenedSSHConnectionHook.encode(
        message.hook.openedSshConnection,
        writer.uint32(858).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "callback") {
      CallbackHook.encode(
        message.hook.callback,
        writer.uint32(866).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "integrationReady") {
      IntegrationReadyHook.encode(
        message.hook.integrationReady,
        writer.uint32(874).fork()
      ).ldelim();
    }
    if (message.hook?.$case === "hide") {
      HideHook.encode(message.hook.hide, writer.uint32(882).fork()).ldelim();
    }
    if (message.hook?.$case === "event") {
      EventHook.encode(message.hook.event, writer.uint32(890).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Hook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseHook } as Hook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 100:
          message.hook = {
            $case: "editBuffer",
            editBuffer: EditBufferHook.decode(reader, reader.uint32()),
          };
          break;
        case 101:
          message.hook = {
            $case: "init",
            init: InitHook.decode(reader, reader.uint32()),
          };
          break;
        case 102:
          message.hook = {
            $case: "prompt",
            prompt: PromptHook.decode(reader, reader.uint32()),
          };
          break;
        case 103:
          message.hook = {
            $case: "preExec",
            preExec: PreExecHook.decode(reader, reader.uint32()),
          };
          break;
        case 104:
          message.hook = {
            $case: "postExec",
            postExec: PostExecHook.decode(reader, reader.uint32()),
          };
          break;
        case 105:
          message.hook = {
            $case: "keyboardFocusChanged",
            keyboardFocusChanged: KeyboardFocusChangedHook.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 106:
          message.hook = {
            $case: "tmuxPaneChanged",
            tmuxPaneChanged: TmuxPaneChangedHook.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 107:
          message.hook = {
            $case: "openedSshConnection",
            openedSshConnection: OpenedSSHConnectionHook.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 108:
          message.hook = {
            $case: "callback",
            callback: CallbackHook.decode(reader, reader.uint32()),
          };
          break;
        case 109:
          message.hook = {
            $case: "integrationReady",
            integrationReady: IntegrationReadyHook.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 110:
          message.hook = {
            $case: "hide",
            hide: HideHook.decode(reader, reader.uint32()),
          };
          break;
        case 111:
          message.hook = {
            $case: "event",
            event: EventHook.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Hook {
    const message = { ...baseHook } as Hook;
    if (object.editBuffer !== undefined && object.editBuffer !== null) {
      message.hook = {
        $case: "editBuffer",
        editBuffer: EditBufferHook.fromJSON(object.editBuffer),
      };
    }
    if (object.init !== undefined && object.init !== null) {
      message.hook = { $case: "init", init: InitHook.fromJSON(object.init) };
    }
    if (object.prompt !== undefined && object.prompt !== null) {
      message.hook = {
        $case: "prompt",
        prompt: PromptHook.fromJSON(object.prompt),
      };
    }
    if (object.preExec !== undefined && object.preExec !== null) {
      message.hook = {
        $case: "preExec",
        preExec: PreExecHook.fromJSON(object.preExec),
      };
    }
    if (object.postExec !== undefined && object.postExec !== null) {
      message.hook = {
        $case: "postExec",
        postExec: PostExecHook.fromJSON(object.postExec),
      };
    }
    if (
      object.keyboardFocusChanged !== undefined &&
      object.keyboardFocusChanged !== null
    ) {
      message.hook = {
        $case: "keyboardFocusChanged",
        keyboardFocusChanged: KeyboardFocusChangedHook.fromJSON(
          object.keyboardFocusChanged
        ),
      };
    }
    if (
      object.tmuxPaneChanged !== undefined &&
      object.tmuxPaneChanged !== null
    ) {
      message.hook = {
        $case: "tmuxPaneChanged",
        tmuxPaneChanged: TmuxPaneChangedHook.fromJSON(object.tmuxPaneChanged),
      };
    }
    if (
      object.openedSshConnection !== undefined &&
      object.openedSshConnection !== null
    ) {
      message.hook = {
        $case: "openedSshConnection",
        openedSshConnection: OpenedSSHConnectionHook.fromJSON(
          object.openedSshConnection
        ),
      };
    }
    if (object.callback !== undefined && object.callback !== null) {
      message.hook = {
        $case: "callback",
        callback: CallbackHook.fromJSON(object.callback),
      };
    }
    if (
      object.integrationReady !== undefined &&
      object.integrationReady !== null
    ) {
      message.hook = {
        $case: "integrationReady",
        integrationReady: IntegrationReadyHook.fromJSON(
          object.integrationReady
        ),
      };
    }
    if (object.hide !== undefined && object.hide !== null) {
      message.hook = { $case: "hide", hide: HideHook.fromJSON(object.hide) };
    }
    if (object.event !== undefined && object.event !== null) {
      message.hook = {
        $case: "event",
        event: EventHook.fromJSON(object.event),
      };
    }
    return message;
  },

  toJSON(message: Hook): unknown {
    const obj: any = {};
    message.hook?.$case === "editBuffer" &&
      (obj.editBuffer = message.hook?.editBuffer
        ? EditBufferHook.toJSON(message.hook?.editBuffer)
        : undefined);
    message.hook?.$case === "init" &&
      (obj.init = message.hook?.init
        ? InitHook.toJSON(message.hook?.init)
        : undefined);
    message.hook?.$case === "prompt" &&
      (obj.prompt = message.hook?.prompt
        ? PromptHook.toJSON(message.hook?.prompt)
        : undefined);
    message.hook?.$case === "preExec" &&
      (obj.preExec = message.hook?.preExec
        ? PreExecHook.toJSON(message.hook?.preExec)
        : undefined);
    message.hook?.$case === "postExec" &&
      (obj.postExec = message.hook?.postExec
        ? PostExecHook.toJSON(message.hook?.postExec)
        : undefined);
    message.hook?.$case === "keyboardFocusChanged" &&
      (obj.keyboardFocusChanged = message.hook?.keyboardFocusChanged
        ? KeyboardFocusChangedHook.toJSON(message.hook?.keyboardFocusChanged)
        : undefined);
    message.hook?.$case === "tmuxPaneChanged" &&
      (obj.tmuxPaneChanged = message.hook?.tmuxPaneChanged
        ? TmuxPaneChangedHook.toJSON(message.hook?.tmuxPaneChanged)
        : undefined);
    message.hook?.$case === "openedSshConnection" &&
      (obj.openedSshConnection = message.hook?.openedSshConnection
        ? OpenedSSHConnectionHook.toJSON(message.hook?.openedSshConnection)
        : undefined);
    message.hook?.$case === "callback" &&
      (obj.callback = message.hook?.callback
        ? CallbackHook.toJSON(message.hook?.callback)
        : undefined);
    message.hook?.$case === "integrationReady" &&
      (obj.integrationReady = message.hook?.integrationReady
        ? IntegrationReadyHook.toJSON(message.hook?.integrationReady)
        : undefined);
    message.hook?.$case === "hide" &&
      (obj.hide = message.hook?.hide
        ? HideHook.toJSON(message.hook?.hide)
        : undefined);
    message.hook?.$case === "event" &&
      (obj.event = message.hook?.event
        ? EventHook.toJSON(message.hook?.event)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<Hook>): Hook {
    const message = { ...baseHook } as Hook;
    if (
      object.hook?.$case === "editBuffer" &&
      object.hook?.editBuffer !== undefined &&
      object.hook?.editBuffer !== null
    ) {
      message.hook = {
        $case: "editBuffer",
        editBuffer: EditBufferHook.fromPartial(object.hook.editBuffer),
      };
    }
    if (
      object.hook?.$case === "init" &&
      object.hook?.init !== undefined &&
      object.hook?.init !== null
    ) {
      message.hook = {
        $case: "init",
        init: InitHook.fromPartial(object.hook.init),
      };
    }
    if (
      object.hook?.$case === "prompt" &&
      object.hook?.prompt !== undefined &&
      object.hook?.prompt !== null
    ) {
      message.hook = {
        $case: "prompt",
        prompt: PromptHook.fromPartial(object.hook.prompt),
      };
    }
    if (
      object.hook?.$case === "preExec" &&
      object.hook?.preExec !== undefined &&
      object.hook?.preExec !== null
    ) {
      message.hook = {
        $case: "preExec",
        preExec: PreExecHook.fromPartial(object.hook.preExec),
      };
    }
    if (
      object.hook?.$case === "postExec" &&
      object.hook?.postExec !== undefined &&
      object.hook?.postExec !== null
    ) {
      message.hook = {
        $case: "postExec",
        postExec: PostExecHook.fromPartial(object.hook.postExec),
      };
    }
    if (
      object.hook?.$case === "keyboardFocusChanged" &&
      object.hook?.keyboardFocusChanged !== undefined &&
      object.hook?.keyboardFocusChanged !== null
    ) {
      message.hook = {
        $case: "keyboardFocusChanged",
        keyboardFocusChanged: KeyboardFocusChangedHook.fromPartial(
          object.hook.keyboardFocusChanged
        ),
      };
    }
    if (
      object.hook?.$case === "tmuxPaneChanged" &&
      object.hook?.tmuxPaneChanged !== undefined &&
      object.hook?.tmuxPaneChanged !== null
    ) {
      message.hook = {
        $case: "tmuxPaneChanged",
        tmuxPaneChanged: TmuxPaneChangedHook.fromPartial(
          object.hook.tmuxPaneChanged
        ),
      };
    }
    if (
      object.hook?.$case === "openedSshConnection" &&
      object.hook?.openedSshConnection !== undefined &&
      object.hook?.openedSshConnection !== null
    ) {
      message.hook = {
        $case: "openedSshConnection",
        openedSshConnection: OpenedSSHConnectionHook.fromPartial(
          object.hook.openedSshConnection
        ),
      };
    }
    if (
      object.hook?.$case === "callback" &&
      object.hook?.callback !== undefined &&
      object.hook?.callback !== null
    ) {
      message.hook = {
        $case: "callback",
        callback: CallbackHook.fromPartial(object.hook.callback),
      };
    }
    if (
      object.hook?.$case === "integrationReady" &&
      object.hook?.integrationReady !== undefined &&
      object.hook?.integrationReady !== null
    ) {
      message.hook = {
        $case: "integrationReady",
        integrationReady: IntegrationReadyHook.fromPartial(
          object.hook.integrationReady
        ),
      };
    }
    if (
      object.hook?.$case === "hide" &&
      object.hook?.hide !== undefined &&
      object.hook?.hide !== null
    ) {
      message.hook = {
        $case: "hide",
        hide: HideHook.fromPartial(object.hook.hide),
      };
    }
    if (
      object.hook?.$case === "event" &&
      object.hook?.event !== undefined &&
      object.hook?.event !== null
    ) {
      message.hook = {
        $case: "event",
        event: EventHook.fromPartial(object.hook.event),
      };
    }
    return message;
  },
};

const baseTerminalIntegrationCommand: object = { identifier: "", action: 0 };

export const TerminalIntegrationCommand = {
  encode(
    message: TerminalIntegrationCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.identifier !== "") {
      writer.uint32(10).string(message.identifier);
    }
    if (message.action !== 0) {
      writer.uint32(16).int32(message.action);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): TerminalIntegrationCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseTerminalIntegrationCommand,
    } as TerminalIntegrationCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.identifier = reader.string();
          break;
        case 2:
          message.action = reader.int32() as any;
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TerminalIntegrationCommand {
    const message = {
      ...baseTerminalIntegrationCommand,
    } as TerminalIntegrationCommand;
    if (object.identifier !== undefined && object.identifier !== null) {
      message.identifier = String(object.identifier);
    } else {
      message.identifier = "";
    }
    if (object.action !== undefined && object.action !== null) {
      message.action = integrationActionFromJSON(object.action);
    } else {
      message.action = 0;
    }
    return message;
  },

  toJSON(message: TerminalIntegrationCommand): unknown {
    const obj: any = {};
    message.identifier !== undefined && (obj.identifier = message.identifier);
    message.action !== undefined &&
      (obj.action = integrationActionToJSON(message.action));
    return obj;
  },

  fromPartial(
    object: DeepPartial<TerminalIntegrationCommand>
  ): TerminalIntegrationCommand {
    const message = {
      ...baseTerminalIntegrationCommand,
    } as TerminalIntegrationCommand;
    message.identifier = object.identifier ?? "";
    message.action = object.action ?? 0;
    return message;
  },
};

const baseListTerminalIntegrationsCommand: object = {};

export const ListTerminalIntegrationsCommand = {
  encode(
    _: ListTerminalIntegrationsCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ListTerminalIntegrationsCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseListTerminalIntegrationsCommand,
    } as ListTerminalIntegrationsCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): ListTerminalIntegrationsCommand {
    const message = {
      ...baseListTerminalIntegrationsCommand,
    } as ListTerminalIntegrationsCommand;
    return message;
  },

  toJSON(_: ListTerminalIntegrationsCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(
    _: DeepPartial<ListTerminalIntegrationsCommand>
  ): ListTerminalIntegrationsCommand {
    const message = {
      ...baseListTerminalIntegrationsCommand,
    } as ListTerminalIntegrationsCommand;
    return message;
  },
};

const baseLogoutCommand: object = {};

export const LogoutCommand = {
  encode(
    _: LogoutCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): LogoutCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseLogoutCommand } as LogoutCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): LogoutCommand {
    const message = { ...baseLogoutCommand } as LogoutCommand;
    return message;
  },

  toJSON(_: LogoutCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(_: DeepPartial<LogoutCommand>): LogoutCommand {
    const message = { ...baseLogoutCommand } as LogoutCommand;
    return message;
  },
};

const baseRestartCommand: object = {};

export const RestartCommand = {
  encode(
    _: RestartCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): RestartCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseRestartCommand } as RestartCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): RestartCommand {
    const message = { ...baseRestartCommand } as RestartCommand;
    return message;
  },

  toJSON(_: RestartCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(_: DeepPartial<RestartCommand>): RestartCommand {
    const message = { ...baseRestartCommand } as RestartCommand;
    return message;
  },
};

const baseQuitCommand: object = {};

export const QuitCommand = {
  encode(_: QuitCommand, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): QuitCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseQuitCommand } as QuitCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): QuitCommand {
    const message = { ...baseQuitCommand } as QuitCommand;
    return message;
  },

  toJSON(_: QuitCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(_: DeepPartial<QuitCommand>): QuitCommand {
    const message = { ...baseQuitCommand } as QuitCommand;
    return message;
  },
};

const baseUpdateCommand: object = { force: false };

export const UpdateCommand = {
  encode(
    message: UpdateCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.force === true) {
      writer.uint32(8).bool(message.force);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UpdateCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseUpdateCommand } as UpdateCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.force = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): UpdateCommand {
    const message = { ...baseUpdateCommand } as UpdateCommand;
    if (object.force !== undefined && object.force !== null) {
      message.force = Boolean(object.force);
    } else {
      message.force = false;
    }
    return message;
  },

  toJSON(message: UpdateCommand): unknown {
    const obj: any = {};
    message.force !== undefined && (obj.force = message.force);
    return obj;
  },

  fromPartial(object: DeepPartial<UpdateCommand>): UpdateCommand {
    const message = { ...baseUpdateCommand } as UpdateCommand;
    message.force = object.force ?? false;
    return message;
  },
};

const baseDiagnosticsCommand: object = {};

export const DiagnosticsCommand = {
  encode(
    _: DiagnosticsCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): DiagnosticsCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseDiagnosticsCommand } as DiagnosticsCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): DiagnosticsCommand {
    const message = { ...baseDiagnosticsCommand } as DiagnosticsCommand;
    return message;
  },

  toJSON(_: DiagnosticsCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(_: DeepPartial<DiagnosticsCommand>): DiagnosticsCommand {
    const message = { ...baseDiagnosticsCommand } as DiagnosticsCommand;
    return message;
  },
};

const baseReportWindowCommand: object = {
  report: "",
  path: "",
  figEnvVar: "",
  terminal: "",
};

export const ReportWindowCommand = {
  encode(
    message: ReportWindowCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.report !== "") {
      writer.uint32(10).string(message.report);
    }
    if (message.path !== "") {
      writer.uint32(18).string(message.path);
    }
    if (message.figEnvVar !== "") {
      writer.uint32(26).string(message.figEnvVar);
    }
    if (message.terminal !== "") {
      writer.uint32(34).string(message.terminal);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ReportWindowCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReportWindowCommand } as ReportWindowCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.report = reader.string();
          break;
        case 2:
          message.path = reader.string();
          break;
        case 3:
          message.figEnvVar = reader.string();
          break;
        case 4:
          message.terminal = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReportWindowCommand {
    const message = { ...baseReportWindowCommand } as ReportWindowCommand;
    if (object.report !== undefined && object.report !== null) {
      message.report = String(object.report);
    } else {
      message.report = "";
    }
    if (object.path !== undefined && object.path !== null) {
      message.path = String(object.path);
    } else {
      message.path = "";
    }
    if (object.figEnvVar !== undefined && object.figEnvVar !== null) {
      message.figEnvVar = String(object.figEnvVar);
    } else {
      message.figEnvVar = "";
    }
    if (object.terminal !== undefined && object.terminal !== null) {
      message.terminal = String(object.terminal);
    } else {
      message.terminal = "";
    }
    return message;
  },

  toJSON(message: ReportWindowCommand): unknown {
    const obj: any = {};
    message.report !== undefined && (obj.report = message.report);
    message.path !== undefined && (obj.path = message.path);
    message.figEnvVar !== undefined && (obj.figEnvVar = message.figEnvVar);
    message.terminal !== undefined && (obj.terminal = message.terminal);
    return obj;
  },

  fromPartial(object: DeepPartial<ReportWindowCommand>): ReportWindowCommand {
    const message = { ...baseReportWindowCommand } as ReportWindowCommand;
    message.report = object.report ?? "";
    message.path = object.path ?? "";
    message.figEnvVar = object.figEnvVar ?? "";
    message.terminal = object.terminal ?? "";
    return message;
  },
};

const baseRestartSettingsListenerCommand: object = {};

export const RestartSettingsListenerCommand = {
  encode(
    _: RestartSettingsListenerCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): RestartSettingsListenerCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseRestartSettingsListenerCommand,
    } as RestartSettingsListenerCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): RestartSettingsListenerCommand {
    const message = {
      ...baseRestartSettingsListenerCommand,
    } as RestartSettingsListenerCommand;
    return message;
  },

  toJSON(_: RestartSettingsListenerCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(
    _: DeepPartial<RestartSettingsListenerCommand>
  ): RestartSettingsListenerCommand {
    const message = {
      ...baseRestartSettingsListenerCommand,
    } as RestartSettingsListenerCommand;
    return message;
  },
};

const baseRunInstallScriptCommand: object = {};

export const RunInstallScriptCommand = {
  encode(
    _: RunInstallScriptCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): RunInstallScriptCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseRunInstallScriptCommand,
    } as RunInstallScriptCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): RunInstallScriptCommand {
    const message = {
      ...baseRunInstallScriptCommand,
    } as RunInstallScriptCommand;
    return message;
  },

  toJSON(_: RunInstallScriptCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(
    _: DeepPartial<RunInstallScriptCommand>
  ): RunInstallScriptCommand {
    const message = {
      ...baseRunInstallScriptCommand,
    } as RunInstallScriptCommand;
    return message;
  },
};

const baseBuildCommand: object = {};

export const BuildCommand = {
  encode(
    message: BuildCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.branch !== undefined) {
      writer.uint32(10).string(message.branch);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BuildCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseBuildCommand } as BuildCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.branch = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): BuildCommand {
    const message = { ...baseBuildCommand } as BuildCommand;
    if (object.branch !== undefined && object.branch !== null) {
      message.branch = String(object.branch);
    } else {
      message.branch = undefined;
    }
    return message;
  },

  toJSON(message: BuildCommand): unknown {
    const obj: any = {};
    message.branch !== undefined && (obj.branch = message.branch);
    return obj;
  },

  fromPartial(object: DeepPartial<BuildCommand>): BuildCommand {
    const message = { ...baseBuildCommand } as BuildCommand;
    message.branch = object.branch ?? undefined;
    return message;
  },
};

const baseOpenUiElementCommand: object = { element: 0 };

export const OpenUiElementCommand = {
  encode(
    message: OpenUiElementCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.element !== 0) {
      writer.uint32(8).int32(message.element);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): OpenUiElementCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseOpenUiElementCommand } as OpenUiElementCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.element = reader.int32() as any;
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): OpenUiElementCommand {
    const message = { ...baseOpenUiElementCommand } as OpenUiElementCommand;
    if (object.element !== undefined && object.element !== null) {
      message.element = uiElementFromJSON(object.element);
    } else {
      message.element = 0;
    }
    return message;
  },

  toJSON(message: OpenUiElementCommand): unknown {
    const obj: any = {};
    message.element !== undefined &&
      (obj.element = uiElementToJSON(message.element));
    return obj;
  },

  fromPartial(object: DeepPartial<OpenUiElementCommand>): OpenUiElementCommand {
    const message = { ...baseOpenUiElementCommand } as OpenUiElementCommand;
    message.element = object.element ?? 0;
    return message;
  },
};

const baseResetCacheCommand: object = {};

export const ResetCacheCommand = {
  encode(
    _: ResetCacheCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ResetCacheCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseResetCacheCommand } as ResetCacheCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): ResetCacheCommand {
    const message = { ...baseResetCacheCommand } as ResetCacheCommand;
    return message;
  },

  toJSON(_: ResetCacheCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(_: DeepPartial<ResetCacheCommand>): ResetCacheCommand {
    const message = { ...baseResetCacheCommand } as ResetCacheCommand;
    return message;
  },
};

const baseDebugModeCommand: object = {};

export const DebugModeCommand = {
  encode(
    message: DebugModeCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.setDebugMode !== undefined) {
      writer.uint32(8).bool(message.setDebugMode);
    }
    if (message.toggleDebugMode !== undefined) {
      writer.uint32(16).bool(message.toggleDebugMode);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): DebugModeCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseDebugModeCommand } as DebugModeCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.setDebugMode = reader.bool();
          break;
        case 2:
          message.toggleDebugMode = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): DebugModeCommand {
    const message = { ...baseDebugModeCommand } as DebugModeCommand;
    if (object.setDebugMode !== undefined && object.setDebugMode !== null) {
      message.setDebugMode = Boolean(object.setDebugMode);
    } else {
      message.setDebugMode = undefined;
    }
    if (
      object.toggleDebugMode !== undefined &&
      object.toggleDebugMode !== null
    ) {
      message.toggleDebugMode = Boolean(object.toggleDebugMode);
    } else {
      message.toggleDebugMode = undefined;
    }
    return message;
  },

  toJSON(message: DebugModeCommand): unknown {
    const obj: any = {};
    message.setDebugMode !== undefined &&
      (obj.setDebugMode = message.setDebugMode);
    message.toggleDebugMode !== undefined &&
      (obj.toggleDebugMode = message.toggleDebugMode);
    return obj;
  },

  fromPartial(object: DeepPartial<DebugModeCommand>): DebugModeCommand {
    const message = { ...baseDebugModeCommand } as DebugModeCommand;
    message.setDebugMode = object.setDebugMode ?? undefined;
    message.toggleDebugMode = object.toggleDebugMode ?? undefined;
    return message;
  },
};

const basePromptAccessibilityCommand: object = {};

export const PromptAccessibilityCommand = {
  encode(
    _: PromptAccessibilityCommand,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): PromptAccessibilityCommand {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...basePromptAccessibilityCommand,
    } as PromptAccessibilityCommand;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): PromptAccessibilityCommand {
    const message = {
      ...basePromptAccessibilityCommand,
    } as PromptAccessibilityCommand;
    return message;
  },

  toJSON(_: PromptAccessibilityCommand): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(
    _: DeepPartial<PromptAccessibilityCommand>
  ): PromptAccessibilityCommand {
    const message = {
      ...basePromptAccessibilityCommand,
    } as PromptAccessibilityCommand;
    return message;
  },
};

const baseShellContext: object = {};

export const ShellContext = {
  encode(
    message: ShellContext,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.pid !== undefined) {
      writer.uint32(8).int32(message.pid);
    }
    if (message.ttys !== undefined) {
      writer.uint32(18).string(message.ttys);
    }
    if (message.processName !== undefined) {
      writer.uint32(26).string(message.processName);
    }
    if (message.currentWorkingDirectory !== undefined) {
      writer.uint32(34).string(message.currentWorkingDirectory);
    }
    if (message.sessionId !== undefined) {
      writer.uint32(42).string(message.sessionId);
    }
    if (message.integrationVersion !== undefined) {
      writer.uint32(48).int32(message.integrationVersion);
    }
    if (message.terminal !== undefined) {
      writer.uint32(58).string(message.terminal);
    }
    if (message.hostname !== undefined) {
      writer.uint32(66).string(message.hostname);
    }
    if (message.remoteContext !== undefined) {
      ShellContext.encode(
        message.remoteContext,
        writer.uint32(74).fork()
      ).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ShellContext {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseShellContext } as ShellContext;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.pid = reader.int32();
          break;
        case 2:
          message.ttys = reader.string();
          break;
        case 3:
          message.processName = reader.string();
          break;
        case 4:
          message.currentWorkingDirectory = reader.string();
          break;
        case 5:
          message.sessionId = reader.string();
          break;
        case 6:
          message.integrationVersion = reader.int32();
          break;
        case 7:
          message.terminal = reader.string();
          break;
        case 8:
          message.hostname = reader.string();
          break;
        case 9:
          message.remoteContext = ShellContext.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ShellContext {
    const message = { ...baseShellContext } as ShellContext;
    if (object.pid !== undefined && object.pid !== null) {
      message.pid = Number(object.pid);
    } else {
      message.pid = undefined;
    }
    if (object.ttys !== undefined && object.ttys !== null) {
      message.ttys = String(object.ttys);
    } else {
      message.ttys = undefined;
    }
    if (object.processName !== undefined && object.processName !== null) {
      message.processName = String(object.processName);
    } else {
      message.processName = undefined;
    }
    if (
      object.currentWorkingDirectory !== undefined &&
      object.currentWorkingDirectory !== null
    ) {
      message.currentWorkingDirectory = String(object.currentWorkingDirectory);
    } else {
      message.currentWorkingDirectory = undefined;
    }
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = String(object.sessionId);
    } else {
      message.sessionId = undefined;
    }
    if (
      object.integrationVersion !== undefined &&
      object.integrationVersion !== null
    ) {
      message.integrationVersion = Number(object.integrationVersion);
    } else {
      message.integrationVersion = undefined;
    }
    if (object.terminal !== undefined && object.terminal !== null) {
      message.terminal = String(object.terminal);
    } else {
      message.terminal = undefined;
    }
    if (object.hostname !== undefined && object.hostname !== null) {
      message.hostname = String(object.hostname);
    } else {
      message.hostname = undefined;
    }
    if (object.remoteContext !== undefined && object.remoteContext !== null) {
      message.remoteContext = ShellContext.fromJSON(object.remoteContext);
    } else {
      message.remoteContext = undefined;
    }
    return message;
  },

  toJSON(message: ShellContext): unknown {
    const obj: any = {};
    message.pid !== undefined && (obj.pid = message.pid);
    message.ttys !== undefined && (obj.ttys = message.ttys);
    message.processName !== undefined &&
      (obj.processName = message.processName);
    message.currentWorkingDirectory !== undefined &&
      (obj.currentWorkingDirectory = message.currentWorkingDirectory);
    message.sessionId !== undefined && (obj.sessionId = message.sessionId);
    message.integrationVersion !== undefined &&
      (obj.integrationVersion = message.integrationVersion);
    message.terminal !== undefined && (obj.terminal = message.terminal);
    message.hostname !== undefined && (obj.hostname = message.hostname);
    message.remoteContext !== undefined &&
      (obj.remoteContext = message.remoteContext
        ? ShellContext.toJSON(message.remoteContext)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<ShellContext>): ShellContext {
    const message = { ...baseShellContext } as ShellContext;
    message.pid = object.pid ?? undefined;
    message.ttys = object.ttys ?? undefined;
    message.processName = object.processName ?? undefined;
    message.currentWorkingDirectory =
      object.currentWorkingDirectory ?? undefined;
    message.sessionId = object.sessionId ?? undefined;
    message.integrationVersion = object.integrationVersion ?? undefined;
    message.terminal = object.terminal ?? undefined;
    message.hostname = object.hostname ?? undefined;
    if (object.remoteContext !== undefined && object.remoteContext !== null) {
      message.remoteContext = ShellContext.fromPartial(object.remoteContext);
    } else {
      message.remoteContext = undefined;
    }
    return message;
  },
};

const baseEditBufferHook: object = { text: "", cursor: 0, histno: 0 };

export const EditBufferHook = {
  encode(
    message: EditBufferHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.context !== undefined) {
      ShellContext.encode(message.context, writer.uint32(10).fork()).ldelim();
    }
    if (message.text !== "") {
      writer.uint32(18).string(message.text);
    }
    if (message.cursor !== 0) {
      writer.uint32(24).int64(message.cursor);
    }
    if (message.histno !== 0) {
      writer.uint32(32).int64(message.histno);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): EditBufferHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseEditBufferHook } as EditBufferHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.context = ShellContext.decode(reader, reader.uint32());
          break;
        case 2:
          message.text = reader.string();
          break;
        case 3:
          message.cursor = longToNumber(reader.int64() as Long);
          break;
        case 4:
          message.histno = longToNumber(reader.int64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): EditBufferHook {
    const message = { ...baseEditBufferHook } as EditBufferHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromJSON(object.context);
    } else {
      message.context = undefined;
    }
    if (object.text !== undefined && object.text !== null) {
      message.text = String(object.text);
    } else {
      message.text = "";
    }
    if (object.cursor !== undefined && object.cursor !== null) {
      message.cursor = Number(object.cursor);
    } else {
      message.cursor = 0;
    }
    if (object.histno !== undefined && object.histno !== null) {
      message.histno = Number(object.histno);
    } else {
      message.histno = 0;
    }
    return message;
  },

  toJSON(message: EditBufferHook): unknown {
    const obj: any = {};
    message.context !== undefined &&
      (obj.context = message.context
        ? ShellContext.toJSON(message.context)
        : undefined);
    message.text !== undefined && (obj.text = message.text);
    message.cursor !== undefined && (obj.cursor = message.cursor);
    message.histno !== undefined && (obj.histno = message.histno);
    return obj;
  },

  fromPartial(object: DeepPartial<EditBufferHook>): EditBufferHook {
    const message = { ...baseEditBufferHook } as EditBufferHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromPartial(object.context);
    } else {
      message.context = undefined;
    }
    message.text = object.text ?? "";
    message.cursor = object.cursor ?? 0;
    message.histno = object.histno ?? 0;
    return message;
  },
};

const baseInitHook: object = { calledDirect: false, bundle: "" };

export const InitHook = {
  encode(
    message: InitHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.context !== undefined) {
      ShellContext.encode(message.context, writer.uint32(10).fork()).ldelim();
    }
    if (message.calledDirect === true) {
      writer.uint32(16).bool(message.calledDirect);
    }
    if (message.bundle !== "") {
      writer.uint32(26).string(message.bundle);
    }
    Object.entries(message.env).forEach(([key, value]) => {
      InitHook_EnvEntry.encode(
        { key: key as any, value },
        writer.uint32(802).fork()
      ).ldelim();
    });
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): InitHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInitHook } as InitHook;
    message.env = {};
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.context = ShellContext.decode(reader, reader.uint32());
          break;
        case 2:
          message.calledDirect = reader.bool();
          break;
        case 3:
          message.bundle = reader.string();
          break;
        case 100:
          const entry100 = InitHook_EnvEntry.decode(reader, reader.uint32());
          if (entry100.value !== undefined) {
            message.env[entry100.key] = entry100.value;
          }
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InitHook {
    const message = { ...baseInitHook } as InitHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromJSON(object.context);
    } else {
      message.context = undefined;
    }
    if (object.calledDirect !== undefined && object.calledDirect !== null) {
      message.calledDirect = Boolean(object.calledDirect);
    } else {
      message.calledDirect = false;
    }
    if (object.bundle !== undefined && object.bundle !== null) {
      message.bundle = String(object.bundle);
    } else {
      message.bundle = "";
    }
    message.env = {};
    if (object.env !== undefined && object.env !== null) {
      Object.entries(object.env).forEach(([key, value]) => {
        message.env[key] = String(value);
      });
    }
    return message;
  },

  toJSON(message: InitHook): unknown {
    const obj: any = {};
    message.context !== undefined &&
      (obj.context = message.context
        ? ShellContext.toJSON(message.context)
        : undefined);
    message.calledDirect !== undefined &&
      (obj.calledDirect = message.calledDirect);
    message.bundle !== undefined && (obj.bundle = message.bundle);
    obj.env = {};
    if (message.env) {
      Object.entries(message.env).forEach(([k, v]) => {
        obj.env[k] = v;
      });
    }
    return obj;
  },

  fromPartial(object: DeepPartial<InitHook>): InitHook {
    const message = { ...baseInitHook } as InitHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromPartial(object.context);
    } else {
      message.context = undefined;
    }
    message.calledDirect = object.calledDirect ?? false;
    message.bundle = object.bundle ?? "";
    message.env = {};
    if (object.env !== undefined && object.env !== null) {
      Object.entries(object.env).forEach(([key, value]) => {
        if (value !== undefined) {
          message.env[key] = String(value);
        }
      });
    }
    return message;
  },
};

const baseInitHook_EnvEntry: object = { key: "", value: "" };

export const InitHook_EnvEntry = {
  encode(
    message: InitHook_EnvEntry,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.key !== "") {
      writer.uint32(10).string(message.key);
    }
    if (message.value !== "") {
      writer.uint32(18).string(message.value);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): InitHook_EnvEntry {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInitHook_EnvEntry } as InitHook_EnvEntry;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.key = reader.string();
          break;
        case 2:
          message.value = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InitHook_EnvEntry {
    const message = { ...baseInitHook_EnvEntry } as InitHook_EnvEntry;
    if (object.key !== undefined && object.key !== null) {
      message.key = String(object.key);
    } else {
      message.key = "";
    }
    if (object.value !== undefined && object.value !== null) {
      message.value = String(object.value);
    } else {
      message.value = "";
    }
    return message;
  },

  toJSON(message: InitHook_EnvEntry): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = message.key);
    message.value !== undefined && (obj.value = message.value);
    return obj;
  },

  fromPartial(object: DeepPartial<InitHook_EnvEntry>): InitHook_EnvEntry {
    const message = { ...baseInitHook_EnvEntry } as InitHook_EnvEntry;
    message.key = object.key ?? "";
    message.value = object.value ?? "";
    return message;
  },
};

const basePromptHook: object = {};

export const PromptHook = {
  encode(
    message: PromptHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.context !== undefined) {
      ShellContext.encode(message.context, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PromptHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...basePromptHook } as PromptHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.context = ShellContext.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PromptHook {
    const message = { ...basePromptHook } as PromptHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromJSON(object.context);
    } else {
      message.context = undefined;
    }
    return message;
  },

  toJSON(message: PromptHook): unknown {
    const obj: any = {};
    message.context !== undefined &&
      (obj.context = message.context
        ? ShellContext.toJSON(message.context)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<PromptHook>): PromptHook {
    const message = { ...basePromptHook } as PromptHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromPartial(object.context);
    } else {
      message.context = undefined;
    }
    return message;
  },
};

const basePreExecHook: object = {};

export const PreExecHook = {
  encode(
    message: PreExecHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.context !== undefined) {
      ShellContext.encode(message.context, writer.uint32(10).fork()).ldelim();
    }
    if (message.command !== undefined) {
      writer.uint32(18).string(message.command);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PreExecHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...basePreExecHook } as PreExecHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.context = ShellContext.decode(reader, reader.uint32());
          break;
        case 2:
          message.command = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PreExecHook {
    const message = { ...basePreExecHook } as PreExecHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromJSON(object.context);
    } else {
      message.context = undefined;
    }
    if (object.command !== undefined && object.command !== null) {
      message.command = String(object.command);
    } else {
      message.command = undefined;
    }
    return message;
  },

  toJSON(message: PreExecHook): unknown {
    const obj: any = {};
    message.context !== undefined &&
      (obj.context = message.context
        ? ShellContext.toJSON(message.context)
        : undefined);
    message.command !== undefined && (obj.command = message.command);
    return obj;
  },

  fromPartial(object: DeepPartial<PreExecHook>): PreExecHook {
    const message = { ...basePreExecHook } as PreExecHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromPartial(object.context);
    } else {
      message.context = undefined;
    }
    message.command = object.command ?? undefined;
    return message;
  },
};

const basePostExecHook: object = { command: "", exitCode: 0 };

export const PostExecHook = {
  encode(
    message: PostExecHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.context !== undefined) {
      ShellContext.encode(message.context, writer.uint32(10).fork()).ldelim();
    }
    if (message.command !== "") {
      writer.uint32(18).string(message.command);
    }
    if (message.exitCode !== 0) {
      writer.uint32(24).int32(message.exitCode);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PostExecHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...basePostExecHook } as PostExecHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.context = ShellContext.decode(reader, reader.uint32());
          break;
        case 2:
          message.command = reader.string();
          break;
        case 3:
          message.exitCode = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PostExecHook {
    const message = { ...basePostExecHook } as PostExecHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromJSON(object.context);
    } else {
      message.context = undefined;
    }
    if (object.command !== undefined && object.command !== null) {
      message.command = String(object.command);
    } else {
      message.command = "";
    }
    if (object.exitCode !== undefined && object.exitCode !== null) {
      message.exitCode = Number(object.exitCode);
    } else {
      message.exitCode = 0;
    }
    return message;
  },

  toJSON(message: PostExecHook): unknown {
    const obj: any = {};
    message.context !== undefined &&
      (obj.context = message.context
        ? ShellContext.toJSON(message.context)
        : undefined);
    message.command !== undefined && (obj.command = message.command);
    message.exitCode !== undefined && (obj.exitCode = message.exitCode);
    return obj;
  },

  fromPartial(object: DeepPartial<PostExecHook>): PostExecHook {
    const message = { ...basePostExecHook } as PostExecHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromPartial(object.context);
    } else {
      message.context = undefined;
    }
    message.command = object.command ?? "";
    message.exitCode = object.exitCode ?? 0;
    return message;
  },
};

const baseKeyboardFocusChangedHook: object = {
  appIdentifier: "",
  focusedSessionId: "",
};

export const KeyboardFocusChangedHook = {
  encode(
    message: KeyboardFocusChangedHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.appIdentifier !== "") {
      writer.uint32(10).string(message.appIdentifier);
    }
    if (message.focusedSessionId !== "") {
      writer.uint32(18).string(message.focusedSessionId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): KeyboardFocusChangedHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseKeyboardFocusChangedHook,
    } as KeyboardFocusChangedHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.appIdentifier = reader.string();
          break;
        case 2:
          message.focusedSessionId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): KeyboardFocusChangedHook {
    const message = {
      ...baseKeyboardFocusChangedHook,
    } as KeyboardFocusChangedHook;
    if (object.appIdentifier !== undefined && object.appIdentifier !== null) {
      message.appIdentifier = String(object.appIdentifier);
    } else {
      message.appIdentifier = "";
    }
    if (
      object.focusedSessionId !== undefined &&
      object.focusedSessionId !== null
    ) {
      message.focusedSessionId = String(object.focusedSessionId);
    } else {
      message.focusedSessionId = "";
    }
    return message;
  },

  toJSON(message: KeyboardFocusChangedHook): unknown {
    const obj: any = {};
    message.appIdentifier !== undefined &&
      (obj.appIdentifier = message.appIdentifier);
    message.focusedSessionId !== undefined &&
      (obj.focusedSessionId = message.focusedSessionId);
    return obj;
  },

  fromPartial(
    object: DeepPartial<KeyboardFocusChangedHook>
  ): KeyboardFocusChangedHook {
    const message = {
      ...baseKeyboardFocusChangedHook,
    } as KeyboardFocusChangedHook;
    message.appIdentifier = object.appIdentifier ?? "";
    message.focusedSessionId = object.focusedSessionId ?? "";
    return message;
  },
};

const baseTmuxPaneChangedHook: object = { paneIdentifier: 0 };

export const TmuxPaneChangedHook = {
  encode(
    message: TmuxPaneChangedHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.paneIdentifier !== 0) {
      writer.uint32(8).int32(message.paneIdentifier);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TmuxPaneChangedHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseTmuxPaneChangedHook } as TmuxPaneChangedHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.paneIdentifier = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TmuxPaneChangedHook {
    const message = { ...baseTmuxPaneChangedHook } as TmuxPaneChangedHook;
    if (object.paneIdentifier !== undefined && object.paneIdentifier !== null) {
      message.paneIdentifier = Number(object.paneIdentifier);
    } else {
      message.paneIdentifier = 0;
    }
    return message;
  },

  toJSON(message: TmuxPaneChangedHook): unknown {
    const obj: any = {};
    message.paneIdentifier !== undefined &&
      (obj.paneIdentifier = message.paneIdentifier);
    return obj;
  },

  fromPartial(object: DeepPartial<TmuxPaneChangedHook>): TmuxPaneChangedHook {
    const message = { ...baseTmuxPaneChangedHook } as TmuxPaneChangedHook;
    message.paneIdentifier = object.paneIdentifier ?? 0;
    return message;
  },
};

const baseOpenedSSHConnectionHook: object = { controlPath: "" };

export const OpenedSSHConnectionHook = {
  encode(
    message: OpenedSSHConnectionHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.context !== undefined) {
      ShellContext.encode(message.context, writer.uint32(10).fork()).ldelim();
    }
    if (message.controlPath !== "") {
      writer.uint32(18).string(message.controlPath);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): OpenedSSHConnectionHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseOpenedSSHConnectionHook,
    } as OpenedSSHConnectionHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.context = ShellContext.decode(reader, reader.uint32());
          break;
        case 2:
          message.controlPath = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): OpenedSSHConnectionHook {
    const message = {
      ...baseOpenedSSHConnectionHook,
    } as OpenedSSHConnectionHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromJSON(object.context);
    } else {
      message.context = undefined;
    }
    if (object.controlPath !== undefined && object.controlPath !== null) {
      message.controlPath = String(object.controlPath);
    } else {
      message.controlPath = "";
    }
    return message;
  },

  toJSON(message: OpenedSSHConnectionHook): unknown {
    const obj: any = {};
    message.context !== undefined &&
      (obj.context = message.context
        ? ShellContext.toJSON(message.context)
        : undefined);
    message.controlPath !== undefined &&
      (obj.controlPath = message.controlPath);
    return obj;
  },

  fromPartial(
    object: DeepPartial<OpenedSSHConnectionHook>
  ): OpenedSSHConnectionHook {
    const message = {
      ...baseOpenedSSHConnectionHook,
    } as OpenedSSHConnectionHook;
    if (object.context !== undefined && object.context !== null) {
      message.context = ShellContext.fromPartial(object.context);
    } else {
      message.context = undefined;
    }
    message.controlPath = object.controlPath ?? "";
    return message;
  },
};

const baseCallbackHook: object = { handlerId: "", filepath: "", exitCode: "" };

export const CallbackHook = {
  encode(
    message: CallbackHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.handlerId !== "") {
      writer.uint32(10).string(message.handlerId);
    }
    if (message.filepath !== "") {
      writer.uint32(18).string(message.filepath);
    }
    if (message.exitCode !== "") {
      writer.uint32(26).string(message.exitCode);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): CallbackHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCallbackHook } as CallbackHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.handlerId = reader.string();
          break;
        case 2:
          message.filepath = reader.string();
          break;
        case 3:
          message.exitCode = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CallbackHook {
    const message = { ...baseCallbackHook } as CallbackHook;
    if (object.handlerId !== undefined && object.handlerId !== null) {
      message.handlerId = String(object.handlerId);
    } else {
      message.handlerId = "";
    }
    if (object.filepath !== undefined && object.filepath !== null) {
      message.filepath = String(object.filepath);
    } else {
      message.filepath = "";
    }
    if (object.exitCode !== undefined && object.exitCode !== null) {
      message.exitCode = String(object.exitCode);
    } else {
      message.exitCode = "";
    }
    return message;
  },

  toJSON(message: CallbackHook): unknown {
    const obj: any = {};
    message.handlerId !== undefined && (obj.handlerId = message.handlerId);
    message.filepath !== undefined && (obj.filepath = message.filepath);
    message.exitCode !== undefined && (obj.exitCode = message.exitCode);
    return obj;
  },

  fromPartial(object: DeepPartial<CallbackHook>): CallbackHook {
    const message = { ...baseCallbackHook } as CallbackHook;
    message.handlerId = object.handlerId ?? "";
    message.filepath = object.filepath ?? "";
    message.exitCode = object.exitCode ?? "";
    return message;
  },
};

const baseIntegrationReadyHook: object = { identifier: "" };

export const IntegrationReadyHook = {
  encode(
    message: IntegrationReadyHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.identifier !== "") {
      writer.uint32(10).string(message.identifier);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): IntegrationReadyHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseIntegrationReadyHook } as IntegrationReadyHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.identifier = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): IntegrationReadyHook {
    const message = { ...baseIntegrationReadyHook } as IntegrationReadyHook;
    if (object.identifier !== undefined && object.identifier !== null) {
      message.identifier = String(object.identifier);
    } else {
      message.identifier = "";
    }
    return message;
  },

  toJSON(message: IntegrationReadyHook): unknown {
    const obj: any = {};
    message.identifier !== undefined && (obj.identifier = message.identifier);
    return obj;
  },

  fromPartial(object: DeepPartial<IntegrationReadyHook>): IntegrationReadyHook {
    const message = { ...baseIntegrationReadyHook } as IntegrationReadyHook;
    message.identifier = object.identifier ?? "";
    return message;
  },
};

const baseHideHook: object = {};

export const HideHook = {
  encode(_: HideHook, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): HideHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseHideHook } as HideHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): HideHook {
    const message = { ...baseHideHook } as HideHook;
    return message;
  },

  toJSON(_: HideHook): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(_: DeepPartial<HideHook>): HideHook {
    const message = { ...baseHideHook } as HideHook;
    return message;
  },
};

const baseEventHook: object = { eventName: "" };

export const EventHook = {
  encode(
    message: EventHook,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.eventName !== "") {
      writer.uint32(10).string(message.eventName);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): EventHook {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseEventHook } as EventHook;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.eventName = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): EventHook {
    const message = { ...baseEventHook } as EventHook;
    if (object.eventName !== undefined && object.eventName !== null) {
      message.eventName = String(object.eventName);
    } else {
      message.eventName = "";
    }
    return message;
  },

  toJSON(message: EventHook): unknown {
    const obj: any = {};
    message.eventName !== undefined && (obj.eventName = message.eventName);
    return obj;
  },

  fromPartial(object: DeepPartial<EventHook>): EventHook {
    const message = { ...baseEventHook } as EventHook;
    message.eventName = object.eventName ?? "";
    return message;
  },
};

const baseErrorResponse: object = {};

export const ErrorResponse = {
  encode(
    message: ErrorResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.exitCode !== undefined) {
      writer.uint32(8).int32(message.exitCode);
    }
    if (message.message !== undefined) {
      writer.uint32(18).string(message.message);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ErrorResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseErrorResponse } as ErrorResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.exitCode = reader.int32();
          break;
        case 2:
          message.message = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ErrorResponse {
    const message = { ...baseErrorResponse } as ErrorResponse;
    if (object.exitCode !== undefined && object.exitCode !== null) {
      message.exitCode = Number(object.exitCode);
    } else {
      message.exitCode = undefined;
    }
    if (object.message !== undefined && object.message !== null) {
      message.message = String(object.message);
    } else {
      message.message = undefined;
    }
    return message;
  },

  toJSON(message: ErrorResponse): unknown {
    const obj: any = {};
    message.exitCode !== undefined && (obj.exitCode = message.exitCode);
    message.message !== undefined && (obj.message = message.message);
    return obj;
  },

  fromPartial(object: DeepPartial<ErrorResponse>): ErrorResponse {
    const message = { ...baseErrorResponse } as ErrorResponse;
    message.exitCode = object.exitCode ?? undefined;
    message.message = object.message ?? undefined;
    return message;
  },
};

const baseSuccessResponse: object = {};

export const SuccessResponse = {
  encode(
    message: SuccessResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.message !== undefined) {
      writer.uint32(10).string(message.message);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SuccessResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseSuccessResponse } as SuccessResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.message = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): SuccessResponse {
    const message = { ...baseSuccessResponse } as SuccessResponse;
    if (object.message !== undefined && object.message !== null) {
      message.message = String(object.message);
    } else {
      message.message = undefined;
    }
    return message;
  },

  toJSON(message: SuccessResponse): unknown {
    const obj: any = {};
    message.message !== undefined && (obj.message = message.message);
    return obj;
  },

  fromPartial(object: DeepPartial<SuccessResponse>): SuccessResponse {
    const message = { ...baseSuccessResponse } as SuccessResponse;
    message.message = object.message ?? undefined;
    return message;
  },
};

const baseTerminalIntegration: object = { bundleIdentifier: "", name: "" };

export const TerminalIntegration = {
  encode(
    message: TerminalIntegration,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.bundleIdentifier !== "") {
      writer.uint32(10).string(message.bundleIdentifier);
    }
    if (message.name !== "") {
      writer.uint32(18).string(message.name);
    }
    if (message.status !== undefined) {
      writer.uint32(26).string(message.status);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TerminalIntegration {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseTerminalIntegration } as TerminalIntegration;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.bundleIdentifier = reader.string();
          break;
        case 2:
          message.name = reader.string();
          break;
        case 3:
          message.status = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TerminalIntegration {
    const message = { ...baseTerminalIntegration } as TerminalIntegration;
    if (
      object.bundleIdentifier !== undefined &&
      object.bundleIdentifier !== null
    ) {
      message.bundleIdentifier = String(object.bundleIdentifier);
    } else {
      message.bundleIdentifier = "";
    }
    if (object.name !== undefined && object.name !== null) {
      message.name = String(object.name);
    } else {
      message.name = "";
    }
    if (object.status !== undefined && object.status !== null) {
      message.status = String(object.status);
    } else {
      message.status = undefined;
    }
    return message;
  },

  toJSON(message: TerminalIntegration): unknown {
    const obj: any = {};
    message.bundleIdentifier !== undefined &&
      (obj.bundleIdentifier = message.bundleIdentifier);
    message.name !== undefined && (obj.name = message.name);
    message.status !== undefined && (obj.status = message.status);
    return obj;
  },

  fromPartial(object: DeepPartial<TerminalIntegration>): TerminalIntegration {
    const message = { ...baseTerminalIntegration } as TerminalIntegration;
    message.bundleIdentifier = object.bundleIdentifier ?? "";
    message.name = object.name ?? "";
    message.status = object.status ?? undefined;
    return message;
  },
};

const baseTerminalIntegrationsListResponse: object = {};

export const TerminalIntegrationsListResponse = {
  encode(
    message: TerminalIntegrationsListResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.integrations) {
      TerminalIntegration.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): TerminalIntegrationsListResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseTerminalIntegrationsListResponse,
    } as TerminalIntegrationsListResponse;
    message.integrations = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.integrations.push(
            TerminalIntegration.decode(reader, reader.uint32())
          );
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TerminalIntegrationsListResponse {
    const message = {
      ...baseTerminalIntegrationsListResponse,
    } as TerminalIntegrationsListResponse;
    message.integrations = (object.integrations ?? []).map((e: any) =>
      TerminalIntegration.fromJSON(e)
    );
    return message;
  },

  toJSON(message: TerminalIntegrationsListResponse): unknown {
    const obj: any = {};
    if (message.integrations) {
      obj.integrations = message.integrations.map((e) =>
        e ? TerminalIntegration.toJSON(e) : undefined
      );
    } else {
      obj.integrations = [];
    }
    return obj;
  },

  fromPartial(
    object: DeepPartial<TerminalIntegrationsListResponse>
  ): TerminalIntegrationsListResponse {
    const message = {
      ...baseTerminalIntegrationsListResponse,
    } as TerminalIntegrationsListResponse;
    message.integrations = (object.integrations ?? []).map((e) =>
      TerminalIntegration.fromPartial(e)
    );
    return message;
  },
};

const baseDiagnosticsResponse: object = {
  distribution: "",
  beta: false,
  debugAutocomplete: false,
  developerModeEnabled: false,
  currentLayoutName: "",
  isRunningOnReadOnlyVolume: false,
  pathToBundle: "",
  accessibility: "",
  keypath: "",
  docker: "",
  symlinked: "",
  onlytab: "",
  installscript: "",
  psudoterminalPath: "",
  securekeyboard: "",
  securekeyboardPath: "",
  currentProcess: "",
  currentWindowIdentifier: "",
  autocomplete: false,
};

export const DiagnosticsResponse = {
  encode(
    message: DiagnosticsResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.distribution !== "") {
      writer.uint32(10).string(message.distribution);
    }
    if (message.beta === true) {
      writer.uint32(16).bool(message.beta);
    }
    if (message.debugAutocomplete === true) {
      writer.uint32(24).bool(message.debugAutocomplete);
    }
    if (message.developerModeEnabled === true) {
      writer.uint32(32).bool(message.developerModeEnabled);
    }
    if (message.currentLayoutName !== "") {
      writer.uint32(42).string(message.currentLayoutName);
    }
    if (message.isRunningOnReadOnlyVolume === true) {
      writer.uint32(48).bool(message.isRunningOnReadOnlyVolume);
    }
    if (message.pathToBundle !== "") {
      writer.uint32(58).string(message.pathToBundle);
    }
    if (message.accessibility !== "") {
      writer.uint32(66).string(message.accessibility);
    }
    if (message.keypath !== "") {
      writer.uint32(74).string(message.keypath);
    }
    if (message.docker !== "") {
      writer.uint32(82).string(message.docker);
    }
    if (message.symlinked !== "") {
      writer.uint32(90).string(message.symlinked);
    }
    if (message.onlytab !== "") {
      writer.uint32(98).string(message.onlytab);
    }
    if (message.installscript !== "") {
      writer.uint32(106).string(message.installscript);
    }
    if (message.psudoterminalPath !== "") {
      writer.uint32(114).string(message.psudoterminalPath);
    }
    if (message.securekeyboard !== "") {
      writer.uint32(122).string(message.securekeyboard);
    }
    if (message.securekeyboardPath !== "") {
      writer.uint32(130).string(message.securekeyboardPath);
    }
    if (message.currentProcess !== "") {
      writer.uint32(138).string(message.currentProcess);
    }
    if (message.currentWindowIdentifier !== "") {
      writer.uint32(146).string(message.currentWindowIdentifier);
    }
    if (message.autocomplete === true) {
      writer.uint32(152).bool(message.autocomplete);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): DiagnosticsResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseDiagnosticsResponse } as DiagnosticsResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.distribution = reader.string();
          break;
        case 2:
          message.beta = reader.bool();
          break;
        case 3:
          message.debugAutocomplete = reader.bool();
          break;
        case 4:
          message.developerModeEnabled = reader.bool();
          break;
        case 5:
          message.currentLayoutName = reader.string();
          break;
        case 6:
          message.isRunningOnReadOnlyVolume = reader.bool();
          break;
        case 7:
          message.pathToBundle = reader.string();
          break;
        case 8:
          message.accessibility = reader.string();
          break;
        case 9:
          message.keypath = reader.string();
          break;
        case 10:
          message.docker = reader.string();
          break;
        case 11:
          message.symlinked = reader.string();
          break;
        case 12:
          message.onlytab = reader.string();
          break;
        case 13:
          message.installscript = reader.string();
          break;
        case 14:
          message.psudoterminalPath = reader.string();
          break;
        case 15:
          message.securekeyboard = reader.string();
          break;
        case 16:
          message.securekeyboardPath = reader.string();
          break;
        case 17:
          message.currentProcess = reader.string();
          break;
        case 18:
          message.currentWindowIdentifier = reader.string();
          break;
        case 19:
          message.autocomplete = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): DiagnosticsResponse {
    const message = { ...baseDiagnosticsResponse } as DiagnosticsResponse;
    if (object.distribution !== undefined && object.distribution !== null) {
      message.distribution = String(object.distribution);
    } else {
      message.distribution = "";
    }
    if (object.beta !== undefined && object.beta !== null) {
      message.beta = Boolean(object.beta);
    } else {
      message.beta = false;
    }
    if (
      object.debugAutocomplete !== undefined &&
      object.debugAutocomplete !== null
    ) {
      message.debugAutocomplete = Boolean(object.debugAutocomplete);
    } else {
      message.debugAutocomplete = false;
    }
    if (
      object.developerModeEnabled !== undefined &&
      object.developerModeEnabled !== null
    ) {
      message.developerModeEnabled = Boolean(object.developerModeEnabled);
    } else {
      message.developerModeEnabled = false;
    }
    if (
      object.currentLayoutName !== undefined &&
      object.currentLayoutName !== null
    ) {
      message.currentLayoutName = String(object.currentLayoutName);
    } else {
      message.currentLayoutName = "";
    }
    if (
      object.isRunningOnReadOnlyVolume !== undefined &&
      object.isRunningOnReadOnlyVolume !== null
    ) {
      message.isRunningOnReadOnlyVolume = Boolean(
        object.isRunningOnReadOnlyVolume
      );
    } else {
      message.isRunningOnReadOnlyVolume = false;
    }
    if (object.pathToBundle !== undefined && object.pathToBundle !== null) {
      message.pathToBundle = String(object.pathToBundle);
    } else {
      message.pathToBundle = "";
    }
    if (object.accessibility !== undefined && object.accessibility !== null) {
      message.accessibility = String(object.accessibility);
    } else {
      message.accessibility = "";
    }
    if (object.keypath !== undefined && object.keypath !== null) {
      message.keypath = String(object.keypath);
    } else {
      message.keypath = "";
    }
    if (object.docker !== undefined && object.docker !== null) {
      message.docker = String(object.docker);
    } else {
      message.docker = "";
    }
    if (object.symlinked !== undefined && object.symlinked !== null) {
      message.symlinked = String(object.symlinked);
    } else {
      message.symlinked = "";
    }
    if (object.onlytab !== undefined && object.onlytab !== null) {
      message.onlytab = String(object.onlytab);
    } else {
      message.onlytab = "";
    }
    if (object.installscript !== undefined && object.installscript !== null) {
      message.installscript = String(object.installscript);
    } else {
      message.installscript = "";
    }
    if (
      object.psudoterminalPath !== undefined &&
      object.psudoterminalPath !== null
    ) {
      message.psudoterminalPath = String(object.psudoterminalPath);
    } else {
      message.psudoterminalPath = "";
    }
    if (object.securekeyboard !== undefined && object.securekeyboard !== null) {
      message.securekeyboard = String(object.securekeyboard);
    } else {
      message.securekeyboard = "";
    }
    if (
      object.securekeyboardPath !== undefined &&
      object.securekeyboardPath !== null
    ) {
      message.securekeyboardPath = String(object.securekeyboardPath);
    } else {
      message.securekeyboardPath = "";
    }
    if (object.currentProcess !== undefined && object.currentProcess !== null) {
      message.currentProcess = String(object.currentProcess);
    } else {
      message.currentProcess = "";
    }
    if (
      object.currentWindowIdentifier !== undefined &&
      object.currentWindowIdentifier !== null
    ) {
      message.currentWindowIdentifier = String(object.currentWindowIdentifier);
    } else {
      message.currentWindowIdentifier = "";
    }
    if (object.autocomplete !== undefined && object.autocomplete !== null) {
      message.autocomplete = Boolean(object.autocomplete);
    } else {
      message.autocomplete = false;
    }
    return message;
  },

  toJSON(message: DiagnosticsResponse): unknown {
    const obj: any = {};
    message.distribution !== undefined &&
      (obj.distribution = message.distribution);
    message.beta !== undefined && (obj.beta = message.beta);
    message.debugAutocomplete !== undefined &&
      (obj.debugAutocomplete = message.debugAutocomplete);
    message.developerModeEnabled !== undefined &&
      (obj.developerModeEnabled = message.developerModeEnabled);
    message.currentLayoutName !== undefined &&
      (obj.currentLayoutName = message.currentLayoutName);
    message.isRunningOnReadOnlyVolume !== undefined &&
      (obj.isRunningOnReadOnlyVolume = message.isRunningOnReadOnlyVolume);
    message.pathToBundle !== undefined &&
      (obj.pathToBundle = message.pathToBundle);
    message.accessibility !== undefined &&
      (obj.accessibility = message.accessibility);
    message.keypath !== undefined && (obj.keypath = message.keypath);
    message.docker !== undefined && (obj.docker = message.docker);
    message.symlinked !== undefined && (obj.symlinked = message.symlinked);
    message.onlytab !== undefined && (obj.onlytab = message.onlytab);
    message.installscript !== undefined &&
      (obj.installscript = message.installscript);
    message.psudoterminalPath !== undefined &&
      (obj.psudoterminalPath = message.psudoterminalPath);
    message.securekeyboard !== undefined &&
      (obj.securekeyboard = message.securekeyboard);
    message.securekeyboardPath !== undefined &&
      (obj.securekeyboardPath = message.securekeyboardPath);
    message.currentProcess !== undefined &&
      (obj.currentProcess = message.currentProcess);
    message.currentWindowIdentifier !== undefined &&
      (obj.currentWindowIdentifier = message.currentWindowIdentifier);
    message.autocomplete !== undefined &&
      (obj.autocomplete = message.autocomplete);
    return obj;
  },

  fromPartial(object: DeepPartial<DiagnosticsResponse>): DiagnosticsResponse {
    const message = { ...baseDiagnosticsResponse } as DiagnosticsResponse;
    message.distribution = object.distribution ?? "";
    message.beta = object.beta ?? false;
    message.debugAutocomplete = object.debugAutocomplete ?? false;
    message.developerModeEnabled = object.developerModeEnabled ?? false;
    message.currentLayoutName = object.currentLayoutName ?? "";
    message.isRunningOnReadOnlyVolume =
      object.isRunningOnReadOnlyVolume ?? false;
    message.pathToBundle = object.pathToBundle ?? "";
    message.accessibility = object.accessibility ?? "";
    message.keypath = object.keypath ?? "";
    message.docker = object.docker ?? "";
    message.symlinked = object.symlinked ?? "";
    message.onlytab = object.onlytab ?? "";
    message.installscript = object.installscript ?? "";
    message.psudoterminalPath = object.psudoterminalPath ?? "";
    message.securekeyboard = object.securekeyboard ?? "";
    message.securekeyboardPath = object.securekeyboardPath ?? "";
    message.currentProcess = object.currentProcess ?? "";
    message.currentWindowIdentifier = object.currentWindowIdentifier ?? "";
    message.autocomplete = object.autocomplete ?? false;
    return message;
  },
};

const baseCommandResponse: object = {};

export const CommandResponse = {
  encode(
    message: CommandResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== undefined) {
      writer.uint32(8).int64(message.id);
    }
    if (message.response?.$case === "error") {
      ErrorResponse.encode(
        message.response.error,
        writer.uint32(18).fork()
      ).ldelim();
    }
    if (message.response?.$case === "success") {
      SuccessResponse.encode(
        message.response.success,
        writer.uint32(26).fork()
      ).ldelim();
    }
    if (message.response?.$case === "integrationList") {
      TerminalIntegrationsListResponse.encode(
        message.response.integrationList,
        writer.uint32(802).fork()
      ).ldelim();
    }
    if (message.response?.$case === "diagnostics") {
      DiagnosticsResponse.encode(
        message.response.diagnostics,
        writer.uint32(810).fork()
      ).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): CommandResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCommandResponse } as CommandResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = longToNumber(reader.int64() as Long);
          break;
        case 2:
          message.response = {
            $case: "error",
            error: ErrorResponse.decode(reader, reader.uint32()),
          };
          break;
        case 3:
          message.response = {
            $case: "success",
            success: SuccessResponse.decode(reader, reader.uint32()),
          };
          break;
        case 100:
          message.response = {
            $case: "integrationList",
            integrationList: TerminalIntegrationsListResponse.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 101:
          message.response = {
            $case: "diagnostics",
            diagnostics: DiagnosticsResponse.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CommandResponse {
    const message = { ...baseCommandResponse } as CommandResponse;
    if (object.id !== undefined && object.id !== null) {
      message.id = Number(object.id);
    } else {
      message.id = undefined;
    }
    if (object.error !== undefined && object.error !== null) {
      message.response = {
        $case: "error",
        error: ErrorResponse.fromJSON(object.error),
      };
    }
    if (object.success !== undefined && object.success !== null) {
      message.response = {
        $case: "success",
        success: SuccessResponse.fromJSON(object.success),
      };
    }
    if (
      object.integrationList !== undefined &&
      object.integrationList !== null
    ) {
      message.response = {
        $case: "integrationList",
        integrationList: TerminalIntegrationsListResponse.fromJSON(
          object.integrationList
        ),
      };
    }
    if (object.diagnostics !== undefined && object.diagnostics !== null) {
      message.response = {
        $case: "diagnostics",
        diagnostics: DiagnosticsResponse.fromJSON(object.diagnostics),
      };
    }
    return message;
  },

  toJSON(message: CommandResponse): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.response?.$case === "error" &&
      (obj.error = message.response?.error
        ? ErrorResponse.toJSON(message.response?.error)
        : undefined);
    message.response?.$case === "success" &&
      (obj.success = message.response?.success
        ? SuccessResponse.toJSON(message.response?.success)
        : undefined);
    message.response?.$case === "integrationList" &&
      (obj.integrationList = message.response?.integrationList
        ? TerminalIntegrationsListResponse.toJSON(
            message.response?.integrationList
          )
        : undefined);
    message.response?.$case === "diagnostics" &&
      (obj.diagnostics = message.response?.diagnostics
        ? DiagnosticsResponse.toJSON(message.response?.diagnostics)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<CommandResponse>): CommandResponse {
    const message = { ...baseCommandResponse } as CommandResponse;
    message.id = object.id ?? undefined;
    if (
      object.response?.$case === "error" &&
      object.response?.error !== undefined &&
      object.response?.error !== null
    ) {
      message.response = {
        $case: "error",
        error: ErrorResponse.fromPartial(object.response.error),
      };
    }
    if (
      object.response?.$case === "success" &&
      object.response?.success !== undefined &&
      object.response?.success !== null
    ) {
      message.response = {
        $case: "success",
        success: SuccessResponse.fromPartial(object.response.success),
      };
    }
    if (
      object.response?.$case === "integrationList" &&
      object.response?.integrationList !== undefined &&
      object.response?.integrationList !== null
    ) {
      message.response = {
        $case: "integrationList",
        integrationList: TerminalIntegrationsListResponse.fromPartial(
          object.response.integrationList
        ),
      };
    }
    if (
      object.response?.$case === "diagnostics" &&
      object.response?.diagnostics !== undefined &&
      object.response?.diagnostics !== null
    ) {
      message.response = {
        $case: "diagnostics",
        diagnostics: DiagnosticsResponse.fromPartial(
          object.response.diagnostics
        ),
      };
    }
    return message;
  },
};

declare var self: any | undefined;
declare var window: any | undefined;
declare var global: any | undefined;
var globalThis: any = (() => {
  if (typeof globalThis !== "undefined") return globalThis;
  if (typeof self !== "undefined") return self;
  if (typeof window !== "undefined") return window;
  if (typeof global !== "undefined") return global;
  throw "Unable to locate global object";
})();

type Builtin =
  | Date
  | Function
  | Uint8Array
  | string
  | number
  | boolean
  | undefined;
export type DeepPartial<T> = T extends Builtin
  ? T
  : T extends Array<infer U>
  ? Array<DeepPartial<U>>
  : T extends ReadonlyArray<infer U>
  ? ReadonlyArray<DeepPartial<U>>
  : T extends { $case: string }
  ? { [K in keyof Omit<T, "$case">]?: DeepPartial<T[K]> } & {
      $case: T["$case"];
    }
  : T extends {}
  ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

function longToNumber(long: Long): number {
  if (long.gt(Number.MAX_SAFE_INTEGER)) {
    throw new globalThis.Error("Value is larger than Number.MAX_SAFE_INTEGER");
  }
  return long.toNumber();
}

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}
