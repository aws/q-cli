import logger, { Logger } from "loglevel";
import { Settings, Debugger } from "@withfig/api-bindings";
import { getVersionFromVersionedSpec } from "@fig/autocomplete-helpers";
import {
  convertSubcommand,
  initializeDefault,
  SpecMixin,
  applyMixin,
} from "@fig/autocomplete-shared";
import {
  withTimeout,
  SpecLocationSource,
  splitPath,
  ensureTrailingSlash,
} from "@internal/shared/utils";
import { Subcommand, SpecLocation } from "@internal/shared/internal";
import {
  SETTINGS,
  getSetting,
  executeCommand,
  isInDevMode,
} from "@amzn/fig-io-api-bindings-wrappers";
import { AuthClient } from "@amzn/fig-io-api-client";
import {
  importFromPublicCDN,
  publicSpecExists,
  getPrivateSpec,
  importFromPrivateCDN,
  SpecFileImport,
  getVersionFromFullFile,
  importSpecFromFile,
  getSpecInfo,
  isDiffVersionedSpec,
  preloadMixins,
  getMixinCacheKey,
  importFromLocalhost,
} from "./loadHelpers.js";
import {
  DisabledSpecError,
  MissingSpecError,
  WrongDiffVersionedSpecError,
} from "./errors.js";
import { mixinCache, specCache } from "./caches.js";

/**
 * This searches for the first directory containing a .fig/ folder in the parent directories
 */
const searchFigFolder = async (currentDirectory: string) => {
  try {
    return ensureTrailingSlash(
      await executeCommand(
        `cd ${currentDirectory} && until [[ -f .fig/autocomplete/build/_shortcuts.js ]] || [[ $PWD = $HOME ]] || [[ $PWD = "/" ]]; do cd ..; done; echo $PWD`,
      ),
    );
  } catch {
    return ensureTrailingSlash(currentDirectory);
  }
};

export const serializeSpecLocation = (location: SpecLocation): string => {
  if (location.type === SpecLocationSource.GLOBAL) {
    return `global://name=${location.name}`;
  }
  return `local://path=${location.path ?? ""}&name=${location.name}`;
};

export const getSpecPath = async (
  name: string,
  cwd: string,
  isScript?: boolean,
): Promise<SpecLocation> => {
  if (name === "?") {
    // If the user is searching for _shortcuts.js by using "?"
    const path = await searchFigFolder(cwd);
    return { name: "_shortcuts", type: SpecLocationSource.LOCAL, path };
  }

  const personalShortcutsToken =
    getSetting(SETTINGS.PERSONAL_SHORTCUTS_TOKEN) || "+";
  if (name === personalShortcutsToken) {
    return { name: "+", type: SpecLocationSource.LOCAL, path: "~/" };
  }

  const [path, basename] = splitPath(name);

  if (!isScript) {
    const type = SpecLocationSource.GLOBAL;

    const privateNamespaceId = getPrivateSpec({
      name,
      isScript: false,
    })?.namespaceId;
    if (privateNamespaceId !== undefined) {
      return { name, type, privateNamespaceId };
    }

    // If `isScript` is undefined, we are parsing the first token, and
    // any path with a / is a script.
    if (isScript === undefined) {
      // special-case: Symfony has "bin/console" which can be invoked directly
      // and should not require a user to create script completions for it
      if (name === "bin/console" || name.endsWith("/bin/console")) {
        return { name: "php/bin-console", type };
      }
      if (!path.includes("/")) {
        return { name, type };
      }
    } else if (["/", "./", "~/"].every((prefix) => !path.startsWith(prefix))) {
      return { name, type };
    }
  }

  const type = SpecLocationSource.LOCAL;
  if (path.startsWith("/") || path.startsWith("~/")) {
    return { name: basename, type, path };
  }

  const relative = path.startsWith("./") ? path.slice(2) : path;
  return { name: basename, type, path: `${cwd}/${relative}` };
};

type ResolvedSpecLocation =
  | { type: "public"; name: string }
  | { type: "private"; namespace: string; name: string };

const loadMixinCached = async (
  resolvedLocation: ResolvedSpecLocation,
  authClient: AuthClient,
): Promise<SpecMixin | undefined> => {
  if (mixinCache.size === 0) {
    await withTimeout(5000, preloadMixins(authClient));
  }

  const key = getMixinCacheKey(
    resolvedLocation.name,
    "namespace" in resolvedLocation ? resolvedLocation.namespace : undefined,
  );
  if (mixinCache.has(key)) {
    return mixinCache.get(key);
  }
  return undefined;
};

