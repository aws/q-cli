// @ts-check

import tseslint from "typescript-eslint";
import { CONFIG } from "./common.js";
import react from "eslint-plugin-react/configs/recommended.js";
import jsxRuntime from "eslint-plugin-react/configs/jsx-runtime.js";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tseslint.config(
  ...CONFIG,
  react,
  jsxRuntime,
  {
    plugins: {
      "react-hooks": reactHooks,
    },
    // @ts-ignore
    rules: reactHooks.configs.recommended.rules, 
  },
  {
    plugins: {
      "react-refresh": reactRefresh,
    },
    rules: {
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
    },
  },
);
