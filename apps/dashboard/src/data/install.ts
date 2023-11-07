import { InstallCheckWithInstallKey } from "@/types/preferences";
import accessibility_fig from "@assets/images/accessibility_fig.png?url"

const installChecks: InstallCheckWithInstallKey[] = [
  {
    id: "dotfiles",
    installKey: "dotfiles",
    title: "Shell integrations",
    description: [
      "Integrate CodeWhisperer with your local shell so we can run the necessary hooks.",
    ],
    action: "Install",
    explainer: {
      title: "What's happening under the hood?",
      steps: [
        [
          {
            content:
              "CodeWhisperer will add one line to the top and bottom of the following files (if they exist):",
            tag: "span",
          },
          {
            content: ".zshrc",
            tag: "code",
          },
          {
            content: ".zprofile",
            tag: "code",
          },
          {
            content: ".bashrc",
            tag: "code",
          },
          {
            content: ".bash_profile",
            tag: "code",
          },
          {
            content: ".profile",
            tag: "code",
          },
        ],
        [
          {
            content: "Your dotfiles will be backed up to",
            tag: "span",
          },
          {
            content: "~/.codewhisperer.dotfiles.bak/",
            tag: "code",
          },
        ],
        [
          {
            content: "Uninstalling CodeWhisperer will remove these lines.",
            tag: "span",
          },
        ],
      ],
    },
  },
  {
    id: "accessibility",
    installKey: "accessibility",
    title: "Enable accessibility",
    description: [
      "Grant accessibility permissions so CodeWhisperer can position the completion window and insert text on your behalf.",
      "If enabling it isn't working, try toggling it off and on again or restarting CodeWhisperer.",
    ],
    image: accessibility_fig,
    action: "Enable",
  },
  // {
  //   id: "inputMethod",
  //   installKey: "inputMethod",
  //   title: "Input methods",
  //   description:
  //     ["Integrate CodeWhisperer with your local shell so we can run the necessary hooks."],
  //   image: '/asdf',
  //     action: "Enable",
  // },
];

export default installChecks;
