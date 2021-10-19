import { sendPseudoterminalExecuteRequest, sendPseudoterminalWriteRequest } from "./requests";

const execute = async (
    command: string,
    options: {
      env: Record<string, string> | undefined,
      directory: string | undefined,
      isPipelined: boolean | undefined,
      backgroundJob: boolean | undefined,
    } | undefined
    ) =>
    sendPseudoterminalExecuteRequest({
      command: command,
      isPipelined: options?.isPipelined ?? false,
      backgroundJob: options?.backgroundJob ?? true,
      workingDirectory: options?.directory,
      env: []
    });
    
const write = async (
  text: string,
): Promise<void> => sendPseudoterminalWriteRequest({
  input: {
    $case: "text",
    text: text
  }
})

const PTY = { execute, write };

export default PTY;