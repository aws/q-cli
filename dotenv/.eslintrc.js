module.exports = {
  env: {
    browser: true,
    es2021: true,
    node: true,
    "jest/globals": true,
  },
  extends: [
    "airbnb/base",
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:import/typescript",
    "prettier",
  ],

  parserOptions: {
    ecmaVersion: 12,
    parser: "@typescript-eslint/parser",
    sourceType: "module",
  },
  plugins: ["@typescript-eslint", "jest", "prettier"],
  rules: {
    "prettier/prettier": ["error"],
    "import/prefer-default-export": 0,
    "@typescript-eslint/explicit-module-boundary-types": 0,
    "@typescript-eslint/no-empty-function": 0,
    "max-len": ["error", {code: 100, tabWidth: 2}],
    "semi": ["error", "always"],
    "no-console": 0,
    "no-shadow": "off",
    "@typescript-eslint/no-shadow": ["error"],
    "no-use-before-define": "off",
    "@typescript-eslint/no-use-before-define": ["error", "nofunc"],
    "no-bitwise": "off",
    "import/extensions": 0,
    "import/no-unresolved": 0,
    "no-restricted-syntax": 0,
    "keyword-spacing": "error"
  },
};
