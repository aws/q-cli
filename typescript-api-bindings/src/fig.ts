/* eslint-disable */
import Long from "long";
import _m0 from "protobufjs/minimal";

export const protobufPackage = "fig";

export enum Modifiers {
  CONTROL = 0,
  OPTION = 1,
  COMMAND = 2,
  SHIFT = 3,
  FUNCTION = 4,
  NUMPAD = 5,
  UNRECOGNIZED = -1,
}

export function modifiersFromJSON(object: any): Modifiers {
  switch (object) {
    case 0:
    case "CONTROL":
      return Modifiers.CONTROL;
    case 1:
    case "OPTION":
      return Modifiers.OPTION;
    case 2:
    case "COMMAND":
      return Modifiers.COMMAND;
    case 3:
    case "SHIFT":
      return Modifiers.SHIFT;
    case 4:
    case "FUNCTION":
      return Modifiers.FUNCTION;
    case 5:
    case "NUMPAD":
      return Modifiers.NUMPAD;
    case -1:
    case "UNRECOGNIZED":
    default:
      return Modifiers.UNRECOGNIZED;
  }
}

export function modifiersToJSON(object: Modifiers): string {
  switch (object) {
    case Modifiers.CONTROL:
      return "CONTROL";
    case Modifiers.OPTION:
      return "OPTION";
    case Modifiers.COMMAND:
      return "COMMAND";
    case Modifiers.SHIFT:
      return "SHIFT";
    case Modifiers.FUNCTION:
      return "FUNCTION";
    case Modifiers.NUMPAD:
      return "NUMPAD";
    default:
      return "UNKNOWN";
  }
}

export enum ActionAvailability {
  ALWAYS = 0,
  /** WHEN_FOCUSED - the action can only be performed when the app has keyboard focus */
  WHEN_FOCUSED = 1,
  /** WHEN_VISIBLE - the action can only be performed when the app is visible */
  WHEN_VISIBLE = 2,
  /** WHEN_HIDDEN - the action can only be performed when the app is hidden */
  WHEN_HIDDEN = 3,
  UNRECOGNIZED = -1,
}

export function actionAvailabilityFromJSON(object: any): ActionAvailability {
  switch (object) {
    case 0:
    case "ALWAYS":
      return ActionAvailability.ALWAYS;
    case 1:
    case "WHEN_FOCUSED":
      return ActionAvailability.WHEN_FOCUSED;
    case 2:
    case "WHEN_VISIBLE":
      return ActionAvailability.WHEN_VISIBLE;
    case 3:
    case "WHEN_HIDDEN":
      return ActionAvailability.WHEN_HIDDEN;
    case -1:
    case "UNRECOGNIZED":
    default:
      return ActionAvailability.UNRECOGNIZED;
  }
}

export function actionAvailabilityToJSON(object: ActionAvailability): string {
  switch (object) {
    case ActionAvailability.ALWAYS:
      return "ALWAYS";
    case ActionAvailability.WHEN_FOCUSED:
      return "WHEN_FOCUSED";
    case ActionAvailability.WHEN_VISIBLE:
      return "WHEN_VISIBLE";
    case ActionAvailability.WHEN_HIDDEN:
      return "WHEN_HIDDEN";
    default:
      return "UNKNOWN";
  }
}

export enum NotificationType {
  ALL = 0,
  NOTIFY_ON_EDITBUFFFER_CHANGE = 1,
  NOTIFY_ON_SETTINGS_CHANGE = 2,
  NOTIFY_ON_PROMPT = 3,
  NOTIFY_ON_LOCATION_CHANGE = 4,
  NOTIFY_ON_PROCESS_CHANGED = 5,
  NOTIFY_ON_KEYBINDING_PRESSED = 6,
  NOTIFY_ON_FOCUS_CHANGED = 7,
  UNRECOGNIZED = -1,
}

export function notificationTypeFromJSON(object: any): NotificationType {
  switch (object) {
    case 0:
    case "ALL":
      return NotificationType.ALL;
    case 1:
    case "NOTIFY_ON_EDITBUFFFER_CHANGE":
      return NotificationType.NOTIFY_ON_EDITBUFFFER_CHANGE;
    case 2:
    case "NOTIFY_ON_SETTINGS_CHANGE":
      return NotificationType.NOTIFY_ON_SETTINGS_CHANGE;
    case 3:
    case "NOTIFY_ON_PROMPT":
      return NotificationType.NOTIFY_ON_PROMPT;
    case 4:
    case "NOTIFY_ON_LOCATION_CHANGE":
      return NotificationType.NOTIFY_ON_LOCATION_CHANGE;
    case 5:
    case "NOTIFY_ON_PROCESS_CHANGED":
      return NotificationType.NOTIFY_ON_PROCESS_CHANGED;
    case 6:
    case "NOTIFY_ON_KEYBINDING_PRESSED":
      return NotificationType.NOTIFY_ON_KEYBINDING_PRESSED;
    case 7:
    case "NOTIFY_ON_FOCUS_CHANGED":
      return NotificationType.NOTIFY_ON_FOCUS_CHANGED;
    case -1:
    case "UNRECOGNIZED":
    default:
      return NotificationType.UNRECOGNIZED;
  }
}

export function notificationTypeToJSON(object: NotificationType): string {
  switch (object) {
    case NotificationType.ALL:
      return "ALL";
    case NotificationType.NOTIFY_ON_EDITBUFFFER_CHANGE:
      return "NOTIFY_ON_EDITBUFFFER_CHANGE";
    case NotificationType.NOTIFY_ON_SETTINGS_CHANGE:
      return "NOTIFY_ON_SETTINGS_CHANGE";
    case NotificationType.NOTIFY_ON_PROMPT:
      return "NOTIFY_ON_PROMPT";
    case NotificationType.NOTIFY_ON_LOCATION_CHANGE:
      return "NOTIFY_ON_LOCATION_CHANGE";
    case NotificationType.NOTIFY_ON_PROCESS_CHANGED:
      return "NOTIFY_ON_PROCESS_CHANGED";
    case NotificationType.NOTIFY_ON_KEYBINDING_PRESSED:
      return "NOTIFY_ON_KEYBINDING_PRESSED";
    case NotificationType.NOTIFY_ON_FOCUS_CHANGED:
      return "NOTIFY_ON_FOCUS_CHANGED";
    default:
      return "UNKNOWN";
  }
}

export interface ClientOriginatedMessage {
  id?: number | undefined;
  submessage?:
    | {
        $case: "positionWindowRequest";
        positionWindowRequest: PositionWindowRequest;
      }
    | {
        $case: "pseudoterminalExecuteRequest";
        pseudoterminalExecuteRequest: PseudoterminalExecuteRequest;
      }
    | {
        $case: "pseudoterminalWriteRequest";
        pseudoterminalWriteRequest: PseudoterminalWriteRequest;
      }
    | { $case: "readFileRequest"; readFileRequest: ReadFileRequest }
    | { $case: "writeFileRequest"; writeFileRequest: WriteFileRequest }
    | {
        $case: "contentsOfDirectoryRequest";
        contentsOfDirectoryRequest: ContentsOfDirectoryRequest;
      }
    | { $case: "notificationRequest"; notificationRequest: NotificationRequest }
    | {
        $case: "getSettingsPropertyRequest";
        getSettingsPropertyRequest: GetSettingsPropertyRequest;
      }
    | {
        $case: "updateSettingsPropertyRequest";
        updateSettingsPropertyRequest: UpdateSettingsPropertyRequest;
      }
    | { $case: "insertTextRequest"; insertTextRequest: InsertTextRequest }
    | {
        $case: "updateApplicationPropertiesRequest";
        updateApplicationPropertiesRequest: UpdateApplicationPropertiesRequest;
      }
    | {
        $case: "destinationOfSymbolicLinkRequest";
        destinationOfSymbolicLinkRequest: DestinationOfSymbolicLinkRequest;
      };
}

export interface ServerOriginatedMessage {
  id?: number | undefined;
  submessage?:
    | { $case: "error"; error: string }
    | { $case: "success"; success: boolean }
    | {
        $case: "positionWindowResponse";
        positionWindowResponse: PositionWindowResponse;
      }
    | {
        $case: "pseudoterminalExecuteResponse";
        pseudoterminalExecuteResponse: PseudoterminalExecuteResponse;
      }
    | { $case: "readFileResponse"; readFileResponse: ReadFileResponse }
    | {
        $case: "contentsOfDirectoryResponse";
        contentsOfDirectoryResponse: ContentsOfDirectoryResponse;
      }
    | {
        $case: "getSettingsPropertyResponse";
        getSettingsPropertyResponse: GetSettingsPropertyResponse;
      }
    | {
        $case: "destinationOfSymbolicLinkResponse";
        destinationOfSymbolicLinkResponse: DestinationOfSymbolicLinkResponse;
      }
    | { $case: "notification"; notification: Notification };
}

