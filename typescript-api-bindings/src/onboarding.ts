import { OnboardingAction } from "./fig.pb";
import { sendOnboardingRequest } from "./requests";

export async function onboard(onboardingAction: OnboardingAction) {
  await sendOnboardingRequest({
    action: onboardingAction
  });
}
