import {
  AuthBuilderIdStartDeviceAuthorizationResponse,
  // eslint-disable-next-line camelcase
  AuthStatusResponse_AuthKind,
  AuthBuilderIdPollCreateTokenResponse_PollStatus as PollStatus,
} from "@fig/fig-api-proto/fig";
import {
  sendAuthBuilderIdStartDeviceAuthorizationRequest,
  sendAuthBuilderIdPollCreateTokenRequest,
  sendAuthStatusRequest,
} from "./requests.js";

export function status() {
  return sendAuthStatusRequest({}).then((res) => {
    let authKind: "BuilderId" | "IamIdentityCenter" | undefined;
    switch (res.authKind) {
      // eslint-disable-next-line camelcase
      case AuthStatusResponse_AuthKind.BUILDER_ID:
        authKind = "BuilderId";
        break;
      // eslint-disable-next-line camelcase
      case AuthStatusResponse_AuthKind.IAM_IDENTITY_CENTER:
        authKind = "IamIdentityCenter";
        break;
      default:
        break;
    }

    return {
      authed: res.authed,
      authKind,
      startUrl: res.startUrl,
      region: res.region,
    };
  });
}

export function builderIdStartDeviceAuthorization({
  region,
  startUrl,
}: {
  region?: string;
  startUrl?: string;
} = {}): Promise<AuthBuilderIdStartDeviceAuthorizationResponse> {
  return sendAuthBuilderIdStartDeviceAuthorizationRequest({
    region,
    startUrl,
  });
}

export async function builderIdPollCreateToken({
  authRequestId,
  expiresIn,
  interval,
}: AuthBuilderIdStartDeviceAuthorizationResponse) {
  for (let i = 0; i < Math.ceil(expiresIn / interval); i += 1) {
    // eslint-disable-next-line no-await-in-loop
    await new Promise((resolve) => {
      setTimeout(resolve, interval * 1000);
    });

    // eslint-disable-next-line no-await-in-loop
    const pollStatus = await sendAuthBuilderIdPollCreateTokenRequest({
      authRequestId,
    });

    switch (pollStatus.status) {
      case PollStatus.COMPLETE:
        return;
      case PollStatus.PENDING:
        break;
      case PollStatus.ERROR:
        console.error("Failed to poll builder id token", pollStatus);
        throw new Error(pollStatus.error);
      default:
        throw new Error(`Unknown poll status: ${pollStatus.status}`);
    }
  }
}
