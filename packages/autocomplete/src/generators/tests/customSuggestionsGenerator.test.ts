import { Annotation } from "@aws/amazon-q-developer-cli-autocomplete-parser";
import {
  MockInstance,
  afterEach,
  beforeAll,
  describe,
  expect,
  it,
  vi,
} from "vitest";
import { getCustomSuggestions } from "../customSuggestionsGenerator";
import * as helpers from "../helpers";
import { GeneratorContext } from "../helpers";
import { create } from "@bufbuild/protobuf";
import { RunProcessResponseSchema } from "@aws/amazon-q-developer-cli-proto/fig";
import { IpcClient } from "@aws/amazon-q-developer-cli-ipc-client-core";

const context: GeneratorContext = {
  annotations: [] as Annotation[],
  tokenArray: [] as string[],
  currentWorkingDirectory: "/",
  currentProcess: "zsh",
  sshPrefix: "",
  searchTerm: "",
  environmentVariables: {},
};

describe("getCustomSuggestions", () => {
  const ipcClient = {
    runProcess: vi.fn(async (_sessionId, _request) => {
      return create(RunProcessResponseSchema, {
        exitCode: 0,
        stdout: "a/\nx\nc/\nl",
        stderr: "",
      });
    }),
  } as Partial<IpcClient> as IpcClient;

  let runCachedGenerator: MockInstance;

  beforeAll(() => {
    runCachedGenerator = vi.spyOn(helpers, "runCachedGenerator");
  });

  afterEach(() => {
    runCachedGenerator.mockClear();
  });

  it("should return the result", async () => {
    expect(
      await getCustomSuggestions(
        ipcClient,
        {
          custom: () => Promise.resolve([{ name: "hello" }, { name: "world" }]),
        },
        context,
      ),
    ).toEqual([
      { name: "hello", type: "arg" },
      { name: "world", type: "arg" },
    ]);
  });

  it("should return the result and infer type", async () => {
    expect(
      await getCustomSuggestions(
        ipcClient,
        {
          custom: () =>
            Promise.resolve([
              { name: "hello", type: "shortcut" },
              { name: "world", type: "folder" },
            ]),
        },
        context,
      ),
    ).toEqual([
      { name: "hello", type: "shortcut" },
      { name: "world", type: "folder" },
    ]);
  });

  it("should call runCachedGenerator", async () => {
    await getCustomSuggestions(
      ipcClient,
      {
        custom: () => Promise.resolve([{ name: "hello" }, { name: "world" }]),
      },
      context,
    );

    expect(runCachedGenerator).toHaveBeenCalled();
  });

  it("should call runCachedGenerator and the custom function", async () => {
    const custom = vi
      .fn()
      .mockResolvedValue([{ name: "hello" }, { name: "world" }]);

    await getCustomSuggestions(ipcClient, { custom }, context);

    expect(runCachedGenerator).toHaveBeenCalled();
    expect(custom).toHaveBeenCalled();

    custom.mockClear();
  });
});
