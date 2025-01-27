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
};

export interface IpcClient {
  // Request
  insertText: (sessionId: string, request: InsertTextRequest) => void;
  intercept: (sessionId: string, request: InterceptRequest) => void;

  // Request -> Response
  runProcess: (
    sessionId: string,
    request: RunProcessRequest,
  ) => Promise<RunProcessResponse>;

  // Notifications
  onEditBufferChange: (
    callback: (notification: EditBufferHook) => void,
  ) => void;

  onPrompt: (callback: (notification: PromptHook) => void) => void;

  onPreExec: (callback: (notification: PreExecHook) => void) => void;

  onPostExec: (callback: (notification: PostExecHook) => void) => void;

  onInterceptedKey: (
    callback: (notification: InterceptedKeyHook) => void,
  ) => void;
}
