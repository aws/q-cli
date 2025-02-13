import * as helpers from "../helpers";
import { Annotation } from "@aws/amazon-q-developer-cli-autocomplete-parser";
import { describe, expect, it, vi } from "vitest";
import { getScriptSuggestions } from "../scriptSuggestionsGenerator";
import { IpcClient } from "@aws/amazon-q-developer-cli-ipc-client-core";
import { create } from "@bufbuild/protobuf";
import { afterEach } from "node:test";
import { RunProcessResponseSchema } from "@aws/amazon-q-developer-cli-proto/fig";
import { RunProcessRequestSchema } from "@aws/amazon-q-developer-cli-proto/remote";

const context: helpers.GeneratorContext = {
  annotations: [] as Annotation[],
  tokenArray: [] as string[],
  currentWorkingDirectory: "/",
  sshPrefix: "",
  currentProcess: "zsh",
  searchTerm: "",
  environmentVariables: {},
};

describe("getScriptSuggestions", () => {
  const ipcClient = {
    runProcess: vi.fn(async (_sessionId, _request) => {
      return create(RunProcessResponseSchema, {
        exitCode: 0,
        stdout: "a/\nx\nc/\nl",
        stderr: "",
      });
    }),
  } as Partial<IpcClient> as IpcClient;

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("should return empty suggestions if no script in generator", async () => {
    expect(
      await getScriptSuggestions(ipcClient, { script: [] }, context, 5000),
    ).toEqual([]);
  });

  it("should return empty suggestions if no splitOn or postProcess", async () => {
    expect(
      await getScriptSuggestions(
        ipcClient,
        { script: ["ascript"] },
        context,
        5000,
      ),
    ).toEqual([]);
  });

  it("should return the result with splitOn", async () => {
    expect(
      await getScriptSuggestions(
        ipcClient,
        { script: ["ascript"], splitOn: "\n" },
        context,
        5000,
      ),
    ).toEqual([
      { insertValue: "a/", isDangerous: undefined, name: "a/", type: "arg" },
      { insertValue: "x", isDangerous: undefined, name: "x", type: "arg" },
      { insertValue: "c/", isDangerous: undefined, name: "c/", type: "arg" },
      { insertValue: "l", isDangerous: undefined, name: "l", type: "arg" },
    ]);
  });

  it("should return the result with postProcess", async () => {
    const postProcess = vi
      .fn()
      .mockReturnValue([{ name: "hello" }, { name: "world" }]);

    expect(
      await getScriptSuggestions(
        ipcClient,
        { script: ["ascript"], postProcess },
        context,
        5000,
      ),
    ).toEqual([
      { name: "hello", type: "arg" },
      { name: "world", type: "arg" },
    ]);
    expect(postProcess).toHaveBeenCalledWith("a/\nx\nc/\nl", []);
  });

  it("should return the result with postProcess and infer type", async () => {
    const postProcess = vi.fn().mockReturnValue([
      { name: "hello", type: "auto-execute" },
      { name: "world", type: "folder" },
    ]);

    expect(
      await getScriptSuggestions(
        ipcClient,
        { script: ["ascript"], postProcess },
        context,
        5000,
      ),
    ).toEqual([
      { name: "hello", type: "auto-execute" },
      { name: "world", type: "folder" },
    ]);
    expect(postProcess).toHaveBeenCalledWith("a/\nx\nc/\nl", []);
  });

  it("should call script if provided", async () => {
    const script = vi.fn().mockReturnValue("myscript");
    await getScriptSuggestions(ipcClient, { script }, context, 5000);
    expect(script).toHaveBeenCalledWith([]);
  });

  it("should call runCachedGenerator", async () => {
    const runCachedGenerator = vi.spyOn(helpers, "runCachedGenerator");
    await getScriptSuggestions(
      ipcClient,
      { script: ["ascript"] },
      context,
      5000,
    );
    expect(runCachedGenerator).toHaveBeenCalled();
  });

  it("should call executeCommand", async () => {
    await getScriptSuggestions(
      ipcClient,
      { script: ["ascript"] },
      context,
      5000,
    );
    expect(ipcClient.runProcess).toHaveBeenCalledWith(
      "",
      create(RunProcessRequestSchema, {
        executable: "ascript",
        arguments: [],
        workingDirectory: "/",
      }),
    );
  });

  it("should call executeCommand with 'spec-specified' timeout", async () => {
    await getScriptSuggestions(
      ipcClient,
      { script: ["ascript"], scriptTimeout: 6000 },
      context,
      5000,
    );
    expect(ipcClient.runProcess).toHaveBeenCalledWith(
      "",
      create(RunProcessRequestSchema, {
        executable: "ascript",
        arguments: [],
        workingDirectory: "/",
      }),
    );
  });

  it("should use the greatest between the settings timeout and the spec defined one", async () => {
    await getScriptSuggestions(
      ipcClient,
      { script: ["ascript"], scriptTimeout: 3500 },
      context,
      7000,
    );
    expect(ipcClient.runProcess).toHaveBeenCalledWith(
      "",
      create(RunProcessRequestSchema, {
        executable: "ascript",
        arguments: [],
        workingDirectory: "/",
      }),
    );
  });

  it("should call executeCommand without timeout when the user defined ones are negative", async () => {
    await getScriptSuggestions(
      ipcClient,
      { script: ["ascript"], scriptTimeout: -100 },
      context,
      -1000,
    );
    expect(ipcClient.runProcess).toHaveBeenCalledWith(
      "",
      create(RunProcessRequestSchema, {
        executable: "ascript",
        arguments: [],
        workingDirectory: "/",
      }),
    );
  });

  it("should call executeCommand with settings timeout when no 'spec-specified' one is defined", async () => {
    await getScriptSuggestions(
      ipcClient,
      { script: ["ascript"] },
      context,
      6000,
    );
    expect(ipcClient.runProcess).toHaveBeenCalledWith(
      "",
      create(RunProcessRequestSchema, {
        executable: "ascript",
        arguments: [],
        workingDirectory: "/",
      }),
    );
  });

  describe("deprecated sshPrefix", () => {
    it("should call executeCommand ignoring ssh", async () => {
      await getScriptSuggestions(
        ipcClient,
        { script: ["ascript"] },
        {
          ...context,
          sshPrefix: "ssh -i blabla",
        },
        5000,
      );

      expect(ipcClient.runProcess).toHaveBeenCalledWith(
        "",
        create(RunProcessRequestSchema, {
          executable: "ascript",
          arguments: [],
          workingDirectory: "/",
        }),
      );
    });
  });
});
