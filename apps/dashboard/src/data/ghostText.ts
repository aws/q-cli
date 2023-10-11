const ghostTextSettings = [
  {
    title: "Ghost Text",
    properties: [
      {
        id: "predict.disable",
        title: "Enable CodeWhisperer predictions",
        description:
          "CodeWhisperer will try to complete your current input for you.",
        type: "boolean",
        default: false,
        inverted: true,
        popular: false,
      },
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
