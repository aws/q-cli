import * as wrappers from "@amzn/fig-io-api-bindings-wrappers";
import * as loadHelpers from "../src/loadHelpers";

const { getVersionFromFullFile } = loadHelpers;

const specData = {
  getVersionCommand: jest.fn().mockReturnValue(Promise.resolve("1.0.0")),
  default: () => {},
};

describe("test `getVersionFromFullFile`", () => {
  beforeEach(() => {
    jest.spyOn(loadHelpers, "getCachedCLIVersion").mockReturnValue(null);
  });
  afterEach(() => {
    jest.clearAllMocks();
  });
  it("missing `getVersionCommand` and working `command --version`", async () => {
    jest
      .spyOn(wrappers, "executeCommand")
      .mockReturnValue(Promise.resolve("v2.0.0"));
    const newSpecData = { ...specData, getVersionCommand: undefined };
    const version = await getVersionFromFullFile(newSpecData, "fig");
    expect(version).toEqual("2.0.0");
  });

  it("missing `getVersionCommand` and not working `command --version`", async () => {
    jest
      .spyOn(wrappers, "executeCommand")
      .mockReturnValue(Promise.resolve("No command available."));
    const newSpecData = { ...specData, getVersionCommand: undefined };
    const version = await getVersionFromFullFile(newSpecData, "npm");
    expect(version).toBeUndefined();
  });

  it("missing `getVersionCommand` and throwing `command --version`", async () => {
    jest.spyOn(wrappers, "executeCommand").mockReturnValue(Promise.reject());
    const newSpecData = { ...specData, getVersionCommand: undefined };
    const version = await getVersionFromFullFile(newSpecData, "npm");
    expect(version).toBeUndefined();
  });

  it("working `getVersionCommand`", async () => {
    jest
      .spyOn(wrappers, "executeCommand")
      .mockReturnValue(Promise.resolve("No command available."));
    const version = await getVersionFromFullFile(specData, "node");
    expect(version).toEqual("1.0.0");
  });
});
