const settings = {
  integrations: [
    {
      id: "integrations.terminal.disabled",
      title: "Terminal",
      type: "boolean",
      default: false,
      inverted: true,
    },
    {
      id: "integrations.hyper.disabled",
      title: "Hyper",
      type: "boolean",
      default: false,
      inverted: true,
    },
    {
      id: "integrations.vscode.disabled",
      title: "VSCode",
      type: "boolean",
      default: false,
      inverted: true,
    },
    {
      id: "integrations.iterm.disabled",
      title: "iTerm",
      type: "boolean",
      default: false,
      inverted: true,
    }
  ],
  experimental: [
    {
      id: "figterm.csi-u.enabled",
      title: "CSI u Intercept Mode (experimental)",
      description: "Enable the experimental CSI u integration. This mode allows you to use more modifier keys for keybindings. It however may modify the behavior of keys you press, if you experience issues, please report them to us.",
      type: "boolean",
      default: false,
    }
  ]
}

export default settings