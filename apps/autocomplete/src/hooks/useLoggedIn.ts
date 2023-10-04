import { useRefreshTokenExpirationStatus } from "@amzn/fig-io-api-client";
import { authClient } from "../auth.js";
import { useAutocompleteStore } from "../state/index.js";

export const useLoggedIn = () => {
  const {
    figState: { buffer },
  } = useAutocompleteStore();
  const expirationStatus = useRefreshTokenExpirationStatus(buffer, authClient);
  const isLoggedIn =
    !expirationStatus.loading && expirationStatus.expired === false;
  return isLoggedIn;
};
