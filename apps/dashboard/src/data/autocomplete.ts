const settings = [
  {
    title: 'General',
    properties: [
      {
        id: "autocomplete.insertSpaceAutomatically",
        title:
          "Automatically insert space for subcommands/options that take arguments",
        description:
          "Autocomplete will insert a space after you select a suggestion that contains a mandatory argument (e.g. selecting `git clone`)",
        type: "boolean",
        default: true,
        popular: true,
      },
      {
        id: "autocomplete.disable",
        title: "Enable Autocomplete",
        description:
          "CodeWhisperer will provide a list of subcommands and options for you to choose from.",
        type: "boolean",
        default: false,
        inverted: true,
      },
      {
        id: "autocomplete.immediatelyExecuteAfterSpace",
        title: "Allow Instant Execute After Space",
        description: "Immediately execute commands after pressing [space].",
        type: "boolean",
        default: false,
        popular: false,
      },
      {
        id: "autocomplete.scrollWrapAround",
        title: "Wrap Around",
        description:
          "If true, when the end of suggestions are reached by pressing the down arrow key, it will wrap back around to the top.",
        type: "boolean",
        default: false,
        popular: false,
      },
      {
        id: "autocomplete.alwaysSuggestCurrentToken",
        title: "Always suggest the current token",
        description:
          "Always add the current entered token as a suggestion at the top of the list",
        type: "boolean",
        default: false,
        popular: false,
      },
      {
        id: "autocomplete.fuzzySearch",
        title: "Fuzzy Search",
        description:
          "Search suggestions using substring matching rather than prefix search.",
        type: "boolean",
        default: false,
        popular: false,
      },
      {
        id: "autocomplete.preferVerboseSuggestions",
        title: "Prefer verbose suggestions",
        description: "Use the verbose version of the option/subcommand inserted.",
        example:
          "e.g. selecting `-m, --message` suggestion will insert `--message` rather than `-m`.",
        type: "boolean",
        default: false,
        popular: false,
      },
      {
        id: "autocomplete.onlyShowOnTab",
        title: "Suggest on [tab]",
        description:
          "Only show autocomplete when [tab] is pressed instead of showing it automatically.",
        type: "boolean",
        default: false,
        popular: false,
      },
      {
        id: "autocomplete.disableForCommands",
        title: "Disable CLIs",
        description:
          "A comma separated list of CLI tools that Fig should not autocomplete on ie Fig does not show the autocomplete popup for these CLI tools.",
        example: "e.g. `git`, `npm`, `cd`, `docker`...",
        type: "text",
        default: [],
        popular: false,
      },
      {
        id: "autocomplete.sortMethod",
        title: "Sort Suggestions",
        description: "Specifies how Fig should sort suggestions.",
        type: "select",
        default: "most recent",
        options: ["most recent", "alphabetical"],
        popular: false,
      },
      {
        id: "autocomplete.scriptTimeout",
        title: "Script Timeout",
        description:
          "Specify the timeout in ms for scripts executed in completion spec generators.",
        type: "number",
        default: 5000,
        popular: false,
      },
      {
        id: "autocomplete.immediatelyRunDangerousCommands",
        title: "Dangerous",
        description:
          'If true, users will be able to immediately run suggestions that completion specs have marked as "dangerous" rather than having to hit [enter] twice.',
        example: "(e.g. `rm -rf`)",
        type: "boolean",
        default: false,
        popular: false,
      },
      {
        id: "autocomplete.immediatelyRunGitAliases",
        title: "Safe git aliases",
        description:
          "When disabled, Autocomplete will treat git aliases as dangerous.",
        type: "boolean",
        default: true,
        popular: false,
      },
      {
        id: "autocomplete.firstTokenCompletion",
        title: "First token completion",
        description: "Offer completions for the first token of command.",
        example: "e.g. `cd`, `git`, etc.",
        type: "boolean",
        default: false,
        popular: false,
      },
    ],
  },
  {
    title: 'Appearance',
    withPreview: true,
    properties: [
      {
        id: "autocomplete.theme",
        title: "Theme",
        type: "select",
        options: [
          "system",
          "dark",
          "light",
          "cobalt",
          "cobalt2",
          "dracula",
          "github-dark",
          "gruvbox",
          "monokai-dark",
          "nightowl",
          "nord",
          "panda",
          "poimandres",
          "the-unnamed",
          "synthwave-84",
          "solarized-light",
          "solarized-dark",
        ],
        default: "system",
        popular: true,
      },
      {
        id: "autocomplete.height",
        title: "Window height",
        type: "number",
        default: 140,
        popular: false,
      },
      {
        id: "autocomplete.width",
        title: "Window width",
        type: "number",
        default: 320,
        popular: false,
      },
      {
        id: "autocomplete.fontFamily",
        title: "Font family",
        default: null,
        type: "text",
        popular: false,
      },
      {
        id: "autocomplete.fontSize",
        title: "Font size",
        default: null,
        type: "number",
        popular: false,
      },
      {
        id: "autocomplete.hidePreviewWindow",
        title: "Hide Preview window",
        description:
          "Hide the Preview window that appears on the side of the Autocomplete window.",
        type: "boolean",
        default: false,
        popular: false,
      },
      // {
      //   id: "autocomplete.iconTheme",
      //   title: "Icon theme",
      //   description: "Change the theme where icons are pulled from.",
      //   type: "text"
      //   default: null,
      //   popular: false
      // },
    ]
  },
  {
    title: 'Developer',
    properties: [
      {
        id: "autocomplete.developerMode",
        title: "Dev Mode",
        description: "Turns off completion-spec caching and loads completion specs from the Specs Folder specified below.",
        example: "Developer Mode changes the way specs are loaded.",
        type: "boolean",
        default: false,
        popular: false
      },
      {
        id: "autocomplete.devCompletionsFolder",
        title: "Specs Folder",
        description: "When Developer Mode is enabled, Fig loads completion specs from the specified directory.",
        type: "text",
        default: null,
        popular: false
      }
    ]
  }
];

