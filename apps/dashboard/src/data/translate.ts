const translateSettings = [{
  title: 'Translate',
  properties: [
    {
      id: "ai.terminal-hash-sub",
      title: "Hashtag Substitution",
      description: "Comments in the shell will be substituted with the `cw ai` command.",
      type: "boolean",
      default: true,
      inverted: true,
    },
    // {
    //   id: "ai.menu-actions",
    //   title: "Menu Actions",
    //   description: "The actions that will be available in the AI menu.",
    //   type: "multiselect",
    //   options: ["execute", "edit", "copy", "regenerate", "ask", "cancel"],
    //   default: ["execute", "edit", "regenerate", "ask", "cancel"],
    //   inverted: true,
    // },
  ]
}]

export default translateSettings

export const intro = {
  title: 'Translation',
  description: 'Translate natural language-to-bash. Just run `cw ai`.',
  link: 'https://fig.io/user-manual/ai',
  enable: {
    flag: 'ai.disable',
    inverted: true,
    default: false
  }
}