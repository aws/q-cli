import logger from "loglevel";
import { SETTINGS, updateSettings } from "@amzn/fig-io-api-bindings-wrappers";
import { SpecLocationSource } from "@internal/shared/utils";
import { makeAuthClient } from "@amzn/fig-io-api-client";
import {
  getSpecPath,
  loadFigSubcommand,
  loadSubcommandCached,
} from "../src/loadSpec";
import * as loadHelpers from "../src/loadHelpers";

const { importSpecFromFile } = loadHelpers;

jest.mock("../src/loadHelpers", () => ({
  importSpecFromFile: jest
    .fn()
    .mockResolvedValue({ default: { name: "loadFromFile" } }),
  getPrivateSpec: jest.fn().mockReturnValue(undefined),
  isDiffVersionedSpec: jest.fn(),
}));

jest.mock("@amzn/fig-io-api-bindings-wrappers", () => ({
  ...jest.requireActual("@amzn/fig-io-api-bindings-wrappers"),
  executeCommand: jest.fn(),
}));

// TODO(fedeci): remove this statement and move fig dir to shared
const FIG_DIR = "~/.fig";

beforeAll(() => {
  updateSettings({});
});

describe("getSpecPath", () => {
  const cwd = "test_cwd";

  it("works", async () => {
    expect(await getSpecPath("git", cwd)).toEqual({
      type: SpecLocationSource.GLOBAL,
      name: "git",
    });
  });

  it("works for specs containing a slash in the name", async () => {
    expect(
      await getSpecPath("@withfig/autocomplete-tools", cwd, false),
    ).toEqual({
      type: SpecLocationSource.GLOBAL,
      name: "@withfig/autocomplete-tools",
    });
  });

  it("works for scripts containing a slash in the name", async () => {
    expect(await getSpecPath("@withfig/autocomplete-tools", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "autocomplete-tools",
      path: `${cwd}/@withfig/`,
    });
  });

  it("works properly with local commands", async () => {
    expect(await getSpecPath("./test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "test",
      path: `${cwd}/`,
    });
    expect(await getSpecPath("~/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `~/`,
      name: "test",
    });
    expect(await getSpecPath("/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `/`,
      name: "test",
    });
    expect(await getSpecPath("/dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `/dir/`,
      name: "test",
    });
    expect(await getSpecPath("~/dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `~/dir/`,
      name: "test",
    });
    expect(await getSpecPath("./dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `${cwd}/dir/`,
      name: "test",
    });
  });

  it("works properly with ? commands", async () => {
    expect(await getSpecPath("?", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `${cwd}/`,
      name: "_shortcuts",
    });
  });

  it("works properly with + commands", async () => {
    expect(await getSpecPath("+", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "+",
      path: "~/",
    });
  });
});

const authClient = makeAuthClient({ os: "macos" });

describe("loadFigSubcommand", () => {
  window.URL.createObjectURL = jest.fn();

  beforeEach(() => {
    (loadHelpers.isDiffVersionedSpec as jest.Mock).mockResolvedValue(false);
    updateSettings({});
  });

  afterEach(() => {
    (loadHelpers.isDiffVersionedSpec as jest.Mock).mockClear();
  });

  it("works with expected input", async () => {
    const result = await loadFigSubcommand(
      { name: "path", type: SpecLocationSource.LOCAL },
      authClient,
    );
    expect(loadHelpers.isDiffVersionedSpec).toHaveBeenCalledTimes(1);
    expect(result.name).toBe("loadFromFile");
  });

  it("works in dev mode", async () => {
    const devPath = "~/some-folder/";
    const specLocation: Fig.SpecLocation = {
      name: "git",
      type: SpecLocationSource.LOCAL,
    };

    updateSettings({
      [SETTINGS.DEV_COMPLETIONS_FOLDER]: devPath,
      [SETTINGS.DEV_MODE_NPM]: false,
      [SETTINGS.DEV_MODE]: false,
    });
    await loadFigSubcommand(specLocation, authClient);
    expect(importSpecFromFile).toHaveBeenLastCalledWith(
      "git",
      `${FIG_DIR}/autocomplete/build/`,
      logger,
    );

    updateSettings({
      [SETTINGS.DEV_COMPLETIONS_FOLDER]: devPath,
      [SETTINGS.DEV_MODE_NPM]: true,
      [SETTINGS.DEV_MODE]: false,
    });
    await loadFigSubcommand(specLocation, authClient);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    updateSettings({
      [SETTINGS.DEV_COMPLETIONS_FOLDER]: devPath,
      [SETTINGS.DEV_MODE_NPM]: false,
      [SETTINGS.DEV_MODE]: true,
    });
    await loadFigSubcommand(specLocation, authClient);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    updateSettings({
      [SETTINGS.DEV_COMPLETIONS_FOLDER]: "~/some-folder/",
      [SETTINGS.DEV_MODE_NPM]: false,
      [SETTINGS.DEV_MODE]: true,
    });
    await loadFigSubcommand(specLocation, authClient);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    expect(loadHelpers.isDiffVersionedSpec).toHaveBeenCalledTimes(4);
  });
});

describe("loadSubcommandCached", () => {
  it("works", async () => {
    const oldLoadSpec = loadFigSubcommand;
    (loadFigSubcommand as jest.Mock) = jest.fn();
    (loadFigSubcommand as jest.Mock).mockResolvedValue({ name: "exampleSpec" });
    const context: Fig.ShellContext = {
      currentWorkingDirectory: "",
      currentProcess: "",
      sshPrefix: "",
      environmentVariables: {},
    };
    await loadSubcommandCached(
      { name: "git", type: SpecLocationSource.LOCAL },
      authClient,
      context,
    );
    await loadSubcommandCached(
      { name: "git", type: SpecLocationSource.LOCAL },
      authClient,
      context,
    );
    expect(loadFigSubcommand).toHaveBeenCalledTimes(1);

    await loadSubcommandCached(
      { name: "hg", type: SpecLocationSource.LOCAL },
      authClient,
      context,
    );
    expect(loadFigSubcommand).toHaveBeenCalledTimes(2);
    (loadFigSubcommand as unknown) = oldLoadSpec;
  });
});
