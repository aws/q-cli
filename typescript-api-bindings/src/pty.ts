import {
  sendPseudoterminalExecuteRequest,
  sendPseudoterminalWriteRequest,
} from './requests';

export async function execute(
  command: string,
  options:
    | {
        env: Record<string, string> | undefined;
        directory: string | undefined;
        isPipelined: boolean | undefined;
        backgroundJob: boolean | undefined;
      }
    | undefined
) {
  return sendPseudoterminalExecuteRequest({
    command: command,
    isPipelined: options?.isPipelined ?? false,
    backgroundJob: options?.backgroundJob ?? true,
    workingDirectory: options?.directory,
    env: [],
  });
}

export async function write(text: string): Promise<void> {
  return sendPseudoterminalWriteRequest({
    input: {
      $case: 'text',
      text: text,
    },
  });
}