export const keybindings = [
  {
    title: 'General',
    actions: [
      {
        id: "autocomplete.toggleHistoryMode",
        title: "Toggle history mode",
        description: "Toggle between history suggestions and Fig spec suggestions",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: ["control+r"]
      },
      {
        id: "autocomplete.toggleFuzzySearch",
        title: "Toggle fuzzy search",
        description: "Toggle between normal prefix search and fuzzy search",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: []
      }
    ],
  },
  {
    title: 'Appearance',
    actions: [
      {
        id: "autocomplete.inreaseSize",
        title: "Increase window size",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: [],
      },
      {
        id: "autocomplete.decreaseSize",
        title: "Decrease window size",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: [],
      },
      {
        id: "autocomplete.toggleAutocomplete",
        title: "Toggle autocomplete",
        description: "Toggle the visibility of the autocomplete window",
        availability: "ALWAYS",
        type: 'keystrokes',
        default: []
      },
      // {
      //   id: "autocomplete.hideAutocomplete",
      //   title: "Hide autocomplete",
      //   "category": "General",
      //   description: "Hide the autocomplete window",
      //   availability: "ALWAYS",
      // type: 'keystrokes',
      //   default: ["esc"]
      // },
      // {
      //   id: "autocomplete.showAutocomplete",
      //   title: "Show autocomplete",
      //   "category": "General",
      //   description: "Show the autocomplete window",
      //   availability: "ALWAYS",
      // type: 'keystrokes',
      //   default: []
      // },
      {
        id: "autocomplete.toggleDescription",
        title: "Toggle description popout",
        description: "Toggle visibility of autocomplete description popout",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: ["control+k"]
      },
      // {
      //   id: "autocomplete.hideDescription",
      //   title: "Hide description popout",
      //   category: "Appearance",
      //   description: "Hide autocomplete description popout",
      //   availability: "WHEN_FOCUSED",
      //   type: 'keystrokes',
      //   default: []
      // },
      // {
      //   id: "autocomplete.showDescription",
      //   title: "Show description popout",
      //   category: "Appearance",
      //   description: "Show autocomplete description popout",
      //   availability: "WHEN_FOCUSED",
      //   type: 'keystrokes',
      //   default: []
      // },
    ],
  },
  {
    title: 'Insertion',
    actions: [
      {
        id: "autocomplete.insertSelected",
        title: "Insert selected",
        description: "Insert selected suggestion",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: ["enter"]
      },
      {
        id: "autocomplete.insertCommonPrefix",
        title: "Insert common prefix or shake",
        description: "Insert shared prefix of available suggestions. Shake if there's no common prefix.",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: ["tab"]
      },
      {
        id: "autocomplete.insertCommonPrefixOrNavigateDown",
        title: "Insert common prefix or navigate",
        description: "Insert shared prefix of available suggestions. Navigate if there's no common prefix.",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: []
      },
      {
        id: "autocomplete.insertCommonPrefixOrInsertSelected",
        title: "Insert common prefix or insert selected",
        description: "Insert shared prefix of available suggestions. Insert currently selected suggestion if there's not common prefix.",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: []
      },
      {
        id: "autocomplete.insertSelectedAndExecute",
        title: "Insert selected and execute",
        description: "Insert selected suggestion and then execute the current command.",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: []
      },
      {
        id: "autocomplete.execute",
        title: "Execute",
        description: "Execute the current command.",
        availability: "WHEN_FOCUSED",
        type: 'keystrokes',
        default: []
      },
    ]
  },
  {
    title: 'Navigation',
    actions: [
      {
        id: "autocomplete.navigateUp",
        title: "Navigate up",
        type: 'keystrokes',
        description: "Scroll up one entry in the list of suggestions",
        availability: "WHEN_FOCUSED",
        default: ["shift+tab", "up", "control+p"]
      },
      {
        id: "autocomplete.navigateDown",
        title: "Navigate down",
        type: 'keystrokes',
        description: "Scroll down one entry in the list of suggestions",
        availability: "WHEN_FOCUSED",
        default: ["down", "control+n"]
      },
    ]
  }
]

export default settings;
