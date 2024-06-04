import { sendRunProcessRequest } from "./requests.js";

export async function run({
  executable,
  args,
  environment,
  workingDirectory,
  terminalSessionId,
  timeout,
}: {
  executable: string;
  args: string[];
  environment?: Record<string, string | undefined>;
  workingDirectory?: string;
  terminalSessionId?: string;
  timeout?: number;
}) {
  const env = environment ?? {};
  return sendRunProcessRequest({
    executable,
    arguments: args,
    env: Object.keys(env).map((key) => ({ key, value: env[key] })),
    workingDirectory,
    terminalSessionId,
    timeout: timeout
      ? {
          nanos: Math.floor((timeout % 1000) * 1_000_000_000),
          secs: Math.floor(timeout / 1000),
        }
      : undefined,
  });
}
