import logger from "loglevel";
import {
  SETTINGS,
  updateSettings,
} from "@aws/amazon-q-developer-cli-api-bindings-wrappers";
import { SpecLocationSource } from "@aws/amazon-q-developer-cli-shared/utils";
import {
  getSpecPath,
  loadFigSubcommand,
  loadSubcommandCached,
} from "../src/loadSpec";
import * as loadHelpers from "../src/loadHelpers";
import {
  expect,
  it,
  beforeAll,
  describe,
  beforeEach,
  vi,
  Mock,
  afterEach,
} from "vitest";
import { IpcClient } from "@aws/amazon-q-developer-cli-ipc-client-core";
import { create } from "@bufbuild/protobuf";
import { RunProcessResponseSchema } from "@aws/amazon-q-developer-cli-proto/fig";

const { importSpecFromFile } = loadHelpers;

vi.mock("../src/loadHelpers", () => ({
  importSpecFromFile: vi
    .fn()
    .mockResolvedValue({ default: { name: "loadFromFile" } }),
  getPrivateSpec: vi.fn().mockReturnValue(undefined),
  isDiffVersionedSpec: vi.fn(),
}));

// TODO: remove this statement and move fig dir to shared
const FIG_DIR = "~/.fig";

const ipcClient = {
  runProcess: async (_sessionId, _request) => {
    return create(RunProcessResponseSchema, {
      exitCode: 0,
      stdout: "test_cwd",
      stderr: "",
    });
  },
} as IpcClient;

beforeAll(() => {
  updateSettings({});
});

describe("getSpecPath", () => {
  const cwd = "test_cwd";

  it("works", async () => {
    expect(await getSpecPath(ipcClient, "git", cwd)).toEqual({
      type: SpecLocationSource.GLOBAL,
      name: "git",
    });
  });

  it("works for specs containing a slash in the name", async () => {
    expect(
      await getSpecPath(ipcClient, "@withfig/autocomplete-tools", cwd, false),
    ).toEqual({
      type: SpecLocationSource.GLOBAL,
      name: "@withfig/autocomplete-tools",
    });
  });

  it("works for scripts containing a slash in the name", async () => {
    expect(
      await getSpecPath(ipcClient, "@withfig/autocomplete-tools", cwd),
    ).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "autocomplete-tools",
      path: `${cwd}/@withfig/`,
    });
  });

  it("works properly with local commands", async () => {
    expect(await getSpecPath(ipcClient, "./test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "test",
      path: `${cwd}/`,
    });
    expect(await getSpecPath(ipcClient, "~/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `~/`,
      name: "test",
    });
    expect(await getSpecPath(ipcClient, "/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `/`,
      name: "test",
    });
    expect(await getSpecPath(ipcClient, "/dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `/dir/`,
      name: "test",
    });
    expect(await getSpecPath(ipcClient, "~/dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `~/dir/`,
      name: "test",
    });
    expect(await getSpecPath(ipcClient, "./dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `${cwd}/dir/`,
      name: "test",
    });
  });

  it("works properly with ? commands", async () => {
    expect(await getSpecPath(ipcClient, "?", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `${cwd}/`,
      name: "_shortcuts",
    });
  });

  it("works properly with + commands", async () => {
    expect(await getSpecPath(ipcClient, "+", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "+",
      path: "~/",
    });
  });
});

describe("loadFigSubcommand", () => {
  window.URL.createObjectURL = vi.fn();

  beforeEach(() => {
    (loadHelpers.isDiffVersionedSpec as Mock).mockResolvedValue(false);
    updateSettings({});
  });

  afterEach(() => {
    (loadHelpers.isDiffVersionedSpec as Mock).mockClear();
  });

  it("works with expected input", async () => {
    const result = await loadFigSubcommand(ipcClient, {
      name: "path",
      type: SpecLocationSource.LOCAL,
    });
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
    await loadFigSubcommand(ipcClient, specLocation);
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
    await loadFigSubcommand(ipcClient, specLocation);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    updateSettings({
      [SETTINGS.DEV_COMPLETIONS_FOLDER]: devPath,
      [SETTINGS.DEV_MODE_NPM]: false,
      [SETTINGS.DEV_MODE]: true,
    });
    await loadFigSubcommand(ipcClient, specLocation);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    updateSettings({
      [SETTINGS.DEV_COMPLETIONS_FOLDER]: "~/some-folder/",
      [SETTINGS.DEV_MODE_NPM]: false,
      [SETTINGS.DEV_MODE]: true,
    });
    await loadFigSubcommand(ipcClient, specLocation);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    expect(loadHelpers.isDiffVersionedSpec).toHaveBeenCalledTimes(4);
  });
});

describe("loadSubcommandCached", () => {
  // This is broken right now...
  it.todo("works", async () => {
    const oldLoadSpec = loadFigSubcommand;
    (loadFigSubcommand as Mock) = vi.fn();
    (loadFigSubcommand as Mock).mockResolvedValue({ name: "exampleSpec" });
    const context: Fig.ShellContext = {
      currentWorkingDirectory: "",
      currentProcess: "",
      sshPrefix: "",
      environmentVariables: {},
    };

    await loadSubcommandCached(
      ipcClient,
      { name: "git", type: SpecLocationSource.LOCAL },
      context,
    );
    await loadSubcommandCached(
      ipcClient,
      { name: "git", type: SpecLocationSource.LOCAL },
      context,
    );
    expect(loadFigSubcommand).toHaveBeenCalledTimes(1);

    await loadSubcommandCached(
      ipcClient,
      { name: "hg", type: SpecLocationSource.LOCAL },
      context,
    );
    expect(loadFigSubcommand).toHaveBeenCalledTimes(2);
    (loadFigSubcommand as unknown) = oldLoadSpec;
  });
});
