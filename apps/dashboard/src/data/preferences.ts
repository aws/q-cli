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
          "By default, AWS collects limited usage information to provide support and improve the product.",
        example:
          "Read our statement on privacy at aws.amazon.com/privacy for more details.",
        type: "boolean",
        default: true,
        popular: false,
      },
      {
        id: "codeWhisperer.shareCodeWhispererContentWithAWS",
        title: "Share CodeWhisperer content with AWS",
        description:
          "Your content processed by CodeWhisperer may be used for service improvement (except for content processed by CodeWhisperer’s Enterprise tier). Disabling this setting will cause AWS to delete all of your content used for that purpose.",
        example:
          "The information used to provide the CodeWhisperer service to you will not be affected. See the Service Terms for more detail.",
        type: "boolean",
        default: true,
        popular: false,
      },
    ],
  },
];

export default generalPreferences;
