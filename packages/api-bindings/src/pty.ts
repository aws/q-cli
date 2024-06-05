import {
  sendPseudoterminalExecuteRequest,
  sendPseudoterminalWriteRequest,
} from "./requests.js";

/**
 * @deprecated
 */
export async function execute(
  command: string,
  options?: {
    env?: Record<string, string>;
    directory?: string;
    isPipelined?: boolean;
    backgroundJob?: boolean;
    terminalSessionId?: string;
  },
) {
  return sendPseudoterminalExecuteRequest({
    command,
    isPipelined: options?.isPipelined ?? false,
    backgroundJob: options?.backgroundJob ?? true,
    workingDirectory: options?.directory,
    env: [],
    terminalSessionId: options?.terminalSessionId,
  });
}

/**
 * @deprecated
 */
export async function write(text: string): Promise<void> {
  return sendPseudoterminalWriteRequest({
    input: {
      $case: "text",
      text,
    },
  });
}