const importSpecFromLocation = async (
  specLocation: SpecLocation,
  authClient: AuthClient,
  localLogger: Logger = logger,
): Promise<{
  specFile: SpecFileImport;
  resolvedLocation?: ResolvedSpecLocation;
}> => {
  // Try loading spec from `devCompletionsFolder` first.
  const devPath = isInDevMode()
    ? (getSetting(SETTINGS.DEV_COMPLETIONS_FOLDER) as string)
    : undefined;

  const devPort = isInDevMode()
    ? getSetting(SETTINGS.DEV_COMPLETIONS_SERVER_PORT)
    : undefined;

  let specFile: SpecFileImport | undefined;
  let resolvedLocation: ResolvedSpecLocation | undefined;

  if (typeof devPort === "string" || typeof devPort === "number") {
    const { diffVersionedFile, name } = specLocation;
    specFile = await importFromLocalhost(
      diffVersionedFile ? `${name}/${diffVersionedFile}` : name,
      devPort,
    );
  }

  if (!specFile && devPath) {
    try {
      const { diffVersionedFile, name } = specLocation;
      const spec = await importSpecFromFile(
        diffVersionedFile ? `${name}/${diffVersionedFile}` : name,
        devPath,
        localLogger,
      );
      specFile = spec;
    } catch {
      // fallback to loading other specs in dev mode.
    }
  }

  if (!specFile && specLocation.type === SpecLocationSource.LOCAL) {
    // If we couldn't successfully load a dev spec try loading from specPath.
    const { name, path } = specLocation;
    const [dirname, basename] = splitPath(`${path || "~/"}${name}`);
    try {
      const privateSpecMatch = await getSpecInfo(
        basename,
        dirname,
        localLogger,
      );
      resolvedLocation = { type: "private", ...privateSpecMatch };
      specFile = await importFromPrivateCDN(privateSpecMatch, authClient);
    } catch (err) {
      specFile = await importSpecFromFile(
        basename,
        `${dirname}.fig/autocomplete/build/`,
        localLogger,
      );
    }
  } else if (!specFile) {
    const { name, diffVersionedFile: versionFileName } = specLocation;
    const privateSpecMatch = getPrivateSpec({ name, isScript: false });

    if (privateSpecMatch) {
      logger.info(`Found private spec ${privateSpecMatch}...`);
      resolvedLocation = { type: "private", ...privateSpecMatch };
      specFile = await importFromPrivateCDN(privateSpecMatch, authClient);
    } else if (await publicSpecExists(name)) {
      // If we're here, importing was successful.
      try {
        const result = await importFromPublicCDN(
          versionFileName ? `${name}/${versionFileName}` : name,
        );
        Debugger.resetDebugger();

        specFile = result;
        resolvedLocation = { type: "public", name };
      } catch (err) {
        Debugger.reportError({
          message: [
            `Autocomplete: Unable to load spec ${name} from any CDN.`,
            "Make sure you're connected to the internet",
          ],
          color: "ff0000",
        });
        throw err;
      }
    } else {
      try {
        specFile = await importSpecFromFile(
          name,
          `~/.fig/autocomplete/build/`,
          localLogger,
        );
      } catch (err) {
        /* empty */
      }
    }
  }

  if (!specFile) {
    throw new MissingSpecError("No spec found");
  }

  return { specFile, resolvedLocation };
};

const tryResolveSpecToSubcommand = async (
  spec: SpecFileImport,
  location: SpecLocation,
  authClient: AuthClient,
): Promise<Fig.Subcommand> => {
  if (typeof spec.default === "function") {
    // Handle versioned specs, either simple versioned or diff versioned.
    const cliVersion = await getVersionFromFullFile(spec, location.name);
    const subcommandOrDiffVersionInfo = await spec.default(cliVersion);

    if ("versionedSpecPath" in subcommandOrDiffVersionInfo) {
      // Handle diff versioned specs.
      const { versionedSpecPath, version } = subcommandOrDiffVersionInfo;
      const [dirname, basename] = splitPath(versionedSpecPath);
      const { specFile } = await importSpecFromLocation(
        {
          ...location,
          name: dirname.slice(0, -1),
          diffVersionedFile: basename,
        },
        authClient,
      );

      if ("versions" in specFile) {
        const result = getVersionFromVersionedSpec(
          specFile.default,
          specFile.versions,
          version,
        );
        return result.spec;
      }

      throw new WrongDiffVersionedSpecError("Invalid versioned specs file");
    }

    return subcommandOrDiffVersionInfo;
  }

  return spec.default;
};

export const loadFigSubcommand = async (
  specLocation: SpecLocation,
  authClient: AuthClient,
  context?: Fig.ShellContext,
  localLogger: Logger = logger,
): Promise<Fig.Subcommand> => {
  const { name } = specLocation;
  const location = (await isDiffVersionedSpec(name))
    ? { ...specLocation, diffVersionedFile: "index" }
    : specLocation;
  const { specFile, resolvedLocation } = await importSpecFromLocation(
    location,
    authClient,
    localLogger,
  );

  const subcommand = await tryResolveSpecToSubcommand(
    specFile,
    specLocation,
    authClient,
  );
  const mixin =
    resolvedLocation && (await loadMixinCached(resolvedLocation, authClient));
  return mixin
    ? applyMixin(
        subcommand,
        context ?? {
          currentProcess: "",
          currentWorkingDirectory: "",
          sshPrefix: "",
          environmentVariables: {},
        },
        mixin,
      )
    : subcommand;
};

export const loadSubcommandCached = async (
  specLocation: SpecLocation,
  authClient: AuthClient,
  context?: Fig.ShellContext,
  localLogger: Logger = logger,
): Promise<Subcommand> => {
  const { name, type: source } = specLocation;
  const path =
    specLocation.type === SpecLocationSource.LOCAL ? specLocation.path : "";

  // Do not load completion spec for commands that are 'disabled' by user
  const disabledSpecs =
    <string[]>getSetting(SETTINGS.DISABLE_FOR_COMMANDS) || [];
  if (disabledSpecs.includes(name)) {
    localLogger.info(`Not getting path for disabled spec ${name}`);
    throw new DisabledSpecError("Command requested disabled completion spec");
  }

  const key = [source, path || "", name].join(",");
  if (getSetting(SETTINGS.DEV_MODE_NPM_INVALIDATE_CACHE)) {
    specCache.clear();
    Settings.set(SETTINGS.DEV_MODE_NPM_INVALIDATE_CACHE, false);
  } else if (!getSetting(SETTINGS.DEV_MODE_NPM) && specCache.has(key)) {
    return specCache.get(key) as Subcommand;
  }

  const subcommand = await withTimeout(
    5000,
    loadFigSubcommand(specLocation, authClient, context, localLogger),
  );
  const converted = convertSubcommand(subcommand, initializeDefault);
  specCache.set(key, converted);
  return converted;
};
