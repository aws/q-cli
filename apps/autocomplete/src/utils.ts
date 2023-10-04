import { fs } from "@withfig/api-bindings";
import util from "util";
import { isInDevMode } from "@amzn/fig-io-api-bindings-wrappers";

// Logging functions
const DEFAULT_CONSOLE = {
  log: console.log,
  warn: console.warn,
  error: console.error,
};

export const FIG_DIR = window.fig.constants?.figDotDir ?? "~/.fig";

const NEW_LOG_FN = (...content: unknown[]) => {
  fs.append(
    `${FIG_DIR}/logs/specs.log`,
    `\n${util.format(...content)}`,
  ).finally(() => DEFAULT_CONSOLE.warn("SPEC LOG:", util.format(...content)));
};

export function runPipingConsoleMethods<T>(fn: () => T) {
  try {
    pipeConsoleMethods();
    return fn();
  } finally {
    restoreConsoleMethods();
  }
}

export function pipeConsoleMethods() {
  if (isInDevMode()) {
    console.log = NEW_LOG_FN;
    console.warn = NEW_LOG_FN;
    console.error = NEW_LOG_FN;
  }
}

export function restoreConsoleMethods() {
  if (isInDevMode()) {
    console.log = DEFAULT_CONSOLE.log;
    console.warn = DEFAULT_CONSOLE.warn;
    console.error = DEFAULT_CONSOLE.error;
  }
}
