import logger from "loglevel";
import {
  SETTINGS,
  updateSettings,
} from "@aws/amazon-q-developer-cli-api-bindings-wrappers";
import { SpecLocationSource } from "@aws/amazon-q-developer-cli-shared/utils";
import * as loadSpec from "../src/loadSpec";
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

// vi.mock("../src/loadHelpers", () => ({
//   importSpecFromFile: vi
//     .fn()
//     .mockResolvedValue({ default: { name: "loadFromFile" } }),
//   getPrivateSpec: vi.fn().mockReturnValue(undefined),
//   isDiffVersionedSpec: vi.fn(),
// }));

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

describe("loadSpec.getSpecPath", () => {
  const cwd = "test_cwd";

  it("works", async () => {
    expect(await loadSpec.getSpecPath(ipcClient, "git", cwd)).toEqual({
      type: SpecLocationSource.GLOBAL,
      name: "git",
    });
  });

  it("works for specs containing a slash in the name", async () => {
    expect(
      await loadSpec.getSpecPath(
        ipcClient,
        "@withfig/autocomplete-tools",
        cwd,
        false,
      ),
    ).toEqual({
      type: SpecLocationSource.GLOBAL,
      name: "@withfig/autocomplete-tools",
    });
  });

  it("works for scripts containing a slash in the name", async () => {
    expect(
      await loadSpec.getSpecPath(ipcClient, "@withfig/autocomplete-tools", cwd),
    ).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "autocomplete-tools",
      path: `${cwd}/@withfig/`,
    });
  });

  it("works properly with local commands", async () => {
    expect(await loadSpec.getSpecPath(ipcClient, "./test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "test",
      path: `${cwd}/`,
    });
    expect(await loadSpec.getSpecPath(ipcClient, "~/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `~/`,
      name: "test",
    });
    expect(await loadSpec.getSpecPath(ipcClient, "/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `/`,
      name: "test",
    });
    expect(await loadSpec.getSpecPath(ipcClient, "/dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `/dir/`,
      name: "test",
    });
    expect(await loadSpec.getSpecPath(ipcClient, "~/dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `~/dir/`,
      name: "test",
    });
    expect(await loadSpec.getSpecPath(ipcClient, "./dir/test", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `${cwd}/dir/`,
      name: "test",
    });
  });

  it("works properly with ? commands", async () => {
    expect(await loadSpec.getSpecPath(ipcClient, "?", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      path: `${cwd}/`,
      name: "_shortcuts",
    });
  });

  it("works properly with + commands", async () => {
    expect(await loadSpec.getSpecPath(ipcClient, "+", cwd)).toEqual({
      type: SpecLocationSource.LOCAL,
      name: "+",
      path: "~/",
    });
  });
});

describe("loadSpec.loadFigSubcommand", () => {
  window.URL.createObjectURL = vi.fn();

  beforeEach(() => {
    (loadHelpers.isDiffVersionedSpec as Mock).mockResolvedValue(false);
    updateSettings({});
  });

  afterEach(() => {
    (loadHelpers.isDiffVersionedSpec as Mock).mockClear();
  });

  it("works with expected input", async () => {
    const result = await loadSpec.loadFigSubcommand(ipcClient, {
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
    await loadSpec.loadFigSubcommand(ipcClient, specLocation);
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
    await loadSpec.loadFigSubcommand(ipcClient, specLocation);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    updateSettings({
      [SETTINGS.DEV_COMPLETIONS_FOLDER]: devPath,
      [SETTINGS.DEV_MODE_NPM]: false,
      [SETTINGS.DEV_MODE]: true,
    });
    await loadSpec.loadFigSubcommand(ipcClient, specLocation);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    updateSettings({
      [SETTINGS.DEV_COMPLETIONS_FOLDER]: "~/some-folder/",
      [SETTINGS.DEV_MODE_NPM]: false,
      [SETTINGS.DEV_MODE]: true,
    });
    await loadSpec.loadFigSubcommand(ipcClient, specLocation);
    expect(importSpecFromFile).toHaveBeenLastCalledWith("git", devPath, logger);

    expect(loadHelpers.isDiffVersionedSpec).toHaveBeenCalledTimes(4);
  });
});

describe("loadSpec.loadSubcommandCached", () => {
  it.skip("works", async () => {
    const loadFigSubcommandSpy = vi.spyOn(loadSpec, "loadFigSubcommand");
    loadFigSubcommandSpy.mockResolvedValue({
      name: "exampleSpec",
    });

    const context: Fig.ShellContext = {
      currentWorkingDirectory: "",
      currentProcess: "",
      sshPrefix: "",
      environmentVariables: {},
    };

    await loadSpec.loadSubcommandCached(
      ipcClient,
      { name: "git", type: SpecLocationSource.LOCAL },
      context,
    );
    await loadSpec.loadSubcommandCached(
      ipcClient,
      { name: "git", type: SpecLocationSource.LOCAL },
      context,
    );
    expect(loadSpec.loadFigSubcommand).toHaveBeenCalledTimes(1);

    await loadSpec.loadSubcommandCached(
      ipcClient,
      { name: "hg", type: SpecLocationSource.LOCAL },
      context,
    );
    expect(loadFigSubcommandSpy).toHaveBeenCalledTimes(2);
  });
});
