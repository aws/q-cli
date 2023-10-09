import { InstallCheck } from "@/types/preferences";

const installChecks: InstallCheck[] = [
  {
    id: "shellIntegrations",
    installKey: "dotfiles",
    title: "Shell integrations",
    description:
      ["Pick your favorite tools to configure them for use with CodeWhisperer"],
    example: '/asdf',
    action: "Install",
  },
  {
    id: "accessibility",
    installKey: "accessibility",
    title: "Accessibility settings",
    description:
      ["Fig uses this permission to position the Autocomplete window and insert text on your behalf.", "If enabling it isn't working, try toggling it off and on again or restarting Fig."],
      example: '/asdf',
    action: "Enable",
  },
  {
    id: "inputMethod",
    installKey: "inputMethod",
    title: "Input methods",
    description:
      ["Integrate CodeWhisperer with your local shell so we can run the necessary hooks."],
      example: '/asdf',
      action: "Enable",
  },
  {
    id: "login",
    title: "Log in with Builder ID",
    description: [],
    example: '',
    action: "Log in"
  }
];

export default installChecks