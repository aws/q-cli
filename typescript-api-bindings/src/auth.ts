import {
  AuthBuilderIdStartDeviceAuthorizationResponse,
  AuthBuilderIdPollCreateTokenResponse_PollStatus as PollStatus,
} from "@fig/fig-api-proto/dist/fig.pb";
import {
  sendAuthBuilderIdStartDeviceAuthorizationRequest,
  sendAuthBuilderIdPollCreateTokenRequest,
  sendAuthStatusRequest,
} from "./requests";

export function status() {
  return sendAuthStatusRequest({});
}

export function builderIdStartDeviceAuthorization() {
  return sendAuthBuilderIdStartDeviceAuthorizationRequest({});
}

export async function builderIdPollCreateToken({
  authRequestId,
  expiresIn,
  interval,
}: AuthBuilderIdStartDeviceAuthorizationResponse) {
  for (let i = 0; i < Math.ceil(expiresIn / interval); i++) {
    await new Promise((resolve) => setTimeout(resolve, interval * 1000));

    let status = await sendAuthBuilderIdPollCreateTokenRequest({
      authRequestId,
    });

    switch (status.status) {
      case PollStatus.COMPLETE:
        return;
      case PollStatus.PENDING:
        continue;
      case PollStatus.ERROR:
        throw new Error(status.error);
    }
  }
}
