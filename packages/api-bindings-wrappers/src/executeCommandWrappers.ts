import { Process } from "@withfig/api-bindings";
import { withTimeout } from "@internal/shared/utils";
import { createErrorInstance } from "@internal/shared/errors";
import logger from "loglevel";
import { executeCommand, cleanOutput } from "./executeCommand.js";
import { fread } from "./fs.js";

export const LoginShellError = createErrorInstance("LoginShellError");

const DONE_SOURCING_OSC = "\u001b]697;DoneSourcing\u0007";

let etcShells: Promise<string[]> | undefined;

const getShellExecutable = async (shellName: string) => {
  if (!etcShells) {
    etcShells = fread("/etc/shells").then((shells) =>
      shells
        .split("\n")
        .map((line) => line.trim())
        .filter((line) => line && !line.startsWith("#")),
    );
  }

  try {
    return (
      (await etcShells).find((shell) => shell.includes(shellName)) ??
      (await executeCommand(`which ${shellName}`))
    );
  } catch (_) {
    return undefined;
  }
};

export const executeLoginShell = async ({
  command,
  executable,
  shell,
  timeout,
}: {
  command: string;
  executable?: string;
  shell?: string;
  timeout?: number;
}): Promise<string> => {
  let exe = executable;
  if (!exe) {
    if (!shell) {
      throw new LoginShellError("Must pass shell or executable");
    }
    exe = await getShellExecutable(shell);
    if (!exe) {
      throw new LoginShellError(`Could not find executable for ${shell}`);
    }
  }
  const flags = window.fig.constants?.os === "linux" ? "-lc" : "-lic";
  const rawCommand = `${exe} ${flags} '${command}'`;

  const process = Process.run({
    executable: exe,
    args: [flags, command],
    terminalSessionId: window.globalTerminalSessionId,
  });

  try {
    logger.info(`About to run login shell command '${command}'`, {
      separateProcess: Boolean(window.f.Process),
      shell: exe,
    });
    const start = performance.now();
    const result = await withTimeout(
      timeout ?? 5000,
      process.then((output) => cleanOutput(rawCommand, output)),
    );
    const idx =
      result.lastIndexOf(DONE_SOURCING_OSC) + DONE_SOURCING_OSC.length;
    const trimmed = result.slice(idx);
    const end = performance.now();
    logger.info(`Result of login shell command '${command}'`, {
      result: trimmed,
      time: end - start,
    });
    return trimmed;
  } catch (err) {
    logger.error(`Error running login shell command '${command}'`, { err });
    throw err;
  }
};

export const executeCommandInDir = (
  command: string,
  dir: string,
  sshContextString?: string,
  timeout?: number,
): Promise<string> => {
  const inputDir = dir.replace(/[\s()[\]]/g, "\\$&");
  let commandString = `cd ${inputDir} && ${command} | cat`;

  if (sshContextString) {
    commandString = commandString.replace(/'/g, `'"'"'`);
    commandString = `${sshContextString} '${commandString}'`;
  }

  return executeCommand(
    commandString,
    timeout && timeout > 0 ? timeout : undefined,
  );
};

export const executeShellCommand = (
  cmd: string,
  cwd: string = window.globalCWD,
): Promise<string> => {
  try {
    return executeCommandInDir(cmd, cwd, window.globalSSHString);
  } catch (err) {
    logger.error(err);
    return new Promise((resolve) => {
      resolve("");
    });
  }
};
