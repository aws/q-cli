import { Intro, PrefSection } from "@/components/preference/list";

const inlineShellCompletionSettings: PrefSection[] = [
  {
    title: "Settings",
    properties: [
      {
        id: "inline-shell-completion.enabled",
        title: "Enable inline shell completions",
        description: "This setting only applies in new shell sessions.",
        type: "boolean",
        default: true,
      },
    ],
  },
];

export default inlineShellCompletionSettings;

export const intro: Intro = {
  title: "Inline shell completions",
  description: "AI-generated command suggestions.",
  disabled: true,
  enable: {
    flag: "inline-shell-completion.enabled",
    inverted: false,
    default: true,
  },
};
