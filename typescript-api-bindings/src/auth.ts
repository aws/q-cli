import {
  AuthBuilderIdInitResponse,
  AuthBuilderIdPollResponse_PollStatus as PollStatus,
} from "@fig/fig-api-proto/dist/fig.pb";
import {
  sendAuthBuilderIdInitRequest,
  sendAuthBuilderIdPollRequest,
  sendAuthStatusRequest,
} from "./requests";

export function authStatus() {
  return sendAuthStatusRequest({});
}

export function authBuilderIdInit() {
  return sendAuthBuilderIdInitRequest({});
}

export async function authBuilderIdPoll({
  authRequestId,
  expiresIn,
  interval,
}: AuthBuilderIdInitResponse) {
  for (let i = 0; i < Math.ceil(expiresIn / interval); i++) {
    await new Promise((resolve) => setTimeout(resolve, interval * 1000));

    let status = await sendAuthBuilderIdPollRequest({
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
