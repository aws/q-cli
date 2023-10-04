import { useEffect, useState } from "react";
import { AuthClient, Credentials } from "./auth.js";

type RefreshTokenStatus = {
  loading: boolean;
  expired?: boolean;
};

// TODO(refactoring): why this does not work when I move it inside the useRefresh... hook?
let credentialsRequest: Promise<Credentials>;
let getCredentialsFN: (() => Promise<Credentials>) | undefined;
setTimeout(() => {
  if (getCredentialsFN) {
    credentialsRequest = getCredentialsFN();
  }
}, 2000);

export const useRefreshTokenExpirationStatus = (
  buffer: string,
  client: AuthClient,
) => {
  const [status, setStatus] = useState<RefreshTokenStatus>({ loading: false });

  if (!getCredentialsFN) getCredentialsFN = client.getCredentialsFromFile;

  useEffect(() => {
    let stale = false;
    if (status.expired !== false) {
      if (buffer.trim() === "") {
        // Always hard refresh local credentials representation on a new line.
        credentialsRequest = client.getCredentialsFromFile();
      }
      setStatus((state) => ({ ...state, loading: true }));
      credentialsRequest.then((credentials) => {
        if (!stale) {
          setStatus({
            loading: false,
            expired:
              !credentials.refresh_token ||
              (credentials.refresh_token_expired ?? false),
          });
        }
      });
    }
    return () => {
      stale = true;
    };
  }, [buffer, status.expired]);

  return status;
};
