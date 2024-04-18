import { CLI_BINARY_NAME } from "@/lib/constants";

const supportSteps = {
  steps: [
    `Run \`${CLI_BINARY_NAME} doctor\` to automatically debug`,
    `Run \`${CLI_BINARY_NAME} issue\` to create an auto-populated issue`,
  ],
  links: [
    // {
    //   text: 'Troubleshooting guide',
    //   url: ''
    // },
    {
      text: "User manual",
      url: "https://docs.aws.amazon.com/codewhisperer/latest/userguide/command-line.html",
    },
  ],
};

export default supportSteps;
