import { onboardingActionFromJSON } from "./fig.pb";
import { sendOnboardingRequest } from "./requests";

export type OnboardingAction =
  | "INSTALLATION_SCRIPT"
  | "PROMPT_FOR_ACCESSIBILITY_PERMISSION"
  | "LAUNCH_SHELL_ONBOARDING"
  | "UNINSTALL"
  | "CLOSE_ACCESSIBILITY_PROMPT_WINDOW"
  | "REQUEST_RESTART"
  | "CLOSE_INPUT_METHOD_PROMPT_WINDOW";

export async function onboard(onboardingAction: OnboardingAction) {
  await sendOnboardingRequest({
    action: onboardingActionFromJSON(onboardingAction)
  });
}
