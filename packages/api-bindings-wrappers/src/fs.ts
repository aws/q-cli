import { fs as FileSystem } from "@amzn/fig-io-api-bindings";

export const fread = (path: string): Promise<string> =>
  FileSystem.read(path).then((out) => out ?? "");
