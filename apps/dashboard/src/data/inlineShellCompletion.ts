const inlineShellCompletionSettings = [
  {
    title: "Settings",
    properties: [],
  },
];

export default inlineShellCompletionSettings;

export const intro = {
  title: "Inline shell completions",
  description: "AI-generated command suggestions.",
  link: "https://aws.amazon.com/codewhisperer/",
  enable: {
    flag: "inline-shell-completion.enabled",
    inverted: false,
    default: false,
  },
};
