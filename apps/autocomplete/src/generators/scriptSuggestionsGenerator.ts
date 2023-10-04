import logger from "loglevel";
import { executeCommandInDir } from "@amzn/fig-io-api-bindings-wrappers";
import { runPipingConsoleMethods } from "../utils";
import {
  runCachedGenerator,
  GeneratorContext,
  haveContextForGenerator,
} from "./helpers";

export async function getScriptSuggestions(
  generator: Fig.Generator,
  context: GeneratorContext,
  defaultTimeout: number,
): Promise<Fig.Suggestion[]> {
  const { script, postProcess, splitOn } = generator;
  if (!script) {
    return [];
  }

  if (!haveContextForGenerator(context)) {
    logger.info("Don't have context for custom generator");
    return [];
  }

  try {
    const { isDangerous, tokenArray, currentWorkingDirectory } = context;
    // A script can either be a string or a function that returns a string.
    // If the script is a function, run it, and get the output string.
    const scriptToRun =
      script && typeof script === "function"
        ? runPipingConsoleMethods(() => script(tokenArray))
        : script;

    if (!scriptToRun) {
      return [];
    }
    const data = await runCachedGenerator(
      generator,
      context,
      () =>
        executeCommandInDir(
          scriptToRun,
          currentWorkingDirectory,
          "",
          // If both timeouts are specified use the greatest of the two timeouts.
          generator.scriptTimeout !== undefined &&
            generator.scriptTimeout > defaultTimeout
            ? generator.scriptTimeout
            : defaultTimeout,
        ),
      generator.cache?.cacheKey ?? scriptToRun,
    );

    let result: Array<Fig.Suggestion | string> = [];

    // If we have a splitOn function
    if (splitOn) {
      result = data.trim() === "" ? [] : data.trim().split(splitOn);
    } else if (postProcess) {
      // If we have a post process function
      // The function takes one input and outputs an array
      runPipingConsoleMethods(() => {
        result = postProcess(data, tokenArray);
      });
      result = result.filter(
        (item) => item && (typeof item === "string" || !!item.name),
      );
    }

    // Generator can either output an array of strings or an array of suggestion objects.
    return result.map((item) =>
      typeof item === "string"
        ? { type: "arg", name: item, insertValue: item, isDangerous }
        : { ...item, type: item.type || "arg" },
    );
  } catch (e) {
    logger.error("we had an error with the script generator", e);
    return [];
  }
}
