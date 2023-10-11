const generalPreferences = [{
  title: 'Startup',
  properties: [
    {
      id: "app.launchOnStartup",
      title: "Launch on Start",
      description: "Start CodeWhisperer automatically whenever you restart your computer.",
      type: "boolean",
      default: true,
      popular: false
    },
    // {
    //   id: "app.showAllProducts",
    //   title: "Show All Products",
    //   description: "Show all products in the Fig app sidebar.",
    //   type: "boolean",
    //   default: true,
    //   popular: false
    // },
    {
      id: "app.preferredTerminal",
      title: "Preferred Terminal",
      description: "Choose your preferred terminal for Fig to launch commands in.",
      type: "select",
      "options": [
        "VS Code",
        "iTerm2",
        "Hyper",
        "Alacritty",
        "Kitty",
        "Terminal"
      ],
      default: "Terminal",
      popular: false
    },
    {
      id: "app.disableAutolaunch",
      title: "Open in new shells",
      description: "Automatically launch when opening a new shell session",
      type: "boolean",
      default: false,
      inverted: true,
      popular: false
    },
    {
      id: "app.disableAutoupdates",
      title: "Automatic updates",
      description: "Asynchronously check for updates when launching a new shell session.",
      type: "boolean",
      default: false,
      inverted: true,
      popular: false
    },
    {
      id: "app.hideMenubarIcon",
      title: "Display Menu Bar icon",
      description: "CodeWhisperer icon will appear in the Menu Bar while CodeWhisperer is running.",
      type: "boolean",
      default: true,
      inverted: true,
      popular: false
    },
  ]},
  {
  title: 'Advanced',
  properties: [
    {
      id: "app.beta",
      title: "Beta",
      description: "Opt into more frequent updates with all the newest features (and bugs).",
      type: "boolean",
      default: false,
      popular: false
    },
    {
      id: "cli.tips.disabled",
      title: "Terminal Tips",
      description: "Offers tips at the top of your terminal on start",
      type: "boolean",
      default: false,
      popular: false
    },
    {
      id: "telemetry.disabled",
      title: "Disable Telemetry",
      description: "By default, AWS collects limited usage information to provide support and improve the product.",
      example: "Read our statement on privacy at aws.amazon.com/privacy for more details.",
      type: "boolean",
      default: false,
      popular: false
    },
  ]
}]

export default generalPreferences