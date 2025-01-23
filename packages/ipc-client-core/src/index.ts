import type {
  InsertTextRequest,
  InterceptRequest,
} from "@aws/amazon-q-developer-cli-proto/figterm";
import type { RunProcessRequest } from "@aws/amazon-q-developer-cli-proto/remote";
import type {
  RunProcessResponse,
  EditBufferChangedNotification,
} from "@aws/amazon-q-developer-cli-proto/fig";
import type {
  EditBufferHook,
  InterceptedKeyHook,
  PostExecHook,
  PreExecHook,
  PromptHook,
} from "@aws/amazon-q-developer-cli-proto/local";
import { State } from "./state.js";
import { Settings } from "./settings.js";
import type { UnsubscribeFunction } from "emittery";

export type {
  InsertTextRequest,
  InterceptRequest,
  RunProcessRequest,
  RunProcessResponse,
  EditBufferChangedNotification,
  InterceptedKeyHook,
  PostExecHook,
  PreExecHook,
  PromptHook,
  State,
  Settings,
};

export interface IpcClient {
  state?: State;
  settings?: Settings;

  // Request
  insertText: (sessionId: string, request: InsertTextRequest) => void;
  intercept: (sessionId: string, request: InterceptRequest) => void;

  // Request -> Response
  runProcess: (
    sessionId: string,
    request: RunProcessRequest,
  ) => Promise<RunProcessResponse>;

  ping: () => Promise<void>;

  // Notifications
  onEditBufferChange: (
    callback: (notification: EditBufferHook) => void | Promise<void>,
  ) => UnsubscribeFunction;

  onPrompt: (
    callback: (notification: PromptHook) => void | Promise<void>,
  ) => UnsubscribeFunction;

  onPreExec: (
    callback: (notification: PreExecHook) => void | Promise<void>,
  ) => UnsubscribeFunction;

  onPostExec: (
    callback: (notification: PostExecHook) => void | Promise<void>,
  ) => UnsubscribeFunction;

  onInterceptedKey: (
    callback: (notification: InterceptedKeyHook) => void | Promise<void>,
  ) => UnsubscribeFunction;

  isActive: (timeout?: number | undefined) => Promise<boolean>;
}
