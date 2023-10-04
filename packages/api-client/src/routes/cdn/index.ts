import { AuthClient } from "../../auth.js";
import { makeRequest } from "../../request.js";
import type { PrivateSpecInfo } from "./types.js";

export const getPrivateSpecList = (authClient: AuthClient) =>
  makeRequest("cdn", authClient).then((res) => res.json()) as Promise<
    PrivateSpecInfo[]
  >;

export const getPrivateSpec = (
  ns: string,
  name: string,
  authClient: AuthClient,
) => makeRequest(`cdn/${ns}/${name}`, authClient).then((res) => res.text());
