/**
 *
 * NOTE: this is intended to be separate because executeCommand
 * will often be mocked during testing of functions that call it.
 * If it gets bundled in the same file as the functions that call it
 * jest is not able to mock it (because of esm restrictions).
 *
 */
import { withTimeout } from "@internal/shared/utils";
import { PTY } from "@withfig/api-bindings";
import logger from "loglevel";

export const cleanOutput = (
  command: string,
  {
    stdout,
    stderr,
    exitCode,
  }: {
    stdout: string;
    stderr?: string;
    exitCode?: number;
  },
) => {
  if (!stderr && exitCode === -2) {
    logger.info(
      `Could not determine exit code of pipelined command: ${command}`,
    );
  } else if (stderr || (exitCode !== undefined && exitCode !== 0)) {
    logger.warn(
      `Command ${command} exited with exit code ${exitCode}: ${stderr}`,
    );
  }
  return stdout
    .replace(/\r\n/g, "\n") // Replace carriage returns with just a normal return
    .replace(/\033\[\?25h/g, "") // removes cursor character if present
    .replace(/^\n+/, "") // strips new lines from start of output
    .replace(/\n+$/, ""); // strips new lines from end of output
};

export const executeCommand = async (
  command: string,
  timeout = window.fig.constants?.os === "windows" ? 20000 : 5000,
): Promise<string> => {
  try {
    logger.info(`About to run shell command '${command}'`);
    const start = performance.now();
    const result = await withTimeout(
      timeout,
      PTY.execute(command, {
        terminalSessionId: window.globalTerminalSessionId,
      }).then((output) => cleanOutput(command, output)),
    );
    const end = performance.now();
    logger.info(`Result of shell command '${command}'`, {
      result,
      time: end - start,
    });
    return result;
  } catch (err) {
    logger.error(`Error running shell command '${command}'`, { err });
    throw err;
  }
};
