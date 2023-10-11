import { InstallCheck } from "@/types/preferences";
import installChecks from "./install";

const onboarding: InstallCheck[] = [
  {
    id: 'welcome',
    title: 'Welcome to CodeWhisperer',
    description: [''],
    action: 'Continue'
  },
  ...installChecks,
  {
    id: "login",
    title: "Signed in with Builder ID",
    description: [
      "AI features won't work if you're no longer signed into Builder ID.",
    ],
    action: "Sign in",
  }
]

export default onboarding