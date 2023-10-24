const ghostTextSettings = [
  {
    title: "Settings",
    properties: [
      {
        id: "predict.telemetry",
        title: "Share CodeWhisperer content with AWS",
        description:
          "Your content processed by CodeWhisperer may be used for service improvement (except for content processed by CodeWhisperer’s Enterprise tier). Disabling this setting will cause AWS to delete all of your content used for that purpose.",
        example:
          "The information used to provide the CodeWhisperer service to you will not be affected. See the Service Terms for more detail.",
        type: "boolean",
        default: false,
        inverted: true,
        popular: false,
      },
    ],
  },
];

export default ghostTextSettings;

export const intro = {
  title: 'GhostText',
  description: 'AI-generated command suggestions.',
  link: 'https://aws.amazon.com/codewhisperer/',
  enable: {
    flag: 'ghost-text.enabled',
    inverted: false,
    default: false,
  }
}