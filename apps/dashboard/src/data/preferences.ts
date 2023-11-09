const generalPreferences = [
  {
    title: "Startup",
    properties: [
      {
        id: "app.launchOnStartup",
        title: "Launch on Start",
        description:
          "Start CodeWhisperer automatically whenever you restart your computer.",
        type: "boolean",
        default: true,
        popular: false,
      },
      // {
      //   id: "app.preferredTerminal",
      //   title: "Preferred Terminal",
      //   description:
      //     "Choose your preferred terminal for CodeWhisperer to launch commands in.",
      //   type: "select",
      //   options: [
      //     "VS Code",
      //     "iTerm2",
      //     "Hyper",
      //     "Alacritty",
      //     "Kitty",
      //     "Terminal",
      //   ],
      //   default: "Terminal",
      //   popular: false,
      // },
      // {
      //   id: "app.disableAutolaunch",
      //   title: "Open in new shells",
      //   description: "Automatically launch when opening a new shell session",
      //   type: "boolean",
      //   default: true,
      //   inverted: true,
      //   popular: false,
      // },
      {
        id: "app.disableAutoupdates",
        title: "Automatic updates",
        description:
          "Asynchronously check for updates when launching a new shell session.",
        type: "boolean",
        default: false,
        inverted: true,
        popular: false,
      },
      {
        id: "app.hideMenubarIcon",
        title: "Display Menu Bar icon",
        description:
          "CodeWhisperer icon will appear in the Menu Bar while CodeWhisperer is running.",
        type: "boolean",
        default: false,
        inverted: true,
        popular: false,
      },
    ],
  },
  {
    title: "Advanced",
    properties: [
      {
        id: "app.beta",
        title: "Beta",
        description:
          "Opt into more frequent updates with all the newest features (and bugs).",
        type: "boolean",
        default: false,
        popular: false,
      },
      // {
      //   id: "cli.tips.disabled",
      //   title: "Terminal Tips",
      //   description: "Offers tips at the top of your terminal on start",
      //   type: "boolean",
      //   default: false,
      //   popular: false,
      // },
      {
        id: "telemetry.enabled",
        title: "Telemetry",
        description:
          "Enable CodeWhisperer for command line to send usage data to AWS",
        example:
          "Learn more at https://docs.aws.amazon.com/codewhisperer/latest/userguide/sharing-data.html",
        type: "boolean",
        default: true,
        popular: false,
      },
      {
        id: "codeWhisperer.shareCodeWhispererContentWithAWS",
        title: "Share CodeWhisperer content with AWS",
        description:
          "When checked, your content processed by CodeWhisperer may be used for service improvement (except for content processed by the Professional CodeWhisperer service tier). Unchecking this box will cause AWS to delete any of your content used for that purpose. The information used to provide the CodeWhisperer service to you will not be affected.",
        example:
          "Learn more at https://docs.aws.amazon.com/codewhisperer/latest/userguide/sharing-data.html",
        type: "boolean",
        default: true,
        popular: false,
      },
    ],
  },
];

export default generalPreferences;
