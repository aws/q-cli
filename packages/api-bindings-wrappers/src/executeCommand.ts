/**
 * NOTE: this is intended to be separate because executeCommand
 * will often be mocked during testing of functions that call it.
 * If it gets bundled in the same file as the functions that call it
 * vitest is not able to mock it (because of esm restrictions).
 */
import { withTimeout } from "@aws/amazon-q-developer-cli-shared/utils";
import logger from "loglevel";
import { IpcClient } from "../../ipc-client-core/dist/index.js";
import {
  DurationSchema,
  EnvironmentVariableSchema,
} from "@aws/amazon-q-developer-cli-proto/fig_common";
import { RunProcessRequestSchema } from "@aws/amazon-q-developer-cli-proto/remote";
import { create } from "@bufbuild/protobuf";

export const cleanOutput = (output: string) =>
  output
    .replace(/\r\n/g, "\n") // Replace carriage returns with just a normal return
    // eslint-disable-next-line no-control-regex
    .replace(/\x1b\[\?25h/g, "") // removes cursor character if present
    .replace(/^\n+/, "") // strips new lines from start of output
    .replace(/\n+$/, ""); // strips new lines from end of output

export const executeCommandTimeout = async (
  ipcClient: IpcClient,
  input: Fig.ExecuteCommandInput,
  timeout = window?.fig?.constants?.os === "windows" ? 20000 : 5000,
): Promise<Fig.ExecuteCommandOutput> => {
  const command = [input.command, ...(input.args ?? [])].join(" ");
  try {
    logger.info(`About to run shell command '${command}'`);
    const start = performance.now();
    const result = await withTimeout(
      Math.max(timeout, input.timeout ?? 0),
      ipcClient.runProcess(
        window.globalTerminalSessionId ?? "",
        create(RunProcessRequestSchema, {
          executable: input.command,
          arguments: input.args,
          env: Object.entries(input.env ?? {}).map(([key, value]) => {
            return create(EnvironmentVariableSchema, {
              key,
              value,
            });
          }),
          workingDirectory: input.cwd,
          timeout: create(DurationSchema, {
            secs: BigInt(Math.floor(timeout / 1000)),
            nanos: (timeout % 1000) * 1000000,
          }),
        }),
      ),
    );
    const end = performance.now();
    logger.info(`Result of shell command '${command}'`, {
      result,
      time: end - start,
    });

    const cleanStdout = cleanOutput(result.stdout);
    const cleanStderr = cleanOutput(result.stderr);

    if (result.exitCode !== 0) {
      logger.warn(
        `Command ${command} exited with exit code ${result.exitCode}: ${cleanStderr}`,
      );
    }
    return {
      status: result.exitCode,
      stdout: cleanStdout,
      stderr: cleanStderr,
    };
  } catch (err) {
    logger.error(`Error running shell command '${command}'`, { err });
    throw err;
  }
};
