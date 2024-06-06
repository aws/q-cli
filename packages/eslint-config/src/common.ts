import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
// @ts-ignore
import unicorn from "eslint-plugin-unicorn";
// @ts-ignore
import eslintConfigPrettier from "eslint-config-prettier";
import type { TSESLint } from "@typescript-eslint/utils";

export const CONFIG: TSESLint.FlatConfig.ConfigArray = [
  eslint.configs.recommended,
  ...tseslint.configs.recommended,

  // typescript-eslint rules
  {
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
    },
  },

  // unicorn rules
  {
    plugins: {
      unicorn,
    },
    rules: {
      // 'unicorn/no-useless-promise-resolve-reject': 'error',
      // 'unicorn/prefer-at': 'error',
      // 'unicorn/prefer-event-target': 'error',
      // 'unicorn/prefer-negative-index': 'error',
      // 'unicorn/prefer-string-slice': 'error',
      // 'unicorn/prefer-regexp-test': 'error',
      // 'unicorn/prefer-ternary': 'error',
      // 'unicorn/custom-error-definition': 'error',
      // 'unicorn/prefer-json-parse-buffer': 'error',
      "unicorn/prefer-module": "error",
      "unicorn/no-abusive-eslint-disable": "error",
      // "unicorn/no-null": "error",
      "unicorn/no-unnecessary-polyfills": "error",
      "unicorn/no-useless-spread": "error",
      // "unicorn/prefer-array-some": "error",
      "unicorn/prefer-blob-reading-methods": "error",
      // "unicorn/prefer-code-point": "error",
      "unicorn/prefer-date-now": "error",
      // "unicorn/prefer-dom-node-text-content": "error",
      // "unicorn/prefer-includes": "error",
      "unicorn/prefer-keyboard-event-key": "error",
      "unicorn/prefer-modern-dom-apis": "error",
      "unicorn/prefer-modern-math-apis": "error",
      "unicorn/prefer-native-coercion-functions": "error",
      "unicorn/prefer-node-protocol": "error",
      "unicorn/prefer-object-from-entries": "error",
      "unicorn/prefer-reflect-apply": "error",
      "unicorn/prefer-string-trim-start-end": "error",
      "unicorn/prefer-type-error": "error",
    },
  },

  eslintConfigPrettier,
];
