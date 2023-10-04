import { AuthClient } from "../../auth.js";
import { makeRequest } from "../../request.js";
import type { MixinMeta } from "./types.js";

export const getAll = (authClient: AuthClient) =>
  makeRequest("mixins", authClient).then((res) => res.json()) as Promise<
    MixinMeta[]
  >;