export interface Point {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface Frame {
  origin: Point | undefined;
  size: Size | undefined;
}

export interface EnvironmentVariable {
  key: string;
  value?: string | undefined;
}

export interface Process {
  pid?: number | undefined;
  executable?: string | undefined;
  directory?: string | undefined;
  env: EnvironmentVariable[];
}

export interface FilePath {
  path?: string | undefined;
  relativeTo?: string | undefined;
  expandTildeInPath?: boolean | undefined;
}

export interface KeyEvent {
  appleKeyCode?: number | undefined;
  characters?: string | undefined;
  charactersIgnoringModifiers?: string | undefined;
  modifiers: Modifiers[];
  isRepeat?: boolean | undefined;
}

export interface Screen {
  frame?: Frame | undefined;
}

export interface Session {
  sessionId?: string | undefined;
  frontmostProcess?: Process | undefined;
  env: EnvironmentVariable[];
}

export interface Application {
  bundleIdentifier?: string | undefined;
  name?: string | undefined;
}

export interface Window {
  windowId?: string | undefined;
  frame?: Frame | undefined;
  currentSession?: Session | undefined;
  app?: Application | undefined;
  currentScreen?: Screen | undefined;
}

export interface TextUpdate {
  insertion?: string | undefined;
  deletion?: number | undefined;
  offset?: number | undefined;
  immediate?: boolean | undefined;
}

export interface InsertTextRequest {
  type?:
    | { $case: "text"; text: string }
    | { $case: "update"; update: TextUpdate };
}

export interface PseudoterminalWriteRequest {
  input?:
    | { $case: "text"; text: string }
    | { $case: "octal"; octal: Uint8Array };
}

export interface PseudoterminalExecuteRequest {
  command: string;
  workingDirectory?: string | undefined;
  backgroundJob?: boolean | undefined;
  isPipelined?: boolean | undefined;
  env: EnvironmentVariable[];
}

export interface PseudoterminalExecuteResponse {
  stdout: string;
  stderr?: string | undefined;
  exitCode?: number | undefined;
}

export interface PositionWindowRequest {
  anchor: Point | undefined;
  size: Size | undefined;
  dryrun?: boolean | undefined;
}

export interface PositionWindowResponse {
  isAbove?: boolean | undefined;
  isClipped?: boolean | undefined;
}

export interface ReadFileRequest {
  path?: FilePath | undefined;
}

export interface ReadFileResponse {
  data?: Uint8Array | undefined;
}

export interface WriteFileRequest {
  path?: FilePath | undefined;
  data?:
    | { $case: "text"; text: string }
    | { $case: "binary"; binary: Uint8Array };
}

export interface ContentsOfDirectoryRequest {
  directory?: FilePath | undefined;
}

export interface ContentsOfDirectoryResponse {
  fileNames: string[];
}

export interface DestinationOfSymbolicLinkRequest {
  path?: FilePath | undefined;
}

export interface DestinationOfSymbolicLinkResponse {
  destination?: FilePath | undefined;
}

export interface GetSettingsPropertyRequest {
  key?: string | undefined;
}

export interface GetSettingsPropertyResponse {
  jsonBlob?: string | undefined;
  isDefault?: boolean | undefined;
}

export interface UpdateSettingsPropertyRequest {
  key?: string | undefined;
  value?: string | undefined;
}

export interface Action {
  /** unique identifier for the action; not user facing. */
  identifier?: string | undefined;
  /** name of action, will appear in user interfaces. */
  name?: string | undefined;
  /** a quick summary of what the action will do */
  description?: string | undefined;
  category?: string | undefined;
  /** when can this action be performed */
  availability?: ActionAvailability | undefined;
}

export interface UpdateApplicationPropertiesRequest {
  interceptBoundKeystrokes?: boolean | undefined;
  actions: Action[];
}

export interface NotificationRequest {
  subscribe?: boolean | undefined;
  type?: NotificationType | undefined;
}

export interface Notification {
  type?:
    | {
        $case: "editBufferNotification";
        editBufferNotification: EditBufferChangedNotification;
      }
    | {
        $case: "settingsChangedNotification";
        settingsChangedNotification: SettingsChangedNotification;
      }
    | {
        $case: "shellPromptReturnedNotification";
        shellPromptReturnedNotification: ShellPromptReturnedNotification;
      }
    | {
        $case: "locationChangedNotification";
        locationChangedNotification: LocationChangedNotification;
      }
    | {
        $case: "processChangeNotification";
        processChangeNotification: ProcessChangedNotification;
      }
    | {
        $case: "keybindingPressedNotification";
        keybindingPressedNotification: KeybindingPressedNotification;
      }
    | {
        $case: "windowFocusChangedNotification";
        windowFocusChangedNotification: WindowFocusChangedNotification;
      };
}

export interface EditBufferChangedNotification {
  sessionId?: string | undefined;
  cursor?: number | undefined;
  buffer?: string | undefined;
}

export interface SettingsChangedNotification {
  jsonBlob?: string | undefined;
}

export interface ShellPromptReturnedNotification {
  sessionId?: string | undefined;
  shell?: Process | undefined;
}

export interface LocationChangedNotification {
  sessionId?: string | undefined;
  hostName?: string | undefined;
  userName?: string | undefined;
  directory?: string | undefined;
}

export interface ProcessChangedNotification {
  sessionId?: string | undefined;
  newProcess?: Process | undefined;
}

export interface KeybindingPressedNotification {
  keypress?: KeyEvent | undefined;
  action?: string | undefined;
}

export interface WindowFocusChangedNotification {
  window?: Window | undefined;
}

/**
 * / Constants
 * Can be found under fig.constants. Certain legacy constants are included at the top-level for backwards compatibility.
 */
export interface Constants {
  /** the current version of Fig */
  version?: string | undefined;
  /** the current build of Fig */
  build?: string | undefined;
  /** the path to the figcli executable */
  cli?: string | undefined;
  /** the filepath of the macOS bundle */
  bundlePath?: string | undefined;
  /** the remote URL where apps are loaded from */
  remote?: string | undefined;
  /** the user's home directory */
  home?: string | undefined;
  /** the user's name (equivalent to running `whoami`) */
  user?: string | undefined;
  /** the default macOS $PATH */
  defaultPath?: string | undefined;
  jsonMessageRecieved?: string | undefined;
  jsonMessageHandler?: string | undefined;
  protoMessageRecieved?: string | undefined;
  protoMessageHandler?: string | undefined;
  /** a comma-separated list of all themes */
  themes?: string | undefined;
}

const baseClientOriginatedMessage: object = {};

export const ClientOriginatedMessage = {
  encode(
    message: ClientOriginatedMessage,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== undefined) {
      writer.uint32(8).int64(message.id);
    }
    if (message.submessage?.$case === "positionWindowRequest") {
      PositionWindowRequest.encode(
        message.submessage.positionWindowRequest,
        writer.uint32(810).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "pseudoterminalExecuteRequest") {
      PseudoterminalExecuteRequest.encode(
        message.submessage.pseudoterminalExecuteRequest,
        writer.uint32(818).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "pseudoterminalWriteRequest") {
      PseudoterminalWriteRequest.encode(
        message.submessage.pseudoterminalWriteRequest,
        writer.uint32(826).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "readFileRequest") {
      ReadFileRequest.encode(
        message.submessage.readFileRequest,
        writer.uint32(834).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "writeFileRequest") {
      WriteFileRequest.encode(
        message.submessage.writeFileRequest,
        writer.uint32(842).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "contentsOfDirectoryRequest") {
      ContentsOfDirectoryRequest.encode(
        message.submessage.contentsOfDirectoryRequest,
        writer.uint32(850).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "notificationRequest") {
      NotificationRequest.encode(
        message.submessage.notificationRequest,
        writer.uint32(858).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "getSettingsPropertyRequest") {
      GetSettingsPropertyRequest.encode(
        message.submessage.getSettingsPropertyRequest,
        writer.uint32(866).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "updateSettingsPropertyRequest") {
      UpdateSettingsPropertyRequest.encode(
        message.submessage.updateSettingsPropertyRequest,
        writer.uint32(874).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "insertTextRequest") {
      InsertTextRequest.encode(
        message.submessage.insertTextRequest,
        writer.uint32(882).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "updateApplicationPropertiesRequest") {
      UpdateApplicationPropertiesRequest.encode(
        message.submessage.updateApplicationPropertiesRequest,
        writer.uint32(890).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "destinationOfSymbolicLinkRequest") {
      DestinationOfSymbolicLinkRequest.encode(
        message.submessage.destinationOfSymbolicLinkRequest,
        writer.uint32(898).fork()
      ).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ClientOriginatedMessage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseClientOriginatedMessage,
    } as ClientOriginatedMessage;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = longToNumber(reader.int64() as Long);
          break;
        case 101:
          message.submessage = {
            $case: "positionWindowRequest",
            positionWindowRequest: PositionWindowRequest.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 102:
          message.submessage = {
            $case: "pseudoterminalExecuteRequest",
            pseudoterminalExecuteRequest: PseudoterminalExecuteRequest.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 103:
          message.submessage = {
            $case: "pseudoterminalWriteRequest",
            pseudoterminalWriteRequest: PseudoterminalWriteRequest.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 104:
          message.submessage = {
            $case: "readFileRequest",
            readFileRequest: ReadFileRequest.decode(reader, reader.uint32()),
          };
          break;
        case 105:
          message.submessage = {
            $case: "writeFileRequest",
            writeFileRequest: WriteFileRequest.decode(reader, reader.uint32()),
          };
          break;
        case 106:
          message.submessage = {
            $case: "contentsOfDirectoryRequest",
            contentsOfDirectoryRequest: ContentsOfDirectoryRequest.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 107:
          message.submessage = {
            $case: "notificationRequest",
            notificationRequest: NotificationRequest.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 108:
          message.submessage = {
            $case: "getSettingsPropertyRequest",
            getSettingsPropertyRequest: GetSettingsPropertyRequest.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 109:
          message.submessage = {
            $case: "updateSettingsPropertyRequest",
            updateSettingsPropertyRequest: UpdateSettingsPropertyRequest.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 110:
          message.submessage = {
            $case: "insertTextRequest",
            insertTextRequest: InsertTextRequest.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 111:
          message.submessage = {
            $case: "updateApplicationPropertiesRequest",
            updateApplicationPropertiesRequest:
              UpdateApplicationPropertiesRequest.decode(
                reader,
                reader.uint32()
              ),
          };
          break;
        case 112:
          message.submessage = {
            $case: "destinationOfSymbolicLinkRequest",
            destinationOfSymbolicLinkRequest:
              DestinationOfSymbolicLinkRequest.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ClientOriginatedMessage {
    const message = {
      ...baseClientOriginatedMessage,
    } as ClientOriginatedMessage;
    if (object.id !== undefined && object.id !== null) {
      message.id = Number(object.id);
    }
    if (
      object.positionWindowRequest !== undefined &&
      object.positionWindowRequest !== null
    ) {
      message.submessage = {
        $case: "positionWindowRequest",
        positionWindowRequest: PositionWindowRequest.fromJSON(
          object.positionWindowRequest
        ),
      };
    }
    if (
      object.pseudoterminalExecuteRequest !== undefined &&
      object.pseudoterminalExecuteRequest !== null
    ) {
      message.submessage = {
        $case: "pseudoterminalExecuteRequest",
        pseudoterminalExecuteRequest: PseudoterminalExecuteRequest.fromJSON(
          object.pseudoterminalExecuteRequest
        ),
      };
    }
    if (
      object.pseudoterminalWriteRequest !== undefined &&
      object.pseudoterminalWriteRequest !== null
    ) {
      message.submessage = {
        $case: "pseudoterminalWriteRequest",
        pseudoterminalWriteRequest: PseudoterminalWriteRequest.fromJSON(
          object.pseudoterminalWriteRequest
        ),
      };
    }
    if (
      object.readFileRequest !== undefined &&
      object.readFileRequest !== null
    ) {
      message.submessage = {
        $case: "readFileRequest",
        readFileRequest: ReadFileRequest.fromJSON(object.readFileRequest),
      };
    }
    if (
      object.writeFileRequest !== undefined &&
      object.writeFileRequest !== null
    ) {
      message.submessage = {
        $case: "writeFileRequest",
        writeFileRequest: WriteFileRequest.fromJSON(object.writeFileRequest),
      };
    }
    if (
      object.contentsOfDirectoryRequest !== undefined &&
      object.contentsOfDirectoryRequest !== null
    ) {
      message.submessage = {
        $case: "contentsOfDirectoryRequest",
        contentsOfDirectoryRequest: ContentsOfDirectoryRequest.fromJSON(
          object.contentsOfDirectoryRequest
        ),
      };
    }
    if (
      object.notificationRequest !== undefined &&
      object.notificationRequest !== null
    ) {
      message.submessage = {
        $case: "notificationRequest",
        notificationRequest: NotificationRequest.fromJSON(
          object.notificationRequest
        ),
      };
    }
    if (
      object.getSettingsPropertyRequest !== undefined &&
      object.getSettingsPropertyRequest !== null
    ) {
      message.submessage = {
        $case: "getSettingsPropertyRequest",
        getSettingsPropertyRequest: GetSettingsPropertyRequest.fromJSON(
          object.getSettingsPropertyRequest
        ),
      };
    }
    if (
      object.updateSettingsPropertyRequest !== undefined &&
      object.updateSettingsPropertyRequest !== null
    ) {
      message.submessage = {
        $case: "updateSettingsPropertyRequest",
        updateSettingsPropertyRequest: UpdateSettingsPropertyRequest.fromJSON(
          object.updateSettingsPropertyRequest
        ),
      };
    }
    if (
      object.insertTextRequest !== undefined &&
      object.insertTextRequest !== null
    ) {
      message.submessage = {
        $case: "insertTextRequest",
        insertTextRequest: InsertTextRequest.fromJSON(object.insertTextRequest),
      };
    }
    if (
      object.updateApplicationPropertiesRequest !== undefined &&
      object.updateApplicationPropertiesRequest !== null
    ) {
      message.submessage = {
        $case: "updateApplicationPropertiesRequest",
        updateApplicationPropertiesRequest:
          UpdateApplicationPropertiesRequest.fromJSON(
            object.updateApplicationPropertiesRequest
          ),
      };
    }
    if (
      object.destinationOfSymbolicLinkRequest !== undefined &&
      object.destinationOfSymbolicLinkRequest !== null
    ) {
      message.submessage = {
        $case: "destinationOfSymbolicLinkRequest",
        destinationOfSymbolicLinkRequest:
          DestinationOfSymbolicLinkRequest.fromJSON(
            object.destinationOfSymbolicLinkRequest
          ),
      };
    }
    return message;
  },

  toJSON(message: ClientOriginatedMessage): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.submessage?.$case === "positionWindowRequest" &&
      (obj.positionWindowRequest = message.submessage?.positionWindowRequest
        ? PositionWindowRequest.toJSON(
            message.submessage?.positionWindowRequest
          )
        : undefined);
    message.submessage?.$case === "pseudoterminalExecuteRequest" &&
      (obj.pseudoterminalExecuteRequest = message.submessage
        ?.pseudoterminalExecuteRequest
        ? PseudoterminalExecuteRequest.toJSON(
            message.submessage?.pseudoterminalExecuteRequest
          )
        : undefined);
    message.submessage?.$case === "pseudoterminalWriteRequest" &&
      (obj.pseudoterminalWriteRequest = message.submessage
        ?.pseudoterminalWriteRequest
        ? PseudoterminalWriteRequest.toJSON(
            message.submessage?.pseudoterminalWriteRequest
          )
        : undefined);
    message.submessage?.$case === "readFileRequest" &&
      (obj.readFileRequest = message.submessage?.readFileRequest
        ? ReadFileRequest.toJSON(message.submessage?.readFileRequest)
        : undefined);
    message.submessage?.$case === "writeFileRequest" &&
      (obj.writeFileRequest = message.submessage?.writeFileRequest
        ? WriteFileRequest.toJSON(message.submessage?.writeFileRequest)
        : undefined);
    message.submessage?.$case === "contentsOfDirectoryRequest" &&
      (obj.contentsOfDirectoryRequest = message.submessage
        ?.contentsOfDirectoryRequest
        ? ContentsOfDirectoryRequest.toJSON(
            message.submessage?.contentsOfDirectoryRequest
          )
        : undefined);
    message.submessage?.$case === "notificationRequest" &&
      (obj.notificationRequest = message.submessage?.notificationRequest
        ? NotificationRequest.toJSON(message.submessage?.notificationRequest)
        : undefined);
    message.submessage?.$case === "getSettingsPropertyRequest" &&
      (obj.getSettingsPropertyRequest = message.submessage
        ?.getSettingsPropertyRequest
        ? GetSettingsPropertyRequest.toJSON(
            message.submessage?.getSettingsPropertyRequest
          )
        : undefined);
    message.submessage?.$case === "updateSettingsPropertyRequest" &&
      (obj.updateSettingsPropertyRequest = message.submessage
        ?.updateSettingsPropertyRequest
        ? UpdateSettingsPropertyRequest.toJSON(
            message.submessage?.updateSettingsPropertyRequest
          )
        : undefined);
    message.submessage?.$case === "insertTextRequest" &&
      (obj.insertTextRequest = message.submessage?.insertTextRequest
        ? InsertTextRequest.toJSON(message.submessage?.insertTextRequest)
        : undefined);
    message.submessage?.$case === "updateApplicationPropertiesRequest" &&
      (obj.updateApplicationPropertiesRequest = message.submessage
        ?.updateApplicationPropertiesRequest
        ? UpdateApplicationPropertiesRequest.toJSON(
            message.submessage?.updateApplicationPropertiesRequest
          )
        : undefined);
    message.submessage?.$case === "destinationOfSymbolicLinkRequest" &&
      (obj.destinationOfSymbolicLinkRequest = message.submessage
        ?.destinationOfSymbolicLinkRequest
        ? DestinationOfSymbolicLinkRequest.toJSON(
            message.submessage?.destinationOfSymbolicLinkRequest
          )
        : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ClientOriginatedMessage>
  ): ClientOriginatedMessage {
    const message = {
      ...baseClientOriginatedMessage,
    } as ClientOriginatedMessage;
    if (object.id !== undefined && object.id !== null) {
      message.id = object.id;
    }
    if (
      object.submessage?.$case === "positionWindowRequest" &&
      object.submessage?.positionWindowRequest !== undefined &&
      object.submessage?.positionWindowRequest !== null
    ) {
      message.submessage = {
        $case: "positionWindowRequest",
        positionWindowRequest: PositionWindowRequest.fromPartial(
          object.submessage.positionWindowRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "pseudoterminalExecuteRequest" &&
      object.submessage?.pseudoterminalExecuteRequest !== undefined &&
      object.submessage?.pseudoterminalExecuteRequest !== null
    ) {
      message.submessage = {
        $case: "pseudoterminalExecuteRequest",
        pseudoterminalExecuteRequest: PseudoterminalExecuteRequest.fromPartial(
          object.submessage.pseudoterminalExecuteRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "pseudoterminalWriteRequest" &&
      object.submessage?.pseudoterminalWriteRequest !== undefined &&
      object.submessage?.pseudoterminalWriteRequest !== null
    ) {
      message.submessage = {
        $case: "pseudoterminalWriteRequest",
        pseudoterminalWriteRequest: PseudoterminalWriteRequest.fromPartial(
          object.submessage.pseudoterminalWriteRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "readFileRequest" &&
      object.submessage?.readFileRequest !== undefined &&
      object.submessage?.readFileRequest !== null
    ) {
      message.submessage = {
        $case: "readFileRequest",
        readFileRequest: ReadFileRequest.fromPartial(
          object.submessage.readFileRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "writeFileRequest" &&
      object.submessage?.writeFileRequest !== undefined &&
      object.submessage?.writeFileRequest !== null
    ) {
      message.submessage = {
        $case: "writeFileRequest",
        writeFileRequest: WriteFileRequest.fromPartial(
          object.submessage.writeFileRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "contentsOfDirectoryRequest" &&
      object.submessage?.contentsOfDirectoryRequest !== undefined &&
      object.submessage?.contentsOfDirectoryRequest !== null
    ) {
      message.submessage = {
        $case: "contentsOfDirectoryRequest",
        contentsOfDirectoryRequest: ContentsOfDirectoryRequest.fromPartial(
          object.submessage.contentsOfDirectoryRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "notificationRequest" &&
      object.submessage?.notificationRequest !== undefined &&
      object.submessage?.notificationRequest !== null
    ) {
      message.submessage = {
        $case: "notificationRequest",
        notificationRequest: NotificationRequest.fromPartial(
          object.submessage.notificationRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "getSettingsPropertyRequest" &&
      object.submessage?.getSettingsPropertyRequest !== undefined &&
      object.submessage?.getSettingsPropertyRequest !== null
    ) {
      message.submessage = {
        $case: "getSettingsPropertyRequest",
        getSettingsPropertyRequest: GetSettingsPropertyRequest.fromPartial(
          object.submessage.getSettingsPropertyRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "updateSettingsPropertyRequest" &&
      object.submessage?.updateSettingsPropertyRequest !== undefined &&
      object.submessage?.updateSettingsPropertyRequest !== null
    ) {
      message.submessage = {
        $case: "updateSettingsPropertyRequest",
        updateSettingsPropertyRequest:
          UpdateSettingsPropertyRequest.fromPartial(
            object.submessage.updateSettingsPropertyRequest
          ),
      };
    }
    if (
      object.submessage?.$case === "insertTextRequest" &&
      object.submessage?.insertTextRequest !== undefined &&
      object.submessage?.insertTextRequest !== null
    ) {
      message.submessage = {
        $case: "insertTextRequest",
        insertTextRequest: InsertTextRequest.fromPartial(
          object.submessage.insertTextRequest
        ),
      };
    }
    if (
      object.submessage?.$case === "updateApplicationPropertiesRequest" &&
      object.submessage?.updateApplicationPropertiesRequest !== undefined &&
      object.submessage?.updateApplicationPropertiesRequest !== null
    ) {
      message.submessage = {
        $case: "updateApplicationPropertiesRequest",
        updateApplicationPropertiesRequest:
          UpdateApplicationPropertiesRequest.fromPartial(
            object.submessage.updateApplicationPropertiesRequest
          ),
      };
    }
    if (
      object.submessage?.$case === "destinationOfSymbolicLinkRequest" &&
      object.submessage?.destinationOfSymbolicLinkRequest !== undefined &&
      object.submessage?.destinationOfSymbolicLinkRequest !== null
    ) {
      message.submessage = {
        $case: "destinationOfSymbolicLinkRequest",
        destinationOfSymbolicLinkRequest:
          DestinationOfSymbolicLinkRequest.fromPartial(
            object.submessage.destinationOfSymbolicLinkRequest
          ),
      };
    }
    return message;
  },
};

const baseServerOriginatedMessage: object = {};

export const ServerOriginatedMessage = {
  encode(
    message: ServerOriginatedMessage,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== undefined) {
      writer.uint32(8).int64(message.id);
    }
    if (message.submessage?.$case === "error") {
      writer.uint32(18).string(message.submessage.error);
    }
    if (message.submessage?.$case === "success") {
      writer.uint32(24).bool(message.submessage.success);
    }
    if (message.submessage?.$case === "positionWindowResponse") {
      PositionWindowResponse.encode(
        message.submessage.positionWindowResponse,
        writer.uint32(802).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "pseudoterminalExecuteResponse") {
      PseudoterminalExecuteResponse.encode(
        message.submessage.pseudoterminalExecuteResponse,
        writer.uint32(810).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "readFileResponse") {
      ReadFileResponse.encode(
        message.submessage.readFileResponse,
        writer.uint32(818).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "contentsOfDirectoryResponse") {
      ContentsOfDirectoryResponse.encode(
        message.submessage.contentsOfDirectoryResponse,
        writer.uint32(826).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "getSettingsPropertyResponse") {
      GetSettingsPropertyResponse.encode(
        message.submessage.getSettingsPropertyResponse,
        writer.uint32(834).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "destinationOfSymbolicLinkResponse") {
      DestinationOfSymbolicLinkResponse.encode(
        message.submessage.destinationOfSymbolicLinkResponse,
        writer.uint32(842).fork()
      ).ldelim();
    }
    if (message.submessage?.$case === "notification") {
      Notification.encode(
        message.submessage.notification,
        writer.uint32(8002).fork()
      ).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ServerOriginatedMessage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseServerOriginatedMessage,
    } as ServerOriginatedMessage;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = longToNumber(reader.int64() as Long);
          break;
        case 2:
          message.submessage = { $case: "error", error: reader.string() };
          break;
        case 3:
          message.submessage = { $case: "success", success: reader.bool() };
          break;
        case 100:
          message.submessage = {
            $case: "positionWindowResponse",
            positionWindowResponse: PositionWindowResponse.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 101:
          message.submessage = {
            $case: "pseudoterminalExecuteResponse",
            pseudoterminalExecuteResponse: PseudoterminalExecuteResponse.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 102:
          message.submessage = {
            $case: "readFileResponse",
            readFileResponse: ReadFileResponse.decode(reader, reader.uint32()),
          };
          break;
        case 103:
          message.submessage = {
            $case: "contentsOfDirectoryResponse",
            contentsOfDirectoryResponse: ContentsOfDirectoryResponse.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 104:
          message.submessage = {
            $case: "getSettingsPropertyResponse",
            getSettingsPropertyResponse: GetSettingsPropertyResponse.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 105:
          message.submessage = {
            $case: "destinationOfSymbolicLinkResponse",
            destinationOfSymbolicLinkResponse:
              DestinationOfSymbolicLinkResponse.decode(reader, reader.uint32()),
          };
          break;
        case 1000:
          message.submessage = {
            $case: "notification",
            notification: Notification.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ServerOriginatedMessage {
    const message = {
      ...baseServerOriginatedMessage,
    } as ServerOriginatedMessage;
    if (object.id !== undefined && object.id !== null) {
      message.id = Number(object.id);
    }
    if (object.error !== undefined && object.error !== null) {
      message.submessage = { $case: "error", error: String(object.error) };
    }
    if (object.success !== undefined && object.success !== null) {
      message.submessage = {
        $case: "success",
        success: Boolean(object.success),
      };
    }
    if (
      object.positionWindowResponse !== undefined &&
      object.positionWindowResponse !== null
    ) {
      message.submessage = {
        $case: "positionWindowResponse",
        positionWindowResponse: PositionWindowResponse.fromJSON(
          object.positionWindowResponse
        ),
      };
    }
    if (
      object.pseudoterminalExecuteResponse !== undefined &&
      object.pseudoterminalExecuteResponse !== null
    ) {
      message.submessage = {
        $case: "pseudoterminalExecuteResponse",
        pseudoterminalExecuteResponse: PseudoterminalExecuteResponse.fromJSON(
          object.pseudoterminalExecuteResponse
        ),
      };
    }
    if (
      object.readFileResponse !== undefined &&
      object.readFileResponse !== null
    ) {
      message.submessage = {
        $case: "readFileResponse",
        readFileResponse: ReadFileResponse.fromJSON(object.readFileResponse),
      };
    }
    if (
      object.contentsOfDirectoryResponse !== undefined &&
      object.contentsOfDirectoryResponse !== null
    ) {
      message.submessage = {
        $case: "contentsOfDirectoryResponse",
        contentsOfDirectoryResponse: ContentsOfDirectoryResponse.fromJSON(
          object.contentsOfDirectoryResponse
        ),
      };
    }
    if (
      object.getSettingsPropertyResponse !== undefined &&
      object.getSettingsPropertyResponse !== null
    ) {
      message.submessage = {
        $case: "getSettingsPropertyResponse",
        getSettingsPropertyResponse: GetSettingsPropertyResponse.fromJSON(
          object.getSettingsPropertyResponse
        ),
      };
    }
    if (
      object.destinationOfSymbolicLinkResponse !== undefined &&
      object.destinationOfSymbolicLinkResponse !== null
    ) {
      message.submessage = {
        $case: "destinationOfSymbolicLinkResponse",
        destinationOfSymbolicLinkResponse:
          DestinationOfSymbolicLinkResponse.fromJSON(
            object.destinationOfSymbolicLinkResponse
          ),
      };
    }
    if (object.notification !== undefined && object.notification !== null) {
      message.submessage = {
        $case: "notification",
        notification: Notification.fromJSON(object.notification),
      };
    }
    return message;
  },

  toJSON(message: ServerOriginatedMessage): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.submessage?.$case === "error" &&
      (obj.error = message.submessage?.error);
    message.submessage?.$case === "success" &&
      (obj.success = message.submessage?.success);
    message.submessage?.$case === "positionWindowResponse" &&
      (obj.positionWindowResponse = message.submessage?.positionWindowResponse
        ? PositionWindowResponse.toJSON(
            message.submessage?.positionWindowResponse
          )
        : undefined);
    message.submessage?.$case === "pseudoterminalExecuteResponse" &&
      (obj.pseudoterminalExecuteResponse = message.submessage
        ?.pseudoterminalExecuteResponse
        ? PseudoterminalExecuteResponse.toJSON(
            message.submessage?.pseudoterminalExecuteResponse
          )
        : undefined);
    message.submessage?.$case === "readFileResponse" &&
      (obj.readFileResponse = message.submessage?.readFileResponse
        ? ReadFileResponse.toJSON(message.submessage?.readFileResponse)
        : undefined);
    message.submessage?.$case === "contentsOfDirectoryResponse" &&
      (obj.contentsOfDirectoryResponse = message.submessage
        ?.contentsOfDirectoryResponse
        ? ContentsOfDirectoryResponse.toJSON(
            message.submessage?.contentsOfDirectoryResponse
          )
        : undefined);
    message.submessage?.$case === "getSettingsPropertyResponse" &&
      (obj.getSettingsPropertyResponse = message.submessage
        ?.getSettingsPropertyResponse
        ? GetSettingsPropertyResponse.toJSON(
            message.submessage?.getSettingsPropertyResponse
          )
        : undefined);
    message.submessage?.$case === "destinationOfSymbolicLinkResponse" &&
      (obj.destinationOfSymbolicLinkResponse = message.submessage
        ?.destinationOfSymbolicLinkResponse
        ? DestinationOfSymbolicLinkResponse.toJSON(
            message.submessage?.destinationOfSymbolicLinkResponse
          )
        : undefined);
    message.submessage?.$case === "notification" &&
      (obj.notification = message.submessage?.notification
        ? Notification.toJSON(message.submessage?.notification)
        : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ServerOriginatedMessage>
  ): ServerOriginatedMessage {
    const message = {
      ...baseServerOriginatedMessage,
    } as ServerOriginatedMessage;
    if (object.id !== undefined && object.id !== null) {
      message.id = object.id;
    }
    if (
      object.submessage?.$case === "error" &&
      object.submessage?.error !== undefined &&
      object.submessage?.error !== null
    ) {
      message.submessage = { $case: "error", error: object.submessage.error };
    }
    if (
      object.submessage?.$case === "success" &&
      object.submessage?.success !== undefined &&
      object.submessage?.success !== null
    ) {
      message.submessage = {
        $case: "success",
        success: object.submessage.success,
      };
    }
    if (
      object.submessage?.$case === "positionWindowResponse" &&
      object.submessage?.positionWindowResponse !== undefined &&
      object.submessage?.positionWindowResponse !== null
    ) {
      message.submessage = {
        $case: "positionWindowResponse",
        positionWindowResponse: PositionWindowResponse.fromPartial(
          object.submessage.positionWindowResponse
        ),
      };
    }
    if (
      object.submessage?.$case === "pseudoterminalExecuteResponse" &&
      object.submessage?.pseudoterminalExecuteResponse !== undefined &&
      object.submessage?.pseudoterminalExecuteResponse !== null
    ) {
      message.submessage = {
        $case: "pseudoterminalExecuteResponse",
        pseudoterminalExecuteResponse:
          PseudoterminalExecuteResponse.fromPartial(
            object.submessage.pseudoterminalExecuteResponse
          ),
      };
    }
    if (
      object.submessage?.$case === "readFileResponse" &&
      object.submessage?.readFileResponse !== undefined &&
      object.submessage?.readFileResponse !== null
    ) {
      message.submessage = {
        $case: "readFileResponse",
        readFileResponse: ReadFileResponse.fromPartial(
          object.submessage.readFileResponse
        ),
      };
    }
    if (
      object.submessage?.$case === "contentsOfDirectoryResponse" &&
      object.submessage?.contentsOfDirectoryResponse !== undefined &&
      object.submessage?.contentsOfDirectoryResponse !== null
    ) {
      message.submessage = {
        $case: "contentsOfDirectoryResponse",
        contentsOfDirectoryResponse: ContentsOfDirectoryResponse.fromPartial(
          object.submessage.contentsOfDirectoryResponse
        ),
      };
    }
    if (
      object.submessage?.$case === "getSettingsPropertyResponse" &&
      object.submessage?.getSettingsPropertyResponse !== undefined &&
      object.submessage?.getSettingsPropertyResponse !== null
    ) {
      message.submessage = {
        $case: "getSettingsPropertyResponse",
        getSettingsPropertyResponse: GetSettingsPropertyResponse.fromPartial(
          object.submessage.getSettingsPropertyResponse
        ),
      };
    }
    if (
      object.submessage?.$case === "destinationOfSymbolicLinkResponse" &&
      object.submessage?.destinationOfSymbolicLinkResponse !== undefined &&
      object.submessage?.destinationOfSymbolicLinkResponse !== null
    ) {
      message.submessage = {
        $case: "destinationOfSymbolicLinkResponse",
        destinationOfSymbolicLinkResponse:
          DestinationOfSymbolicLinkResponse.fromPartial(
            object.submessage.destinationOfSymbolicLinkResponse
          ),
      };
    }
    if (
      object.submessage?.$case === "notification" &&
      object.submessage?.notification !== undefined &&
      object.submessage?.notification !== null
    ) {
      message.submessage = {
        $case: "notification",
        notification: Notification.fromPartial(object.submessage.notification),
      };
    }
    return message;
  },
};

const basePoint: object = { x: 0, y: 0 };

export const Point = {
  encode(message: Point, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.x !== 0) {
      writer.uint32(13).float(message.x);
    }
    if (message.y !== 0) {
      writer.uint32(21).float(message.y);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Point {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...basePoint } as Point;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.x = reader.float();
          break;
        case 2:
          message.y = reader.float();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Point {
    const message = { ...basePoint } as Point;
    if (object.x !== undefined && object.x !== null) {
      message.x = Number(object.x);
    }
    if (object.y !== undefined && object.y !== null) {
      message.y = Number(object.y);
    }
    return message;
  },

  toJSON(message: Point): unknown {
    const obj: any = {};
    message.x !== undefined && (obj.x = message.x);
    message.y !== undefined && (obj.y = message.y);
    return obj;
  },

  fromPartial(object: DeepPartial<Point>): Point {
    const message = { ...basePoint } as Point;
    if (object.x !== undefined && object.x !== null) {
      message.x = object.x;
    }
    if (object.y !== undefined && object.y !== null) {
      message.y = object.y;
    }
    return message;
  },
};

const baseSize: object = { width: 0, height: 0 };

export const Size = {
  encode(message: Size, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.width !== 0) {
      writer.uint32(13).float(message.width);
    }
    if (message.height !== 0) {
      writer.uint32(21).float(message.height);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Size {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseSize } as Size;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.width = reader.float();
          break;
        case 2:
          message.height = reader.float();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Size {
    const message = { ...baseSize } as Size;
    if (object.width !== undefined && object.width !== null) {
      message.width = Number(object.width);
    }
    if (object.height !== undefined && object.height !== null) {
      message.height = Number(object.height);
    }
    return message;
  },

  toJSON(message: Size): unknown {
    const obj: any = {};
    message.width !== undefined && (obj.width = message.width);
    message.height !== undefined && (obj.height = message.height);
    return obj;
  },

  fromPartial(object: DeepPartial<Size>): Size {
    const message = { ...baseSize } as Size;
    if (object.width !== undefined && object.width !== null) {
      message.width = object.width;
    }
    if (object.height !== undefined && object.height !== null) {
      message.height = object.height;
    }
    return message;
  },
};

const baseFrame: object = {};

export const Frame = {
  encode(message: Frame, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.origin !== undefined) {
      Point.encode(message.origin, writer.uint32(10).fork()).ldelim();
    }
    if (message.size !== undefined) {
      Size.encode(message.size, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Frame {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseFrame } as Frame;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.origin = Point.decode(reader, reader.uint32());
          break;
        case 2:
          message.size = Size.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Frame {
    const message = { ...baseFrame } as Frame;
    if (object.origin !== undefined && object.origin !== null) {
      message.origin = Point.fromJSON(object.origin);
    }
    if (object.size !== undefined && object.size !== null) {
      message.size = Size.fromJSON(object.size);
    }
    return message;
  },

  toJSON(message: Frame): unknown {
    const obj: any = {};
    message.origin !== undefined &&
      (obj.origin = message.origin ? Point.toJSON(message.origin) : undefined);
    message.size !== undefined &&
      (obj.size = message.size ? Size.toJSON(message.size) : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<Frame>): Frame {
    const message = { ...baseFrame } as Frame;
    if (object.origin !== undefined && object.origin !== null) {
      message.origin = Point.fromPartial(object.origin);
    }
    if (object.size !== undefined && object.size !== null) {
      message.size = Size.fromPartial(object.size);
    }
    return message;
  },
};

const baseEnvironmentVariable: object = { key: "" };

export const EnvironmentVariable = {
  encode(
    message: EnvironmentVariable,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.key !== "") {
      writer.uint32(10).string(message.key);
    }
    if (message.value !== undefined) {
      writer.uint32(18).string(message.value);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): EnvironmentVariable {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseEnvironmentVariable } as EnvironmentVariable;
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

  fromJSON(object: any): EnvironmentVariable {
    const message = { ...baseEnvironmentVariable } as EnvironmentVariable;
    if (object.key !== undefined && object.key !== null) {
      message.key = String(object.key);
    }
    if (object.value !== undefined && object.value !== null) {
      message.value = String(object.value);
    }
    return message;
  },

  toJSON(message: EnvironmentVariable): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = message.key);
    message.value !== undefined && (obj.value = message.value);
    return obj;
  },

  fromPartial(object: DeepPartial<EnvironmentVariable>): EnvironmentVariable {
    const message = { ...baseEnvironmentVariable } as EnvironmentVariable;
    if (object.key !== undefined && object.key !== null) {
      message.key = object.key;
    }
    if (object.value !== undefined && object.value !== null) {
      message.value = object.value;
    }
    return message;
  },
};

const baseProcess: object = {};

export const Process = {
  encode(
    message: Process,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.pid !== undefined) {
      writer.uint32(8).int32(message.pid);
    }
    if (message.executable !== undefined) {
      writer.uint32(18).string(message.executable);
    }
    if (message.directory !== undefined) {
      writer.uint32(26).string(message.directory);
    }
    for (const v of message.env) {
      EnvironmentVariable.encode(v!, writer.uint32(34).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Process {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseProcess } as Process;
    message.env = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.pid = reader.int32();
          break;
        case 2:
          message.executable = reader.string();
          break;
        case 3:
          message.directory = reader.string();
          break;
        case 4:
          message.env.push(EnvironmentVariable.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Process {
    const message = { ...baseProcess } as Process;
    message.env = [];
    if (object.pid !== undefined && object.pid !== null) {
      message.pid = Number(object.pid);
    }
    if (object.executable !== undefined && object.executable !== null) {
      message.executable = String(object.executable);
    }
    if (object.directory !== undefined && object.directory !== null) {
      message.directory = String(object.directory);
    }
    if (object.env !== undefined && object.env !== null) {
      for (const e of object.env) {
        message.env.push(EnvironmentVariable.fromJSON(e));
      }
    }
    return message;
  },

  toJSON(message: Process): unknown {
    const obj: any = {};
    message.pid !== undefined && (obj.pid = message.pid);
    message.executable !== undefined && (obj.executable = message.executable);
    message.directory !== undefined && (obj.directory = message.directory);
    if (message.env) {
      obj.env = message.env.map((e) =>
        e ? EnvironmentVariable.toJSON(e) : undefined
      );
    } else {
      obj.env = [];
    }
    return obj;
  },

  fromPartial(object: DeepPartial<Process>): Process {
    const message = { ...baseProcess } as Process;
    message.env = [];
    if (object.pid !== undefined && object.pid !== null) {
      message.pid = object.pid;
    }
    if (object.executable !== undefined && object.executable !== null) {
      message.executable = object.executable;
    }
    if (object.directory !== undefined && object.directory !== null) {
      message.directory = object.directory;
    }
    if (object.env !== undefined && object.env !== null) {
      for (const e of object.env) {
        message.env.push(EnvironmentVariable.fromPartial(e));
      }
    }
    return message;
  },
};

const baseFilePath: object = {};

export const FilePath = {
  encode(
    message: FilePath,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.path !== undefined) {
      writer.uint32(10).string(message.path);
    }
    if (message.relativeTo !== undefined) {
      writer.uint32(18).string(message.relativeTo);
    }
    if (message.expandTildeInPath !== undefined) {
      writer.uint32(24).bool(message.expandTildeInPath);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FilePath {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseFilePath } as FilePath;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.path = reader.string();
          break;
        case 2:
          message.relativeTo = reader.string();
          break;
        case 3:
          message.expandTildeInPath = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FilePath {
    const message = { ...baseFilePath } as FilePath;
    if (object.path !== undefined && object.path !== null) {
      message.path = String(object.path);
    }
    if (object.relativeTo !== undefined && object.relativeTo !== null) {
      message.relativeTo = String(object.relativeTo);
    }
    if (
      object.expandTildeInPath !== undefined &&
      object.expandTildeInPath !== null
    ) {
      message.expandTildeInPath = Boolean(object.expandTildeInPath);
    }
    return message;
  },

  toJSON(message: FilePath): unknown {
    const obj: any = {};
    message.path !== undefined && (obj.path = message.path);
    message.relativeTo !== undefined && (obj.relativeTo = message.relativeTo);
    message.expandTildeInPath !== undefined &&
      (obj.expandTildeInPath = message.expandTildeInPath);
    return obj;
  },

  fromPartial(object: DeepPartial<FilePath>): FilePath {
    const message = { ...baseFilePath } as FilePath;
    if (object.path !== undefined && object.path !== null) {
      message.path = object.path;
    }
    if (object.relativeTo !== undefined && object.relativeTo !== null) {
      message.relativeTo = object.relativeTo;
    }
    if (
      object.expandTildeInPath !== undefined &&
      object.expandTildeInPath !== null
    ) {
      message.expandTildeInPath = object.expandTildeInPath;
    }
    return message;
  },
};

const baseKeyEvent: object = { modifiers: 0 };

export const KeyEvent = {
  encode(
    message: KeyEvent,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.appleKeyCode !== undefined) {
      writer.uint32(8).int32(message.appleKeyCode);
    }
    if (message.characters !== undefined) {
      writer.uint32(18).string(message.characters);
    }
    if (message.charactersIgnoringModifiers !== undefined) {
      writer.uint32(26).string(message.charactersIgnoringModifiers);
    }
    writer.uint32(34).fork();
    for (const v of message.modifiers) {
      writer.int32(v);
    }
    writer.ldelim();
    if (message.isRepeat !== undefined) {
      writer.uint32(40).bool(message.isRepeat);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): KeyEvent {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseKeyEvent } as KeyEvent;
    message.modifiers = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.appleKeyCode = reader.int32();
          break;
        case 2:
          message.characters = reader.string();
          break;
        case 3:
          message.charactersIgnoringModifiers = reader.string();
          break;
        case 4:
          if ((tag & 7) === 2) {
            const end2 = reader.uint32() + reader.pos;
            while (reader.pos < end2) {
              message.modifiers.push(reader.int32() as any);
            }
          } else {
            message.modifiers.push(reader.int32() as any);
          }
          break;
        case 5:
          message.isRepeat = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): KeyEvent {
    const message = { ...baseKeyEvent } as KeyEvent;
    message.modifiers = [];
    if (object.appleKeyCode !== undefined && object.appleKeyCode !== null) {
      message.appleKeyCode = Number(object.appleKeyCode);
    }
    if (object.characters !== undefined && object.characters !== null) {
      message.characters = String(object.characters);
    }
    if (
      object.charactersIgnoringModifiers !== undefined &&
      object.charactersIgnoringModifiers !== null
    ) {
      message.charactersIgnoringModifiers = String(
        object.charactersIgnoringModifiers
      );
    }
    if (object.modifiers !== undefined && object.modifiers !== null) {
      for (const e of object.modifiers) {
        message.modifiers.push(modifiersFromJSON(e));
      }
    }
    if (object.isRepeat !== undefined && object.isRepeat !== null) {
      message.isRepeat = Boolean(object.isRepeat);
    }
    return message;
  },

  toJSON(message: KeyEvent): unknown {
    const obj: any = {};
    message.appleKeyCode !== undefined &&
      (obj.appleKeyCode = message.appleKeyCode);
    message.characters !== undefined && (obj.characters = message.characters);
    message.charactersIgnoringModifiers !== undefined &&
      (obj.charactersIgnoringModifiers = message.charactersIgnoringModifiers);
    if (message.modifiers) {
      obj.modifiers = message.modifiers.map((e) => modifiersToJSON(e));
    } else {
      obj.modifiers = [];
    }
    message.isRepeat !== undefined && (obj.isRepeat = message.isRepeat);
    return obj;
  },

  fromPartial(object: DeepPartial<KeyEvent>): KeyEvent {
    const message = { ...baseKeyEvent } as KeyEvent;
    message.modifiers = [];
    if (object.appleKeyCode !== undefined && object.appleKeyCode !== null) {
      message.appleKeyCode = object.appleKeyCode;
    }
    if (object.characters !== undefined && object.characters !== null) {
      message.characters = object.characters;
    }
    if (
      object.charactersIgnoringModifiers !== undefined &&
      object.charactersIgnoringModifiers !== null
    ) {
      message.charactersIgnoringModifiers = object.charactersIgnoringModifiers;
    }
    if (object.modifiers !== undefined && object.modifiers !== null) {
      for (const e of object.modifiers) {
        message.modifiers.push(e);
      }
    }
    if (object.isRepeat !== undefined && object.isRepeat !== null) {
      message.isRepeat = object.isRepeat;
    }
    return message;
  },
};

const baseScreen: object = {};

export const Screen = {
  encode(
    message: Screen,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.frame !== undefined) {
      Frame.encode(message.frame, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Screen {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseScreen } as Screen;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.frame = Frame.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Screen {
    const message = { ...baseScreen } as Screen;
    if (object.frame !== undefined && object.frame !== null) {
      message.frame = Frame.fromJSON(object.frame);
    }
    return message;
  },

  toJSON(message: Screen): unknown {
    const obj: any = {};
    message.frame !== undefined &&
      (obj.frame = message.frame ? Frame.toJSON(message.frame) : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<Screen>): Screen {
    const message = { ...baseScreen } as Screen;
    if (object.frame !== undefined && object.frame !== null) {
      message.frame = Frame.fromPartial(object.frame);
    }
    return message;
  },
};

const baseSession: object = {};

export const Session = {
  encode(
    message: Session,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.sessionId !== undefined) {
      writer.uint32(10).string(message.sessionId);
    }
    if (message.frontmostProcess !== undefined) {
      Process.encode(
        message.frontmostProcess,
        writer.uint32(18).fork()
      ).ldelim();
    }
    for (const v of message.env) {
      EnvironmentVariable.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Session {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseSession } as Session;
    message.env = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.sessionId = reader.string();
          break;
        case 2:
          message.frontmostProcess = Process.decode(reader, reader.uint32());
          break;
        case 3:
          message.env.push(EnvironmentVariable.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Session {
    const message = { ...baseSession } as Session;
    message.env = [];
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = String(object.sessionId);
    }
    if (
      object.frontmostProcess !== undefined &&
      object.frontmostProcess !== null
    ) {
      message.frontmostProcess = Process.fromJSON(object.frontmostProcess);
    }
    if (object.env !== undefined && object.env !== null) {
      for (const e of object.env) {
        message.env.push(EnvironmentVariable.fromJSON(e));
      }
    }
    return message;
  },

  toJSON(message: Session): unknown {
    const obj: any = {};
    message.sessionId !== undefined && (obj.sessionId = message.sessionId);
    message.frontmostProcess !== undefined &&
      (obj.frontmostProcess = message.frontmostProcess
        ? Process.toJSON(message.frontmostProcess)
        : undefined);
    if (message.env) {
      obj.env = message.env.map((e) =>
        e ? EnvironmentVariable.toJSON(e) : undefined
      );
    } else {
      obj.env = [];
    }
    return obj;
  },

  fromPartial(object: DeepPartial<Session>): Session {
    const message = { ...baseSession } as Session;
    message.env = [];
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = object.sessionId;
    }
    if (
      object.frontmostProcess !== undefined &&
      object.frontmostProcess !== null
    ) {
      message.frontmostProcess = Process.fromPartial(object.frontmostProcess);
    }
    if (object.env !== undefined && object.env !== null) {
      for (const e of object.env) {
        message.env.push(EnvironmentVariable.fromPartial(e));
      }
    }
    return message;
  },
};

const baseApplication: object = {};

export const Application = {
  encode(
    message: Application,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.bundleIdentifier !== undefined) {
      writer.uint32(10).string(message.bundleIdentifier);
    }
    if (message.name !== undefined) {
      writer.uint32(18).string(message.name);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Application {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseApplication } as Application;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.bundleIdentifier = reader.string();
          break;
        case 2:
          message.name = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Application {
    const message = { ...baseApplication } as Application;
    if (
      object.bundleIdentifier !== undefined &&
      object.bundleIdentifier !== null
    ) {
      message.bundleIdentifier = String(object.bundleIdentifier);
    }
    if (object.name !== undefined && object.name !== null) {
      message.name = String(object.name);
    }
    return message;
  },

  toJSON(message: Application): unknown {
    const obj: any = {};
    message.bundleIdentifier !== undefined &&
      (obj.bundleIdentifier = message.bundleIdentifier);
    message.name !== undefined && (obj.name = message.name);
    return obj;
  },

  fromPartial(object: DeepPartial<Application>): Application {
    const message = { ...baseApplication } as Application;
    if (
      object.bundleIdentifier !== undefined &&
      object.bundleIdentifier !== null
    ) {
      message.bundleIdentifier = object.bundleIdentifier;
    }
    if (object.name !== undefined && object.name !== null) {
      message.name = object.name;
    }
    return message;
  },
};

const baseWindow: object = {};

export const Window = {
  encode(
    message: Window,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.windowId !== undefined) {
      writer.uint32(10).string(message.windowId);
    }
    if (message.frame !== undefined) {
      Frame.encode(message.frame, writer.uint32(18).fork()).ldelim();
    }
    if (message.currentSession !== undefined) {
      Session.encode(message.currentSession, writer.uint32(26).fork()).ldelim();
    }
    if (message.app !== undefined) {
      Application.encode(message.app, writer.uint32(34).fork()).ldelim();
    }
    if (message.currentScreen !== undefined) {
      Screen.encode(message.currentScreen, writer.uint32(42).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Window {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseWindow } as Window;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.windowId = reader.string();
          break;
        case 2:
          message.frame = Frame.decode(reader, reader.uint32());
          break;
        case 3:
          message.currentSession = Session.decode(reader, reader.uint32());
          break;
        case 4:
          message.app = Application.decode(reader, reader.uint32());
          break;
        case 5:
          message.currentScreen = Screen.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Window {
    const message = { ...baseWindow } as Window;
    if (object.windowId !== undefined && object.windowId !== null) {
      message.windowId = String(object.windowId);
    }
    if (object.frame !== undefined && object.frame !== null) {
      message.frame = Frame.fromJSON(object.frame);
    }
    if (object.currentSession !== undefined && object.currentSession !== null) {
      message.currentSession = Session.fromJSON(object.currentSession);
    }
    if (object.app !== undefined && object.app !== null) {
      message.app = Application.fromJSON(object.app);
    }
    if (object.currentScreen !== undefined && object.currentScreen !== null) {
      message.currentScreen = Screen.fromJSON(object.currentScreen);
    }
    return message;
  },

  toJSON(message: Window): unknown {
    const obj: any = {};
    message.windowId !== undefined && (obj.windowId = message.windowId);
    message.frame !== undefined &&
      (obj.frame = message.frame ? Frame.toJSON(message.frame) : undefined);
    message.currentSession !== undefined &&
      (obj.currentSession = message.currentSession
        ? Session.toJSON(message.currentSession)
        : undefined);
    message.app !== undefined &&
      (obj.app = message.app ? Application.toJSON(message.app) : undefined);
    message.currentScreen !== undefined &&
      (obj.currentScreen = message.currentScreen
        ? Screen.toJSON(message.currentScreen)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<Window>): Window {
    const message = { ...baseWindow } as Window;
    if (object.windowId !== undefined && object.windowId !== null) {
      message.windowId = object.windowId;
    }
    if (object.frame !== undefined && object.frame !== null) {
      message.frame = Frame.fromPartial(object.frame);
    }
    if (object.currentSession !== undefined && object.currentSession !== null) {
      message.currentSession = Session.fromPartial(object.currentSession);
    }
    if (object.app !== undefined && object.app !== null) {
      message.app = Application.fromPartial(object.app);
    }
    if (object.currentScreen !== undefined && object.currentScreen !== null) {
      message.currentScreen = Screen.fromPartial(object.currentScreen);
    }
    return message;
  },
};

const baseTextUpdate: object = {};

export const TextUpdate = {
  encode(
    message: TextUpdate,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.insertion !== undefined) {
      writer.uint32(10).string(message.insertion);
    }
    if (message.deletion !== undefined) {
      writer.uint32(16).int64(message.deletion);
    }
    if (message.offset !== undefined) {
      writer.uint32(24).int64(message.offset);
    }
    if (message.immediate !== undefined) {
      writer.uint32(32).bool(message.immediate);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TextUpdate {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseTextUpdate } as TextUpdate;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.insertion = reader.string();
          break;
        case 2:
          message.deletion = longToNumber(reader.int64() as Long);
          break;
        case 3:
          message.offset = longToNumber(reader.int64() as Long);
          break;
        case 4:
          message.immediate = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TextUpdate {
    const message = { ...baseTextUpdate } as TextUpdate;
    if (object.insertion !== undefined && object.insertion !== null) {
      message.insertion = String(object.insertion);
    }
    if (object.deletion !== undefined && object.deletion !== null) {
      message.deletion = Number(object.deletion);
    }
    if (object.offset !== undefined && object.offset !== null) {
      message.offset = Number(object.offset);
    }
    if (object.immediate !== undefined && object.immediate !== null) {
      message.immediate = Boolean(object.immediate);
    }
    return message;
  },

  toJSON(message: TextUpdate): unknown {
    const obj: any = {};
    message.insertion !== undefined && (obj.insertion = message.insertion);
    message.deletion !== undefined && (obj.deletion = message.deletion);
    message.offset !== undefined && (obj.offset = message.offset);
    message.immediate !== undefined && (obj.immediate = message.immediate);
    return obj;
  },

  fromPartial(object: DeepPartial<TextUpdate>): TextUpdate {
    const message = { ...baseTextUpdate } as TextUpdate;
    if (object.insertion !== undefined && object.insertion !== null) {
      message.insertion = object.insertion;
    }
    if (object.deletion !== undefined && object.deletion !== null) {
      message.deletion = object.deletion;
    }
    if (object.offset !== undefined && object.offset !== null) {
      message.offset = object.offset;
    }
    if (object.immediate !== undefined && object.immediate !== null) {
      message.immediate = object.immediate;
    }
    return message;
  },
};

const baseInsertTextRequest: object = {};

export const InsertTextRequest = {
  encode(
    message: InsertTextRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.type?.$case === "text") {
      writer.uint32(10).string(message.type.text);
    }
    if (message.type?.$case === "update") {
      TextUpdate.encode(message.type.update, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): InsertTextRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertTextRequest } as InsertTextRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.type = { $case: "text", text: reader.string() };
          break;
        case 2:
          message.type = {
            $case: "update",
            update: TextUpdate.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InsertTextRequest {
    const message = { ...baseInsertTextRequest } as InsertTextRequest;
    if (object.text !== undefined && object.text !== null) {
      message.type = { $case: "text", text: String(object.text) };
    }
    if (object.update !== undefined && object.update !== null) {
      message.type = {
        $case: "update",
        update: TextUpdate.fromJSON(object.update),
      };
    }
    return message;
  },

  toJSON(message: InsertTextRequest): unknown {
    const obj: any = {};
    message.type?.$case === "text" && (obj.text = message.type?.text);
    message.type?.$case === "update" &&
      (obj.update = message.type?.update
        ? TextUpdate.toJSON(message.type?.update)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<InsertTextRequest>): InsertTextRequest {
    const message = { ...baseInsertTextRequest } as InsertTextRequest;
    if (
      object.type?.$case === "text" &&
      object.type?.text !== undefined &&
      object.type?.text !== null
    ) {
      message.type = { $case: "text", text: object.type.text };
    }
    if (
      object.type?.$case === "update" &&
      object.type?.update !== undefined &&
      object.type?.update !== null
    ) {
      message.type = {
        $case: "update",
        update: TextUpdate.fromPartial(object.type.update),
      };
    }
    return message;
  },
};

const basePseudoterminalWriteRequest: object = {};

export const PseudoterminalWriteRequest = {
  encode(
    message: PseudoterminalWriteRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.input?.$case === "text") {
      writer.uint32(10).string(message.input.text);
    }
    if (message.input?.$case === "octal") {
      writer.uint32(18).bytes(message.input.octal);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): PseudoterminalWriteRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...basePseudoterminalWriteRequest,
    } as PseudoterminalWriteRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.input = { $case: "text", text: reader.string() };
          break;
        case 2:
          message.input = { $case: "octal", octal: reader.bytes() };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PseudoterminalWriteRequest {
    const message = {
      ...basePseudoterminalWriteRequest,
    } as PseudoterminalWriteRequest;
    if (object.text !== undefined && object.text !== null) {
      message.input = { $case: "text", text: String(object.text) };
    }
    if (object.octal !== undefined && object.octal !== null) {
      message.input = { $case: "octal", octal: bytesFromBase64(object.octal) };
    }
    return message;
  },

  toJSON(message: PseudoterminalWriteRequest): unknown {
    const obj: any = {};
    message.input?.$case === "text" && (obj.text = message.input?.text);
    message.input?.$case === "octal" &&
      (obj.octal =
        message.input?.octal !== undefined
          ? base64FromBytes(message.input?.octal)
          : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<PseudoterminalWriteRequest>
  ): PseudoterminalWriteRequest {
    const message = {
      ...basePseudoterminalWriteRequest,
    } as PseudoterminalWriteRequest;
    if (
      object.input?.$case === "text" &&
      object.input?.text !== undefined &&
      object.input?.text !== null
    ) {
      message.input = { $case: "text", text: object.input.text };
    }
    if (
      object.input?.$case === "octal" &&
      object.input?.octal !== undefined &&
      object.input?.octal !== null
    ) {
      message.input = { $case: "octal", octal: object.input.octal };
    }
    return message;
  },
};

const basePseudoterminalExecuteRequest: object = { command: "" };

export const PseudoterminalExecuteRequest = {
  encode(
    message: PseudoterminalExecuteRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.command !== "") {
      writer.uint32(10).string(message.command);
    }
    if (message.workingDirectory !== undefined) {
      writer.uint32(18).string(message.workingDirectory);
    }
    if (message.backgroundJob !== undefined) {
      writer.uint32(24).bool(message.backgroundJob);
    }
    if (message.isPipelined !== undefined) {
      writer.uint32(32).bool(message.isPipelined);
    }
    for (const v of message.env) {
      EnvironmentVariable.encode(v!, writer.uint32(42).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): PseudoterminalExecuteRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...basePseudoterminalExecuteRequest,
    } as PseudoterminalExecuteRequest;
    message.env = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.command = reader.string();
          break;
        case 2:
          message.workingDirectory = reader.string();
          break;
        case 3:
          message.backgroundJob = reader.bool();
          break;
        case 4:
          message.isPipelined = reader.bool();
          break;
        case 5:
          message.env.push(EnvironmentVariable.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PseudoterminalExecuteRequest {
    const message = {
      ...basePseudoterminalExecuteRequest,
    } as PseudoterminalExecuteRequest;
    message.env = [];
    if (object.command !== undefined && object.command !== null) {
      message.command = String(object.command);
    }
    if (
      object.workingDirectory !== undefined &&
      object.workingDirectory !== null
    ) {
      message.workingDirectory = String(object.workingDirectory);
    }
    if (object.backgroundJob !== undefined && object.backgroundJob !== null) {
      message.backgroundJob = Boolean(object.backgroundJob);
    }
    if (object.isPipelined !== undefined && object.isPipelined !== null) {
      message.isPipelined = Boolean(object.isPipelined);
    }
    if (object.env !== undefined && object.env !== null) {
      for (const e of object.env) {
        message.env.push(EnvironmentVariable.fromJSON(e));
      }
    }
    return message;
  },

  toJSON(message: PseudoterminalExecuteRequest): unknown {
    const obj: any = {};
    message.command !== undefined && (obj.command = message.command);
    message.workingDirectory !== undefined &&
      (obj.workingDirectory = message.workingDirectory);
    message.backgroundJob !== undefined &&
      (obj.backgroundJob = message.backgroundJob);
    message.isPipelined !== undefined &&
      (obj.isPipelined = message.isPipelined);
    if (message.env) {
      obj.env = message.env.map((e) =>
        e ? EnvironmentVariable.toJSON(e) : undefined
      );
    } else {
      obj.env = [];
    }
    return obj;
  },

  fromPartial(
    object: DeepPartial<PseudoterminalExecuteRequest>
  ): PseudoterminalExecuteRequest {
    const message = {
      ...basePseudoterminalExecuteRequest,
    } as PseudoterminalExecuteRequest;
    message.env = [];
    if (object.command !== undefined && object.command !== null) {
      message.command = object.command;
    }
    if (
      object.workingDirectory !== undefined &&
      object.workingDirectory !== null
    ) {
      message.workingDirectory = object.workingDirectory;
    }
    if (object.backgroundJob !== undefined && object.backgroundJob !== null) {
      message.backgroundJob = object.backgroundJob;
    }
    if (object.isPipelined !== undefined && object.isPipelined !== null) {
      message.isPipelined = object.isPipelined;
    }
    if (object.env !== undefined && object.env !== null) {
      for (const e of object.env) {
        message.env.push(EnvironmentVariable.fromPartial(e));
      }
    }
    return message;
  },
};

const basePseudoterminalExecuteResponse: object = { stdout: "" };

export const PseudoterminalExecuteResponse = {
  encode(
    message: PseudoterminalExecuteResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.stdout !== "") {
      writer.uint32(10).string(message.stdout);
    }
    if (message.stderr !== undefined) {
      writer.uint32(18).string(message.stderr);
    }
    if (message.exitCode !== undefined) {
      writer.uint32(24).int32(message.exitCode);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): PseudoterminalExecuteResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...basePseudoterminalExecuteResponse,
    } as PseudoterminalExecuteResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.stdout = reader.string();
          break;
        case 2:
          message.stderr = reader.string();
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

  fromJSON(object: any): PseudoterminalExecuteResponse {
    const message = {
      ...basePseudoterminalExecuteResponse,
    } as PseudoterminalExecuteResponse;
    if (object.stdout !== undefined && object.stdout !== null) {
      message.stdout = String(object.stdout);
    }
    if (object.stderr !== undefined && object.stderr !== null) {
      message.stderr = String(object.stderr);
    }
    if (object.exitCode !== undefined && object.exitCode !== null) {
      message.exitCode = Number(object.exitCode);
    }
    return message;
  },

  toJSON(message: PseudoterminalExecuteResponse): unknown {
    const obj: any = {};
    message.stdout !== undefined && (obj.stdout = message.stdout);
    message.stderr !== undefined && (obj.stderr = message.stderr);
    message.exitCode !== undefined && (obj.exitCode = message.exitCode);
    return obj;
  },

  fromPartial(
    object: DeepPartial<PseudoterminalExecuteResponse>
  ): PseudoterminalExecuteResponse {
    const message = {
      ...basePseudoterminalExecuteResponse,
    } as PseudoterminalExecuteResponse;
    if (object.stdout !== undefined && object.stdout !== null) {
      message.stdout = object.stdout;
    }
    if (object.stderr !== undefined && object.stderr !== null) {
      message.stderr = object.stderr;
    }
    if (object.exitCode !== undefined && object.exitCode !== null) {
      message.exitCode = object.exitCode;
    }
    return message;
  },
};

const basePositionWindowRequest: object = {};

export const PositionWindowRequest = {
  encode(
    message: PositionWindowRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.anchor !== undefined) {
      Point.encode(message.anchor, writer.uint32(10).fork()).ldelim();
    }
    if (message.size !== undefined) {
      Size.encode(message.size, writer.uint32(18).fork()).ldelim();
    }
    if (message.dryrun !== undefined) {
      writer.uint32(24).bool(message.dryrun);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): PositionWindowRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...basePositionWindowRequest } as PositionWindowRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.anchor = Point.decode(reader, reader.uint32());
          break;
        case 2:
          message.size = Size.decode(reader, reader.uint32());
          break;
        case 3:
          message.dryrun = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PositionWindowRequest {
    const message = { ...basePositionWindowRequest } as PositionWindowRequest;
    if (object.anchor !== undefined && object.anchor !== null) {
      message.anchor = Point.fromJSON(object.anchor);
    }
    if (object.size !== undefined && object.size !== null) {
      message.size = Size.fromJSON(object.size);
    }
    if (object.dryrun !== undefined && object.dryrun !== null) {
      message.dryrun = Boolean(object.dryrun);
    }
    return message;
  },

  toJSON(message: PositionWindowRequest): unknown {
    const obj: any = {};
    message.anchor !== undefined &&
      (obj.anchor = message.anchor ? Point.toJSON(message.anchor) : undefined);
    message.size !== undefined &&
      (obj.size = message.size ? Size.toJSON(message.size) : undefined);
    message.dryrun !== undefined && (obj.dryrun = message.dryrun);
    return obj;
  },

  fromPartial(
    object: DeepPartial<PositionWindowRequest>
  ): PositionWindowRequest {
    const message = { ...basePositionWindowRequest } as PositionWindowRequest;
    if (object.anchor !== undefined && object.anchor !== null) {
      message.anchor = Point.fromPartial(object.anchor);
    }
    if (object.size !== undefined && object.size !== null) {
      message.size = Size.fromPartial(object.size);
    }
    if (object.dryrun !== undefined && object.dryrun !== null) {
      message.dryrun = object.dryrun;
    }
    return message;
  },
};

const basePositionWindowResponse: object = {};

export const PositionWindowResponse = {
  encode(
    message: PositionWindowResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.isAbove !== undefined) {
      writer.uint32(8).bool(message.isAbove);
    }
    if (message.isClipped !== undefined) {
      writer.uint32(16).bool(message.isClipped);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): PositionWindowResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...basePositionWindowResponse } as PositionWindowResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.isAbove = reader.bool();
          break;
        case 2:
          message.isClipped = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PositionWindowResponse {
    const message = { ...basePositionWindowResponse } as PositionWindowResponse;
    if (object.isAbove !== undefined && object.isAbove !== null) {
      message.isAbove = Boolean(object.isAbove);
    }
    if (object.isClipped !== undefined && object.isClipped !== null) {
      message.isClipped = Boolean(object.isClipped);
    }
    return message;
  },

  toJSON(message: PositionWindowResponse): unknown {
    const obj: any = {};
    message.isAbove !== undefined && (obj.isAbove = message.isAbove);
    message.isClipped !== undefined && (obj.isClipped = message.isClipped);
    return obj;
  },

  fromPartial(
    object: DeepPartial<PositionWindowResponse>
  ): PositionWindowResponse {
    const message = { ...basePositionWindowResponse } as PositionWindowResponse;
    if (object.isAbove !== undefined && object.isAbove !== null) {
      message.isAbove = object.isAbove;
    }
    if (object.isClipped !== undefined && object.isClipped !== null) {
      message.isClipped = object.isClipped;
    }
    return message;
  },
};

const baseReadFileRequest: object = {};

export const ReadFileRequest = {
  encode(
    message: ReadFileRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.path !== undefined) {
      FilePath.encode(message.path, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ReadFileRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReadFileRequest } as ReadFileRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.path = FilePath.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReadFileRequest {
    const message = { ...baseReadFileRequest } as ReadFileRequest;
    if (object.path !== undefined && object.path !== null) {
      message.path = FilePath.fromJSON(object.path);
    }
    return message;
  },

  toJSON(message: ReadFileRequest): unknown {
    const obj: any = {};
    message.path !== undefined &&
      (obj.path = message.path ? FilePath.toJSON(message.path) : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<ReadFileRequest>): ReadFileRequest {
    const message = { ...baseReadFileRequest } as ReadFileRequest;
    if (object.path !== undefined && object.path !== null) {
      message.path = FilePath.fromPartial(object.path);
    }
    return message;
  },
};

const baseReadFileResponse: object = {};

export const ReadFileResponse = {
  encode(
    message: ReadFileResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.data !== undefined) {
      writer.uint32(10).bytes(message.data);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ReadFileResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReadFileResponse } as ReadFileResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.data = reader.bytes();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReadFileResponse {
    const message = { ...baseReadFileResponse } as ReadFileResponse;
    if (object.data !== undefined && object.data !== null) {
      message.data = bytesFromBase64(object.data);
    }
    return message;
  },

  toJSON(message: ReadFileResponse): unknown {
    const obj: any = {};
    message.data !== undefined &&
      (obj.data =
        message.data !== undefined ? base64FromBytes(message.data) : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<ReadFileResponse>): ReadFileResponse {
    const message = { ...baseReadFileResponse } as ReadFileResponse;
    if (object.data !== undefined && object.data !== null) {
      message.data = object.data;
    }
    return message;
  },
};

const baseWriteFileRequest: object = {};

export const WriteFileRequest = {
  encode(
    message: WriteFileRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.path !== undefined) {
      FilePath.encode(message.path, writer.uint32(10).fork()).ldelim();
    }
    if (message.data?.$case === "text") {
      writer.uint32(18).string(message.data.text);
    }
    if (message.data?.$case === "binary") {
      writer.uint32(26).bytes(message.data.binary);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): WriteFileRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseWriteFileRequest } as WriteFileRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.path = FilePath.decode(reader, reader.uint32());
          break;
        case 2:
          message.data = { $case: "text", text: reader.string() };
          break;
        case 3:
          message.data = { $case: "binary", binary: reader.bytes() };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): WriteFileRequest {
    const message = { ...baseWriteFileRequest } as WriteFileRequest;
    if (object.path !== undefined && object.path !== null) {
      message.path = FilePath.fromJSON(object.path);
    }
    if (object.text !== undefined && object.text !== null) {
      message.data = { $case: "text", text: String(object.text) };
    }
    if (object.binary !== undefined && object.binary !== null) {
      message.data = {
        $case: "binary",
        binary: bytesFromBase64(object.binary),
      };
    }
    return message;
  },

  toJSON(message: WriteFileRequest): unknown {
    const obj: any = {};
    message.path !== undefined &&
      (obj.path = message.path ? FilePath.toJSON(message.path) : undefined);
    message.data?.$case === "text" && (obj.text = message.data?.text);
    message.data?.$case === "binary" &&
      (obj.binary =
        message.data?.binary !== undefined
          ? base64FromBytes(message.data?.binary)
          : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<WriteFileRequest>): WriteFileRequest {
    const message = { ...baseWriteFileRequest } as WriteFileRequest;
    if (object.path !== undefined && object.path !== null) {
      message.path = FilePath.fromPartial(object.path);
    }
    if (
      object.data?.$case === "text" &&
      object.data?.text !== undefined &&
      object.data?.text !== null
    ) {
      message.data = { $case: "text", text: object.data.text };
    }
    if (
      object.data?.$case === "binary" &&
      object.data?.binary !== undefined &&
      object.data?.binary !== null
    ) {
      message.data = { $case: "binary", binary: object.data.binary };
    }
    return message;
  },
};

const baseContentsOfDirectoryRequest: object = {};

export const ContentsOfDirectoryRequest = {
  encode(
    message: ContentsOfDirectoryRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.directory !== undefined) {
      FilePath.encode(message.directory, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ContentsOfDirectoryRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseContentsOfDirectoryRequest,
    } as ContentsOfDirectoryRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.directory = FilePath.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ContentsOfDirectoryRequest {
    const message = {
      ...baseContentsOfDirectoryRequest,
    } as ContentsOfDirectoryRequest;
    if (object.directory !== undefined && object.directory !== null) {
      message.directory = FilePath.fromJSON(object.directory);
    }
    return message;
  },

  toJSON(message: ContentsOfDirectoryRequest): unknown {
    const obj: any = {};
    message.directory !== undefined &&
      (obj.directory = message.directory
        ? FilePath.toJSON(message.directory)
        : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ContentsOfDirectoryRequest>
  ): ContentsOfDirectoryRequest {
    const message = {
      ...baseContentsOfDirectoryRequest,
    } as ContentsOfDirectoryRequest;
    if (object.directory !== undefined && object.directory !== null) {
      message.directory = FilePath.fromPartial(object.directory);
    }
    return message;
  },
};

const baseContentsOfDirectoryResponse: object = { fileNames: "" };

export const ContentsOfDirectoryResponse = {
  encode(
    message: ContentsOfDirectoryResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.fileNames) {
      writer.uint32(10).string(v!);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ContentsOfDirectoryResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseContentsOfDirectoryResponse,
    } as ContentsOfDirectoryResponse;
    message.fileNames = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileNames.push(reader.string());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ContentsOfDirectoryResponse {
    const message = {
      ...baseContentsOfDirectoryResponse,
    } as ContentsOfDirectoryResponse;
    message.fileNames = [];
    if (object.fileNames !== undefined && object.fileNames !== null) {
      for (const e of object.fileNames) {
        message.fileNames.push(String(e));
      }
    }
    return message;
  },

  toJSON(message: ContentsOfDirectoryResponse): unknown {
    const obj: any = {};
    if (message.fileNames) {
      obj.fileNames = message.fileNames.map((e) => e);
    } else {
      obj.fileNames = [];
    }
    return obj;
  },

  fromPartial(
    object: DeepPartial<ContentsOfDirectoryResponse>
  ): ContentsOfDirectoryResponse {
    const message = {
      ...baseContentsOfDirectoryResponse,
    } as ContentsOfDirectoryResponse;
    message.fileNames = [];
    if (object.fileNames !== undefined && object.fileNames !== null) {
      for (const e of object.fileNames) {
        message.fileNames.push(e);
      }
    }
    return message;
  },
};

const baseDestinationOfSymbolicLinkRequest: object = {};

export const DestinationOfSymbolicLinkRequest = {
  encode(
    message: DestinationOfSymbolicLinkRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.path !== undefined) {
      FilePath.encode(message.path, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): DestinationOfSymbolicLinkRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseDestinationOfSymbolicLinkRequest,
    } as DestinationOfSymbolicLinkRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.path = FilePath.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): DestinationOfSymbolicLinkRequest {
    const message = {
      ...baseDestinationOfSymbolicLinkRequest,
    } as DestinationOfSymbolicLinkRequest;
    if (object.path !== undefined && object.path !== null) {
      message.path = FilePath.fromJSON(object.path);
    }
    return message;
  },

  toJSON(message: DestinationOfSymbolicLinkRequest): unknown {
    const obj: any = {};
    message.path !== undefined &&
      (obj.path = message.path ? FilePath.toJSON(message.path) : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<DestinationOfSymbolicLinkRequest>
  ): DestinationOfSymbolicLinkRequest {
    const message = {
      ...baseDestinationOfSymbolicLinkRequest,
    } as DestinationOfSymbolicLinkRequest;
    if (object.path !== undefined && object.path !== null) {
      message.path = FilePath.fromPartial(object.path);
    }
    return message;
  },
};

const baseDestinationOfSymbolicLinkResponse: object = {};

export const DestinationOfSymbolicLinkResponse = {
  encode(
    message: DestinationOfSymbolicLinkResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.destination !== undefined) {
      FilePath.encode(message.destination, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): DestinationOfSymbolicLinkResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseDestinationOfSymbolicLinkResponse,
    } as DestinationOfSymbolicLinkResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.destination = FilePath.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): DestinationOfSymbolicLinkResponse {
    const message = {
      ...baseDestinationOfSymbolicLinkResponse,
    } as DestinationOfSymbolicLinkResponse;
    if (object.destination !== undefined && object.destination !== null) {
      message.destination = FilePath.fromJSON(object.destination);
    }
    return message;
  },

  toJSON(message: DestinationOfSymbolicLinkResponse): unknown {
    const obj: any = {};
    message.destination !== undefined &&
      (obj.destination = message.destination
        ? FilePath.toJSON(message.destination)
        : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<DestinationOfSymbolicLinkResponse>
  ): DestinationOfSymbolicLinkResponse {
    const message = {
      ...baseDestinationOfSymbolicLinkResponse,
    } as DestinationOfSymbolicLinkResponse;
    if (object.destination !== undefined && object.destination !== null) {
      message.destination = FilePath.fromPartial(object.destination);
    }
    return message;
  },
};

const baseGetSettingsPropertyRequest: object = {};

export const GetSettingsPropertyRequest = {
  encode(
    message: GetSettingsPropertyRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.key !== undefined) {
      writer.uint32(10).string(message.key);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): GetSettingsPropertyRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseGetSettingsPropertyRequest,
    } as GetSettingsPropertyRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.key = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): GetSettingsPropertyRequest {
    const message = {
      ...baseGetSettingsPropertyRequest,
    } as GetSettingsPropertyRequest;
    if (object.key !== undefined && object.key !== null) {
      message.key = String(object.key);
    }
    return message;
  },

  toJSON(message: GetSettingsPropertyRequest): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = message.key);
    return obj;
  },

  fromPartial(
    object: DeepPartial<GetSettingsPropertyRequest>
  ): GetSettingsPropertyRequest {
    const message = {
      ...baseGetSettingsPropertyRequest,
    } as GetSettingsPropertyRequest;
    if (object.key !== undefined && object.key !== null) {
      message.key = object.key;
    }
    return message;
  },
};

const baseGetSettingsPropertyResponse: object = {};

export const GetSettingsPropertyResponse = {
  encode(
    message: GetSettingsPropertyResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.jsonBlob !== undefined) {
      writer.uint32(10).string(message.jsonBlob);
    }
    if (message.isDefault !== undefined) {
      writer.uint32(16).bool(message.isDefault);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): GetSettingsPropertyResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseGetSettingsPropertyResponse,
    } as GetSettingsPropertyResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.jsonBlob = reader.string();
          break;
        case 2:
          message.isDefault = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): GetSettingsPropertyResponse {
    const message = {
      ...baseGetSettingsPropertyResponse,
    } as GetSettingsPropertyResponse;
    if (object.jsonBlob !== undefined && object.jsonBlob !== null) {
      message.jsonBlob = String(object.jsonBlob);
    }
    if (object.isDefault !== undefined && object.isDefault !== null) {
      message.isDefault = Boolean(object.isDefault);
    }
    return message;
  },

  toJSON(message: GetSettingsPropertyResponse): unknown {
    const obj: any = {};
    message.jsonBlob !== undefined && (obj.jsonBlob = message.jsonBlob);
    message.isDefault !== undefined && (obj.isDefault = message.isDefault);
    return obj;
  },

  fromPartial(
    object: DeepPartial<GetSettingsPropertyResponse>
  ): GetSettingsPropertyResponse {
    const message = {
      ...baseGetSettingsPropertyResponse,
    } as GetSettingsPropertyResponse;
    if (object.jsonBlob !== undefined && object.jsonBlob !== null) {
      message.jsonBlob = object.jsonBlob;
    }
    if (object.isDefault !== undefined && object.isDefault !== null) {
      message.isDefault = object.isDefault;
    }
    return message;
  },
};

const baseUpdateSettingsPropertyRequest: object = {};

export const UpdateSettingsPropertyRequest = {
  encode(
    message: UpdateSettingsPropertyRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.key !== undefined) {
      writer.uint32(10).string(message.key);
    }
    if (message.value !== undefined) {
      writer.uint32(18).string(message.value);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): UpdateSettingsPropertyRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseUpdateSettingsPropertyRequest,
    } as UpdateSettingsPropertyRequest;
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

  fromJSON(object: any): UpdateSettingsPropertyRequest {
    const message = {
      ...baseUpdateSettingsPropertyRequest,
    } as UpdateSettingsPropertyRequest;
    if (object.key !== undefined && object.key !== null) {
      message.key = String(object.key);
    }
    if (object.value !== undefined && object.value !== null) {
      message.value = String(object.value);
    }
    return message;
  },

  toJSON(message: UpdateSettingsPropertyRequest): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = message.key);
    message.value !== undefined && (obj.value = message.value);
    return obj;
  },

  fromPartial(
    object: DeepPartial<UpdateSettingsPropertyRequest>
  ): UpdateSettingsPropertyRequest {
    const message = {
      ...baseUpdateSettingsPropertyRequest,
    } as UpdateSettingsPropertyRequest;
    if (object.key !== undefined && object.key !== null) {
      message.key = object.key;
    }
    if (object.value !== undefined && object.value !== null) {
      message.value = object.value;
    }
    return message;
  },
};

const baseAction: object = {};

export const Action = {
  encode(
    message: Action,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.identifier !== undefined) {
      writer.uint32(10).string(message.identifier);
    }
    if (message.name !== undefined) {
      writer.uint32(18).string(message.name);
    }
    if (message.description !== undefined) {
      writer.uint32(26).string(message.description);
    }
    if (message.category !== undefined) {
      writer.uint32(34).string(message.category);
    }
    if (message.availability !== undefined) {
      writer.uint32(40).int32(message.availability);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Action {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseAction } as Action;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.identifier = reader.string();
          break;
        case 2:
          message.name = reader.string();
          break;
        case 3:
          message.description = reader.string();
          break;
        case 4:
          message.category = reader.string();
          break;
        case 5:
          message.availability = reader.int32() as any;
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Action {
    const message = { ...baseAction } as Action;
    if (object.identifier !== undefined && object.identifier !== null) {
      message.identifier = String(object.identifier);
    }
    if (object.name !== undefined && object.name !== null) {
      message.name = String(object.name);
    }
    if (object.description !== undefined && object.description !== null) {
      message.description = String(object.description);
    }
    if (object.category !== undefined && object.category !== null) {
      message.category = String(object.category);
    }
    if (object.availability !== undefined && object.availability !== null) {
      message.availability = actionAvailabilityFromJSON(object.availability);
    }
    return message;
  },

  toJSON(message: Action): unknown {
    const obj: any = {};
    message.identifier !== undefined && (obj.identifier = message.identifier);
    message.name !== undefined && (obj.name = message.name);
    message.description !== undefined &&
      (obj.description = message.description);
    message.category !== undefined && (obj.category = message.category);
    message.availability !== undefined &&
      (obj.availability =
        message.availability !== undefined
          ? actionAvailabilityToJSON(message.availability)
          : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<Action>): Action {
    const message = { ...baseAction } as Action;
    if (object.identifier !== undefined && object.identifier !== null) {
      message.identifier = object.identifier;
    }
    if (object.name !== undefined && object.name !== null) {
      message.name = object.name;
    }
    if (object.description !== undefined && object.description !== null) {
      message.description = object.description;
    }
    if (object.category !== undefined && object.category !== null) {
      message.category = object.category;
    }
    if (object.availability !== undefined && object.availability !== null) {
      message.availability = object.availability;
    }
    return message;
  },
};

const baseUpdateApplicationPropertiesRequest: object = {};

export const UpdateApplicationPropertiesRequest = {
  encode(
    message: UpdateApplicationPropertiesRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.interceptBoundKeystrokes !== undefined) {
      writer.uint32(8).bool(message.interceptBoundKeystrokes);
    }
    for (const v of message.actions) {
      Action.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): UpdateApplicationPropertiesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseUpdateApplicationPropertiesRequest,
    } as UpdateApplicationPropertiesRequest;
    message.actions = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.interceptBoundKeystrokes = reader.bool();
          break;
        case 2:
          message.actions.push(Action.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): UpdateApplicationPropertiesRequest {
    const message = {
      ...baseUpdateApplicationPropertiesRequest,
    } as UpdateApplicationPropertiesRequest;
    message.actions = [];
    if (
      object.interceptBoundKeystrokes !== undefined &&
      object.interceptBoundKeystrokes !== null
    ) {
      message.interceptBoundKeystrokes = Boolean(
        object.interceptBoundKeystrokes
      );
    }
    if (object.actions !== undefined && object.actions !== null) {
      for (const e of object.actions) {
        message.actions.push(Action.fromJSON(e));
      }
    }
    return message;
  },

  toJSON(message: UpdateApplicationPropertiesRequest): unknown {
    const obj: any = {};
    message.interceptBoundKeystrokes !== undefined &&
      (obj.interceptBoundKeystrokes = message.interceptBoundKeystrokes);
    if (message.actions) {
      obj.actions = message.actions.map((e) =>
        e ? Action.toJSON(e) : undefined
      );
    } else {
      obj.actions = [];
    }
    return obj;
  },

  fromPartial(
    object: DeepPartial<UpdateApplicationPropertiesRequest>
  ): UpdateApplicationPropertiesRequest {
    const message = {
      ...baseUpdateApplicationPropertiesRequest,
    } as UpdateApplicationPropertiesRequest;
    message.actions = [];
    if (
      object.interceptBoundKeystrokes !== undefined &&
      object.interceptBoundKeystrokes !== null
    ) {
      message.interceptBoundKeystrokes = object.interceptBoundKeystrokes;
    }
    if (object.actions !== undefined && object.actions !== null) {
      for (const e of object.actions) {
        message.actions.push(Action.fromPartial(e));
      }
    }
    return message;
  },
};

const baseNotificationRequest: object = {};

export const NotificationRequest = {
  encode(
    message: NotificationRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.subscribe !== undefined) {
      writer.uint32(8).bool(message.subscribe);
    }
    if (message.type !== undefined) {
      writer.uint32(16).int32(message.type);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): NotificationRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseNotificationRequest } as NotificationRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.subscribe = reader.bool();
          break;
        case 2:
          message.type = reader.int32() as any;
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): NotificationRequest {
    const message = { ...baseNotificationRequest } as NotificationRequest;
    if (object.subscribe !== undefined && object.subscribe !== null) {
      message.subscribe = Boolean(object.subscribe);
    }
    if (object.type !== undefined && object.type !== null) {
      message.type = notificationTypeFromJSON(object.type);
    }
    return message;
  },

  toJSON(message: NotificationRequest): unknown {
    const obj: any = {};
    message.subscribe !== undefined && (obj.subscribe = message.subscribe);
    message.type !== undefined &&
      (obj.type =
        message.type !== undefined
          ? notificationTypeToJSON(message.type)
          : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<NotificationRequest>): NotificationRequest {
    const message = { ...baseNotificationRequest } as NotificationRequest;
    if (object.subscribe !== undefined && object.subscribe !== null) {
      message.subscribe = object.subscribe;
    }
    if (object.type !== undefined && object.type !== null) {
      message.type = object.type;
    }
    return message;
  },
};

const baseNotification: object = {};

export const Notification = {
  encode(
    message: Notification,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.type?.$case === "editBufferNotification") {
      EditBufferChangedNotification.encode(
        message.type.editBufferNotification,
        writer.uint32(10).fork()
      ).ldelim();
    }
    if (message.type?.$case === "settingsChangedNotification") {
      SettingsChangedNotification.encode(
        message.type.settingsChangedNotification,
        writer.uint32(18).fork()
      ).ldelim();
    }
    if (message.type?.$case === "shellPromptReturnedNotification") {
      ShellPromptReturnedNotification.encode(
        message.type.shellPromptReturnedNotification,
        writer.uint32(26).fork()
      ).ldelim();
    }
    if (message.type?.$case === "locationChangedNotification") {
      LocationChangedNotification.encode(
        message.type.locationChangedNotification,
        writer.uint32(34).fork()
      ).ldelim();
    }
    if (message.type?.$case === "processChangeNotification") {
      ProcessChangedNotification.encode(
        message.type.processChangeNotification,
        writer.uint32(42).fork()
      ).ldelim();
    }
    if (message.type?.$case === "keybindingPressedNotification") {
      KeybindingPressedNotification.encode(
        message.type.keybindingPressedNotification,
        writer.uint32(50).fork()
      ).ldelim();
    }
    if (message.type?.$case === "windowFocusChangedNotification") {
      WindowFocusChangedNotification.encode(
        message.type.windowFocusChangedNotification,
        writer.uint32(58).fork()
      ).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Notification {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseNotification } as Notification;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.type = {
            $case: "editBufferNotification",
            editBufferNotification: EditBufferChangedNotification.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 2:
          message.type = {
            $case: "settingsChangedNotification",
            settingsChangedNotification: SettingsChangedNotification.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 3:
          message.type = {
            $case: "shellPromptReturnedNotification",
            shellPromptReturnedNotification:
              ShellPromptReturnedNotification.decode(reader, reader.uint32()),
          };
          break;
        case 4:
          message.type = {
            $case: "locationChangedNotification",
            locationChangedNotification: LocationChangedNotification.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 5:
          message.type = {
            $case: "processChangeNotification",
            processChangeNotification: ProcessChangedNotification.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 6:
          message.type = {
            $case: "keybindingPressedNotification",
            keybindingPressedNotification: KeybindingPressedNotification.decode(
              reader,
              reader.uint32()
            ),
          };
          break;
        case 7:
          message.type = {
            $case: "windowFocusChangedNotification",
            windowFocusChangedNotification:
              WindowFocusChangedNotification.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Notification {
    const message = { ...baseNotification } as Notification;
    if (
      object.editBufferNotification !== undefined &&
      object.editBufferNotification !== null
    ) {
      message.type = {
        $case: "editBufferNotification",
        editBufferNotification: EditBufferChangedNotification.fromJSON(
          object.editBufferNotification
        ),
      };
    }
    if (
      object.settingsChangedNotification !== undefined &&
      object.settingsChangedNotification !== null
    ) {
      message.type = {
        $case: "settingsChangedNotification",
        settingsChangedNotification: SettingsChangedNotification.fromJSON(
          object.settingsChangedNotification
        ),
      };
    }
    if (
      object.shellPromptReturnedNotification !== undefined &&
      object.shellPromptReturnedNotification !== null
    ) {
      message.type = {
        $case: "shellPromptReturnedNotification",
        shellPromptReturnedNotification:
          ShellPromptReturnedNotification.fromJSON(
            object.shellPromptReturnedNotification
          ),
      };
    }
    if (
      object.locationChangedNotification !== undefined &&
      object.locationChangedNotification !== null
    ) {
      message.type = {
        $case: "locationChangedNotification",
        locationChangedNotification: LocationChangedNotification.fromJSON(
          object.locationChangedNotification
        ),
      };
    }
    if (
      object.processChangeNotification !== undefined &&
      object.processChangeNotification !== null
    ) {
      message.type = {
        $case: "processChangeNotification",
        processChangeNotification: ProcessChangedNotification.fromJSON(
          object.processChangeNotification
        ),
      };
    }
    if (
      object.keybindingPressedNotification !== undefined &&
      object.keybindingPressedNotification !== null
    ) {
      message.type = {
        $case: "keybindingPressedNotification",
        keybindingPressedNotification: KeybindingPressedNotification.fromJSON(
          object.keybindingPressedNotification
        ),
      };
    }
    if (
      object.windowFocusChangedNotification !== undefined &&
      object.windowFocusChangedNotification !== null
    ) {
      message.type = {
        $case: "windowFocusChangedNotification",
        windowFocusChangedNotification: WindowFocusChangedNotification.fromJSON(
          object.windowFocusChangedNotification
        ),
      };
    }
    return message;
  },

  toJSON(message: Notification): unknown {
    const obj: any = {};
    message.type?.$case === "editBufferNotification" &&
      (obj.editBufferNotification = message.type?.editBufferNotification
        ? EditBufferChangedNotification.toJSON(
            message.type?.editBufferNotification
          )
        : undefined);
    message.type?.$case === "settingsChangedNotification" &&
      (obj.settingsChangedNotification = message.type
        ?.settingsChangedNotification
        ? SettingsChangedNotification.toJSON(
            message.type?.settingsChangedNotification
          )
        : undefined);
    message.type?.$case === "shellPromptReturnedNotification" &&
      (obj.shellPromptReturnedNotification = message.type
        ?.shellPromptReturnedNotification
        ? ShellPromptReturnedNotification.toJSON(
            message.type?.shellPromptReturnedNotification
          )
        : undefined);
    message.type?.$case === "locationChangedNotification" &&
      (obj.locationChangedNotification = message.type
        ?.locationChangedNotification
        ? LocationChangedNotification.toJSON(
            message.type?.locationChangedNotification
          )
        : undefined);
    message.type?.$case === "processChangeNotification" &&
      (obj.processChangeNotification = message.type?.processChangeNotification
        ? ProcessChangedNotification.toJSON(
            message.type?.processChangeNotification
          )
        : undefined);
    message.type?.$case === "keybindingPressedNotification" &&
      (obj.keybindingPressedNotification = message.type
        ?.keybindingPressedNotification
        ? KeybindingPressedNotification.toJSON(
            message.type?.keybindingPressedNotification
          )
        : undefined);
    message.type?.$case === "windowFocusChangedNotification" &&
      (obj.windowFocusChangedNotification = message.type
        ?.windowFocusChangedNotification
        ? WindowFocusChangedNotification.toJSON(
            message.type?.windowFocusChangedNotification
          )
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<Notification>): Notification {
    const message = { ...baseNotification } as Notification;
    if (
      object.type?.$case === "editBufferNotification" &&
      object.type?.editBufferNotification !== undefined &&
      object.type?.editBufferNotification !== null
    ) {
      message.type = {
        $case: "editBufferNotification",
        editBufferNotification: EditBufferChangedNotification.fromPartial(
          object.type.editBufferNotification
        ),
      };
    }
    if (
      object.type?.$case === "settingsChangedNotification" &&
      object.type?.settingsChangedNotification !== undefined &&
      object.type?.settingsChangedNotification !== null
    ) {
      message.type = {
        $case: "settingsChangedNotification",
        settingsChangedNotification: SettingsChangedNotification.fromPartial(
          object.type.settingsChangedNotification
        ),
      };
    }
    if (
      object.type?.$case === "shellPromptReturnedNotification" &&
      object.type?.shellPromptReturnedNotification !== undefined &&
      object.type?.shellPromptReturnedNotification !== null
    ) {
      message.type = {
        $case: "shellPromptReturnedNotification",
        shellPromptReturnedNotification:
          ShellPromptReturnedNotification.fromPartial(
            object.type.shellPromptReturnedNotification
          ),
      };
    }
    if (
      object.type?.$case === "locationChangedNotification" &&
      object.type?.locationChangedNotification !== undefined &&
      object.type?.locationChangedNotification !== null
    ) {
      message.type = {
        $case: "locationChangedNotification",
        locationChangedNotification: LocationChangedNotification.fromPartial(
          object.type.locationChangedNotification
        ),
      };
    }
    if (
      object.type?.$case === "processChangeNotification" &&
      object.type?.processChangeNotification !== undefined &&
      object.type?.processChangeNotification !== null
    ) {
      message.type = {
        $case: "processChangeNotification",
        processChangeNotification: ProcessChangedNotification.fromPartial(
          object.type.processChangeNotification
        ),
      };
    }
    if (
      object.type?.$case === "keybindingPressedNotification" &&
      object.type?.keybindingPressedNotification !== undefined &&
      object.type?.keybindingPressedNotification !== null
    ) {
      message.type = {
        $case: "keybindingPressedNotification",
        keybindingPressedNotification:
          KeybindingPressedNotification.fromPartial(
            object.type.keybindingPressedNotification
          ),
      };
    }
    if (
      object.type?.$case === "windowFocusChangedNotification" &&
      object.type?.windowFocusChangedNotification !== undefined &&
      object.type?.windowFocusChangedNotification !== null
    ) {
      message.type = {
        $case: "windowFocusChangedNotification",
        windowFocusChangedNotification:
          WindowFocusChangedNotification.fromPartial(
            object.type.windowFocusChangedNotification
          ),
      };
    }
    return message;
  },
};

const baseEditBufferChangedNotification: object = {};

export const EditBufferChangedNotification = {
  encode(
    message: EditBufferChangedNotification,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.sessionId !== undefined) {
      writer.uint32(10).string(message.sessionId);
    }
    if (message.cursor !== undefined) {
      writer.uint32(16).int32(message.cursor);
    }
    if (message.buffer !== undefined) {
      writer.uint32(26).string(message.buffer);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): EditBufferChangedNotification {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseEditBufferChangedNotification,
    } as EditBufferChangedNotification;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.sessionId = reader.string();
          break;
        case 2:
          message.cursor = reader.int32();
          break;
        case 3:
          message.buffer = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): EditBufferChangedNotification {
    const message = {
      ...baseEditBufferChangedNotification,
    } as EditBufferChangedNotification;
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = String(object.sessionId);
    }
    if (object.cursor !== undefined && object.cursor !== null) {
      message.cursor = Number(object.cursor);
    }
    if (object.buffer !== undefined && object.buffer !== null) {
      message.buffer = String(object.buffer);
    }
    return message;
  },

  toJSON(message: EditBufferChangedNotification): unknown {
    const obj: any = {};
    message.sessionId !== undefined && (obj.sessionId = message.sessionId);
    message.cursor !== undefined && (obj.cursor = message.cursor);
    message.buffer !== undefined && (obj.buffer = message.buffer);
    return obj;
  },

  fromPartial(
    object: DeepPartial<EditBufferChangedNotification>
  ): EditBufferChangedNotification {
    const message = {
      ...baseEditBufferChangedNotification,
    } as EditBufferChangedNotification;
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = object.sessionId;
    }
    if (object.cursor !== undefined && object.cursor !== null) {
      message.cursor = object.cursor;
    }
    if (object.buffer !== undefined && object.buffer !== null) {
      message.buffer = object.buffer;
    }
    return message;
  },
};

const baseSettingsChangedNotification: object = {};

export const SettingsChangedNotification = {
  encode(
    message: SettingsChangedNotification,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.jsonBlob !== undefined) {
      writer.uint32(10).string(message.jsonBlob);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): SettingsChangedNotification {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseSettingsChangedNotification,
    } as SettingsChangedNotification;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.jsonBlob = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): SettingsChangedNotification {
    const message = {
      ...baseSettingsChangedNotification,
    } as SettingsChangedNotification;
    if (object.jsonBlob !== undefined && object.jsonBlob !== null) {
      message.jsonBlob = String(object.jsonBlob);
    }
    return message;
  },

  toJSON(message: SettingsChangedNotification): unknown {
    const obj: any = {};
    message.jsonBlob !== undefined && (obj.jsonBlob = message.jsonBlob);
    return obj;
  },

  fromPartial(
    object: DeepPartial<SettingsChangedNotification>
  ): SettingsChangedNotification {
    const message = {
      ...baseSettingsChangedNotification,
    } as SettingsChangedNotification;
    if (object.jsonBlob !== undefined && object.jsonBlob !== null) {
      message.jsonBlob = object.jsonBlob;
    }
    return message;
  },
};

const baseShellPromptReturnedNotification: object = {};

export const ShellPromptReturnedNotification = {
  encode(
    message: ShellPromptReturnedNotification,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.sessionId !== undefined) {
      writer.uint32(10).string(message.sessionId);
    }
    if (message.shell !== undefined) {
      Process.encode(message.shell, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ShellPromptReturnedNotification {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseShellPromptReturnedNotification,
    } as ShellPromptReturnedNotification;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.sessionId = reader.string();
          break;
        case 2:
          message.shell = Process.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ShellPromptReturnedNotification {
    const message = {
      ...baseShellPromptReturnedNotification,
    } as ShellPromptReturnedNotification;
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = String(object.sessionId);
    }
    if (object.shell !== undefined && object.shell !== null) {
      message.shell = Process.fromJSON(object.shell);
    }
    return message;
  },

  toJSON(message: ShellPromptReturnedNotification): unknown {
    const obj: any = {};
    message.sessionId !== undefined && (obj.sessionId = message.sessionId);
    message.shell !== undefined &&
      (obj.shell = message.shell ? Process.toJSON(message.shell) : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ShellPromptReturnedNotification>
  ): ShellPromptReturnedNotification {
    const message = {
      ...baseShellPromptReturnedNotification,
    } as ShellPromptReturnedNotification;
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = object.sessionId;
    }
    if (object.shell !== undefined && object.shell !== null) {
      message.shell = Process.fromPartial(object.shell);
    }
    return message;
  },
};

const baseLocationChangedNotification: object = {};

export const LocationChangedNotification = {
  encode(
    message: LocationChangedNotification,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.sessionId !== undefined) {
      writer.uint32(10).string(message.sessionId);
    }
    if (message.hostName !== undefined) {
      writer.uint32(18).string(message.hostName);
    }
    if (message.userName !== undefined) {
      writer.uint32(26).string(message.userName);
    }
    if (message.directory !== undefined) {
      writer.uint32(34).string(message.directory);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): LocationChangedNotification {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseLocationChangedNotification,
    } as LocationChangedNotification;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.sessionId = reader.string();
          break;
        case 2:
          message.hostName = reader.string();
          break;
        case 3:
          message.userName = reader.string();
          break;
        case 4:
          message.directory = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): LocationChangedNotification {
    const message = {
      ...baseLocationChangedNotification,
    } as LocationChangedNotification;
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = String(object.sessionId);
    }
    if (object.hostName !== undefined && object.hostName !== null) {
      message.hostName = String(object.hostName);
    }
    if (object.userName !== undefined && object.userName !== null) {
      message.userName = String(object.userName);
    }
    if (object.directory !== undefined && object.directory !== null) {
      message.directory = String(object.directory);
    }
    return message;
  },

  toJSON(message: LocationChangedNotification): unknown {
    const obj: any = {};
    message.sessionId !== undefined && (obj.sessionId = message.sessionId);
    message.hostName !== undefined && (obj.hostName = message.hostName);
    message.userName !== undefined && (obj.userName = message.userName);
    message.directory !== undefined && (obj.directory = message.directory);
    return obj;
  },

  fromPartial(
    object: DeepPartial<LocationChangedNotification>
  ): LocationChangedNotification {
    const message = {
      ...baseLocationChangedNotification,
    } as LocationChangedNotification;
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = object.sessionId;
    }
    if (object.hostName !== undefined && object.hostName !== null) {
      message.hostName = object.hostName;
    }
    if (object.userName !== undefined && object.userName !== null) {
      message.userName = object.userName;
    }
    if (object.directory !== undefined && object.directory !== null) {
      message.directory = object.directory;
    }
    return message;
  },
};

const baseProcessChangedNotification: object = {};

export const ProcessChangedNotification = {
  encode(
    message: ProcessChangedNotification,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.sessionId !== undefined) {
      writer.uint32(10).string(message.sessionId);
    }
    if (message.newProcess !== undefined) {
      Process.encode(message.newProcess, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ProcessChangedNotification {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseProcessChangedNotification,
    } as ProcessChangedNotification;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.sessionId = reader.string();
          break;
        case 2:
          message.newProcess = Process.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessChangedNotification {
    const message = {
      ...baseProcessChangedNotification,
    } as ProcessChangedNotification;
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = String(object.sessionId);
    }
    if (object.newProcess !== undefined && object.newProcess !== null) {
      message.newProcess = Process.fromJSON(object.newProcess);
    }
    return message;
  },

  toJSON(message: ProcessChangedNotification): unknown {
    const obj: any = {};
    message.sessionId !== undefined && (obj.sessionId = message.sessionId);
    message.newProcess !== undefined &&
      (obj.newProcess = message.newProcess
        ? Process.toJSON(message.newProcess)
        : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ProcessChangedNotification>
  ): ProcessChangedNotification {
    const message = {
      ...baseProcessChangedNotification,
    } as ProcessChangedNotification;
    if (object.sessionId !== undefined && object.sessionId !== null) {
      message.sessionId = object.sessionId;
    }
    if (object.newProcess !== undefined && object.newProcess !== null) {
      message.newProcess = Process.fromPartial(object.newProcess);
    }
    return message;
  },
};

const baseKeybindingPressedNotification: object = {};

export const KeybindingPressedNotification = {
  encode(
    message: KeybindingPressedNotification,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.keypress !== undefined) {
      KeyEvent.encode(message.keypress, writer.uint32(10).fork()).ldelim();
    }
    if (message.action !== undefined) {
      writer.uint32(18).string(message.action);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): KeybindingPressedNotification {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseKeybindingPressedNotification,
    } as KeybindingPressedNotification;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.keypress = KeyEvent.decode(reader, reader.uint32());
          break;
        case 2:
          message.action = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): KeybindingPressedNotification {
    const message = {
      ...baseKeybindingPressedNotification,
    } as KeybindingPressedNotification;
    if (object.keypress !== undefined && object.keypress !== null) {
      message.keypress = KeyEvent.fromJSON(object.keypress);
    }
    if (object.action !== undefined && object.action !== null) {
      message.action = String(object.action);
    }
    return message;
  },

  toJSON(message: KeybindingPressedNotification): unknown {
    const obj: any = {};
    message.keypress !== undefined &&
      (obj.keypress = message.keypress
        ? KeyEvent.toJSON(message.keypress)
        : undefined);
    message.action !== undefined && (obj.action = message.action);
    return obj;
  },

  fromPartial(
    object: DeepPartial<KeybindingPressedNotification>
  ): KeybindingPressedNotification {
    const message = {
      ...baseKeybindingPressedNotification,
    } as KeybindingPressedNotification;
    if (object.keypress !== undefined && object.keypress !== null) {
      message.keypress = KeyEvent.fromPartial(object.keypress);
    }
    if (object.action !== undefined && object.action !== null) {
      message.action = object.action;
    }
    return message;
  },
};

const baseWindowFocusChangedNotification: object = {};

export const WindowFocusChangedNotification = {
  encode(
    message: WindowFocusChangedNotification,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.window !== undefined) {
      Window.encode(message.window, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): WindowFocusChangedNotification {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseWindowFocusChangedNotification,
    } as WindowFocusChangedNotification;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.window = Window.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): WindowFocusChangedNotification {
    const message = {
      ...baseWindowFocusChangedNotification,
    } as WindowFocusChangedNotification;
    if (object.window !== undefined && object.window !== null) {
      message.window = Window.fromJSON(object.window);
    }
    return message;
  },

  toJSON(message: WindowFocusChangedNotification): unknown {
    const obj: any = {};
    message.window !== undefined &&
      (obj.window = message.window ? Window.toJSON(message.window) : undefined);
    return obj;
  },

  fromPartial(
    object: DeepPartial<WindowFocusChangedNotification>
  ): WindowFocusChangedNotification {
    const message = {
      ...baseWindowFocusChangedNotification,
    } as WindowFocusChangedNotification;
    if (object.window !== undefined && object.window !== null) {
      message.window = Window.fromPartial(object.window);
    }
    return message;
  },
};

const baseConstants: object = {};

export const Constants = {
  encode(
    message: Constants,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.version !== undefined) {
      writer.uint32(10).string(message.version);
    }
    if (message.build !== undefined) {
      writer.uint32(18).string(message.build);
    }
    if (message.cli !== undefined) {
      writer.uint32(26).string(message.cli);
    }
    if (message.bundlePath !== undefined) {
      writer.uint32(34).string(message.bundlePath);
    }
    if (message.remote !== undefined) {
      writer.uint32(42).string(message.remote);
    }
    if (message.home !== undefined) {
      writer.uint32(50).string(message.home);
    }
    if (message.user !== undefined) {
      writer.uint32(58).string(message.user);
    }
    if (message.defaultPath !== undefined) {
      writer.uint32(66).string(message.defaultPath);
    }
    if (message.jsonMessageRecieved !== undefined) {
      writer.uint32(74).string(message.jsonMessageRecieved);
    }
    if (message.jsonMessageHandler !== undefined) {
      writer.uint32(82).string(message.jsonMessageHandler);
    }
    if (message.protoMessageRecieved !== undefined) {
      writer.uint32(90).string(message.protoMessageRecieved);
    }
    if (message.protoMessageHandler !== undefined) {
      writer.uint32(98).string(message.protoMessageHandler);
    }
    if (message.themes !== undefined) {
      writer.uint32(106).string(message.themes);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Constants {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseConstants } as Constants;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.version = reader.string();
          break;
        case 2:
          message.build = reader.string();
          break;
        case 3:
          message.cli = reader.string();
          break;
        case 4:
          message.bundlePath = reader.string();
          break;
        case 5:
          message.remote = reader.string();
          break;
        case 6:
          message.home = reader.string();
          break;
        case 7:
          message.user = reader.string();
          break;
        case 8:
          message.defaultPath = reader.string();
          break;
        case 9:
          message.jsonMessageRecieved = reader.string();
          break;
        case 10:
          message.jsonMessageHandler = reader.string();
          break;
        case 11:
          message.protoMessageRecieved = reader.string();
          break;
        case 12:
          message.protoMessageHandler = reader.string();
          break;
        case 13:
          message.themes = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Constants {
    const message = { ...baseConstants } as Constants;
    if (object.version !== undefined && object.version !== null) {
      message.version = String(object.version);
    }
    if (object.build !== undefined && object.build !== null) {
      message.build = String(object.build);
    }
    if (object.cli !== undefined && object.cli !== null) {
      message.cli = String(object.cli);
    }
    if (object.bundlePath !== undefined && object.bundlePath !== null) {
      message.bundlePath = String(object.bundlePath);
    }
    if (object.remote !== undefined && object.remote !== null) {
      message.remote = String(object.remote);
    }
    if (object.home !== undefined && object.home !== null) {
      message.home = String(object.home);
    }
    if (object.user !== undefined && object.user !== null) {
      message.user = String(object.user);
    }
    if (object.defaultPath !== undefined && object.defaultPath !== null) {
      message.defaultPath = String(object.defaultPath);
    }
    if (
      object.jsonMessageRecieved !== undefined &&
      object.jsonMessageRecieved !== null
    ) {
      message.jsonMessageRecieved = String(object.jsonMessageRecieved);
    }
    if (
      object.jsonMessageHandler !== undefined &&
      object.jsonMessageHandler !== null
    ) {
      message.jsonMessageHandler = String(object.jsonMessageHandler);
    }
    if (
      object.protoMessageRecieved !== undefined &&
      object.protoMessageRecieved !== null
    ) {
      message.protoMessageRecieved = String(object.protoMessageRecieved);
    }
    if (
      object.protoMessageHandler !== undefined &&
      object.protoMessageHandler !== null
    ) {
      message.protoMessageHandler = String(object.protoMessageHandler);
    }
    if (object.themes !== undefined && object.themes !== null) {
      message.themes = String(object.themes);
    }
    return message;
  },

  toJSON(message: Constants): unknown {
    const obj: any = {};
    message.version !== undefined && (obj.version = message.version);
    message.build !== undefined && (obj.build = message.build);
    message.cli !== undefined && (obj.cli = message.cli);
    message.bundlePath !== undefined && (obj.bundlePath = message.bundlePath);
    message.remote !== undefined && (obj.remote = message.remote);
    message.home !== undefined && (obj.home = message.home);
    message.user !== undefined && (obj.user = message.user);
    message.defaultPath !== undefined &&
      (obj.defaultPath = message.defaultPath);
    message.jsonMessageRecieved !== undefined &&
      (obj.jsonMessageRecieved = message.jsonMessageRecieved);
    message.jsonMessageHandler !== undefined &&
      (obj.jsonMessageHandler = message.jsonMessageHandler);
    message.protoMessageRecieved !== undefined &&
      (obj.protoMessageRecieved = message.protoMessageRecieved);
    message.protoMessageHandler !== undefined &&
      (obj.protoMessageHandler = message.protoMessageHandler);
    message.themes !== undefined && (obj.themes = message.themes);
    return obj;
  },

  fromPartial(object: DeepPartial<Constants>): Constants {
    const message = { ...baseConstants } as Constants;
    if (object.version !== undefined && object.version !== null) {
      message.version = object.version;
    }
    if (object.build !== undefined && object.build !== null) {
      message.build = object.build;
    }
    if (object.cli !== undefined && object.cli !== null) {
      message.cli = object.cli;
    }
    if (object.bundlePath !== undefined && object.bundlePath !== null) {
      message.bundlePath = object.bundlePath;
    }
    if (object.remote !== undefined && object.remote !== null) {
      message.remote = object.remote;
    }
    if (object.home !== undefined && object.home !== null) {
      message.home = object.home;
    }
    if (object.user !== undefined && object.user !== null) {
      message.user = object.user;
    }
    if (object.defaultPath !== undefined && object.defaultPath !== null) {
      message.defaultPath = object.defaultPath;
    }
    if (
      object.jsonMessageRecieved !== undefined &&
      object.jsonMessageRecieved !== null
    ) {
      message.jsonMessageRecieved = object.jsonMessageRecieved;
    }
    if (
      object.jsonMessageHandler !== undefined &&
      object.jsonMessageHandler !== null
    ) {
      message.jsonMessageHandler = object.jsonMessageHandler;
    }
    if (
      object.protoMessageRecieved !== undefined &&
      object.protoMessageRecieved !== null
    ) {
      message.protoMessageRecieved = object.protoMessageRecieved;
    }
    if (
      object.protoMessageHandler !== undefined &&
      object.protoMessageHandler !== null
    ) {
      message.protoMessageHandler = object.protoMessageHandler;
    }
    if (object.themes !== undefined && object.themes !== null) {
      message.themes = object.themes;
    }
    return message;
  },
};

declare var self: any | undefined;
declare var window: any | undefined;
var globalThis: any = (() => {
  if (typeof globalThis !== "undefined") return globalThis;
  if (typeof self !== "undefined") return self;
  if (typeof window !== "undefined") return window;
  if (typeof global !== "undefined") return global;
  throw "Unable to locate global object";
})();

const atob: (b64: string) => string =
  globalThis.atob ||
  ((b64) => globalThis.Buffer.from(b64, "base64").toString("binary"));
function bytesFromBase64(b64: string): Uint8Array {
  const bin = atob(b64);
  const arr = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; ++i) {
    arr[i] = bin.charCodeAt(i);
  }
  return arr;
}

const btoa: (bin: string) => string =
  globalThis.btoa ||
  ((bin) => globalThis.Buffer.from(bin, "binary").toString("base64"));
function base64FromBytes(arr: Uint8Array): string {
  const bin: string[] = [];
  for (const byte of arr) {
    bin.push(String.fromCharCode(byte));
  }
  return btoa(bin.join(""));
}

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
