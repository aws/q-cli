/** @type {import('eslint').Linter.Config} */
module.exports = {
  env: {
    browser: true,
  },
  extends: [
    "plugin:react/recommended",
    "plugin:react/jsx-runtime",
    "plugin:react-hooks/recommended",
    "./base.js",
  ],
  plugins: ["react", "react-refresh"],
  globals: {
    fig: true,
    Sentry: true,
  },
  rules: {
    "react/jsx-filename-extension": ["warn", { extensions: [".tsx"] }],
    "react/jsx-key": "warn",
    "react-refresh/only-export-components": [
      "warn",
      { allowConstantExport: true },
    ],
  },
  settings: {
    react: {
      version: "detect",
    },
  },
};
